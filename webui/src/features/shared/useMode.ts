import {useMemo} from 'react';
import {getApi} from '../../api/index';
import {useCapabilities, useGroups, useVersion} from '../../store';
import {useConfig} from '../../store/config';
import {useMainSourceEdit} from '../../store/mainSource';
import {useResource} from '../../store/resource';
import {poll} from '../../store/cadence';
import {editProblem, editToast} from './pending';
import {useT} from '../../i18n';
import {toast} from '../../ui/ui';
import {useAction} from '../../store/action';
import {readSavedGlobalTarget, saveGlobalTarget, useModeDraft, useSavedGlobalTarget, type ModeDraft} from './useModeDraft';
import {readMode, readRuleFallback, writeMode, writeRuleFallback, type OutboundMode} from '../../dae/outboundMode';
import {modeLabels, modeReasons, modeView} from './modeView';
import {findPolicyHub, resolvePolicyState} from './policyTarget';
import {offered} from '../../api/capabilities';
import {failureNotice, LocalError, noticeText} from '../../api/error';
import {engineOf} from '../../api/engines';

function normalizeDraft(draft: ModeDraft, current: OutboundMode, activeTarget: string): ModeDraft | null {
  const next: ModeDraft = {};
  if (draft.mode !== undefined && draft.mode !== current.mode) next.mode = draft.mode;
  if (draft.target !== undefined && draft.target !== activeTarget) next.target = draft.target;
  return next.mode !== undefined || next.target !== undefined ? next : null;
}

export function useMode() {
  const t = useT();
  const api = getApi();
  const capabilities = useCapabilities().data;
  const resources = capabilities?.resources;
  const groupsEnabled = offered(resources, 'groups', {whileLoading: false});
  const groups = useGroups(groupsEnabled);
  const config = useConfig(offered(resources, 'config', {whileLoading: false}));
  const {main, checking, writable, busy, error, retry, apply: write} = useMainSourceEdit();
  // The mode is a rule doona writes into dae text, so another engine's configuration neither shows nor takes one.
  // Until the version names the engine the card waits as it does for the configuration.
  const version = useVersion().data;
  const daeText = engineOf(version).daeText;
  const content = daeText ? (main ?? config.data?.sources.find(source => source.kind === 'main'))?.content : undefined;
  const current = useMemo<OutboundMode>(() => (content == null ? {mode: 'rule'} : readMode(content)), [content]);
  const ruleFallback = useMemo(() => (content == null ? '' : readRuleFallback(content)), [content]);
  const hub = useMemo(() => findPolicyHub(groups.data ?? [], ruleFallback), [groups.data, ruleFallback]);
  const hubResource = useResource(
    {
      key: ['group', {id: hub?.id ?? ''}],
      every: poll.inventory,
      fetch: signal => api.group(hub!.id, signal)
    },
    {enabled: !!hub && groupsEnabled}
  );
  const [staged, setStaged] = useModeDraft();
  const [memoryTarget, setMemoryTarget] = useSavedGlobalTarget();
  const savedTarget = memoryTarget ?? readSavedGlobalTarget();
  const action = useAction<'apply'>({shared: 'mode-action'});
  const configAvailable = (daeText || !version) && !!resources?.config.available;
  const configWritable = daeText && writable && (!!main || checking);
  const policy = useMemo(
    () =>
      resolvePolicyState({
        current,
        groups: groups.data ?? [],
        hub,
        hubDetail: hubResource.data,
        ruleFallback,
        savedTarget
      }),
    [current, groups.data, hub, hubResource.data, ruleFallback, savedTarget]
  );
  const canSelectHub =
    policy.usesHub && !!groupsEnabled && resources?.groups.selection !== false && !!hubResource.data?.capabilities.can_select;
  const targetWritable = current.mode === 'global' ? configWritable : policy.usesHub ? canSelectHub : configWritable;
  // Each write changes the source, which is then checked again. The card keeps its controls meanwhile instead of
  // flashing the read-only status, so the Apply button a screen reader was just told about stays in place.
  const reason = capabilities && engineOf(version).reason('mode', capabilities);
  const view = modeView(current, staged, groups.data ?? [], configWritable, configAvailable, t, content != null, reason, savedTarget, {
    options: policy.options,
    activeTarget: policy.activeTarget,
    targetWritable
  });
  const persistTarget = (target: string) => {
    if (!target) return;
    saveGlobalTarget(target);
    setMemoryTarget(target);
  };
  const syncHubSelection = async (target: string, signal?: AbortSignal) => {
    if (!policy.usesHub || !canSelectHub || !hub) return false;
    const member = policy.hubMembers.find(item => item.name === target);
    if (!member) return false;
    await api.selectGroup(hub.id, {member_id: member.id, network: 'both'}, signal);
    return true;
  };
  const runWrite = (submitted: OutboundMode, onSuccess: () => void | Promise<void>) =>
    action.run('apply', async () => {
      const result = await write(text => writeMode(text, submitted));
      if (result.kind === 'ok') {
        await onSuccess();
      }
      // doona's own refusal is the whole story: its advice leads and the rule in the way is the second line.
      if (result.kind === 'failed' && result.error instanceof LocalError && result.error.key === 'act.modeInterleaved') {
        toast('negative', t(result.error.key), {detail: result.error.detail ?? undefined, error: result.error});
        return;
      }
      const problem = editProblem(result, config.data?.sources ?? [], t);
      if (problem) toast(...editToast(problem));
    });
  const applyMode = () => {
    const mode = staged?.mode;
    if (!mode || mode === current.mode) return;
    const target = view.target;
    if (mode === 'global' && !target) return;
    const submitted: OutboundMode = mode === 'global' ? {mode, target} : {mode};
    void runWrite(submitted, async () => {
      if (mode === 'global' && target) {
        persistTarget(target);
        await syncHubSelection(target).catch(() => undefined);
        await Promise.all([groups.refetch(), hubResource.refetch()]);
      }
      setStaged(prev => {
        if (!prev) return null;
        if (mode !== 'global') return prev.mode === mode ? null : {target: prev.target};
        const rest: ModeDraft = prev.mode === mode ? {target: prev.target === target ? undefined : prev.target} : prev;
        return rest.mode !== undefined || rest.target !== undefined ? rest : null;
      });
      toast('positive', t('act.modeApplied', {mode: t(modeLabels[mode])}));
    });
  };
  const clearStagedTarget = (target: string) =>
    setStaged(prev => {
      if (!prev) return null;
      const rest: ModeDraft = prev.target === target ? {mode: prev.mode} : prev;
      return rest.mode !== undefined || rest.target !== undefined ? rest : null;
    });
  const applyTarget = () => {
    const target = staged?.target;
    if (!target || target === policy.activeTarget) return;
    if (current.mode === 'global') {
      void runWrite({mode: 'global', target}, async () => {
        persistTarget(target);
        await syncHubSelection(target).catch(() => undefined);
        await Promise.all([groups.refetch(), hubResource.refetch()]);
        clearStagedTarget(target);
        toast('positive', t('act.globalApplied', {target}));
      });
      return;
    }
    if (policy.usesHub && hub && canSelectHub) {
      void action.run('apply', async signal => {
        try {
          const switched = await syncHubSelection(target, signal);
          if (!switched) return;
          persistTarget(target);
          clearStagedTarget(target);
          await Promise.all([groups.refetch(), hubResource.refetch()]);
          toast('positive', t('act.globalRuleApplied', {target, mode: t(modeLabels[current.mode])}));
        } catch (err) {
          if (signal.aborted) return;
          const notice = failureNotice(err, t, t('ui.writeFailed'));
          toast(notice.kind, noticeText(notice, t), {
            detail: notice.detail ?? undefined,
            requestId: notice.requestId,
            error: err instanceof Error ? err : undefined
          });
        }
      });
      return;
    }
    if (ruleFallback) {
      void action.run('apply', async () => {
        const result = await write(text => writeRuleFallback(text, target));
        if (result.kind === 'ok') {
          persistTarget(target);
          clearStagedTarget(target);
          await groups.refetch();
          toast('positive', t('act.globalRuleApplied', {target, mode: t(modeLabels[current.mode])}));
        }
        const problem = editProblem(result, config.data?.sources ?? [], t);
        if (problem) toast(...editToast(problem));
      });
      return;
    }
    persistTarget(target);
    clearStagedTarget(target);
    toast('positive', t('act.globalApplied', {target}));
  };
  const apply = () => {
    if (view.modeDirty) applyMode();
    else if (view.targetDirty) applyTarget();
  };
  return {
    ...view,
    reasons: modeReasons(view, busy || !!action.busy, t),
    error,
    retry,
    busy: busy || !!action.busy,
    pick: (mode: string) => {
      if (mode === 'global' || mode === 'direct' || mode === 'rule') {
        setStaged(prev => normalizeDraft({...prev, mode}, current, policy.activeTarget));
      }
    },
    pickTarget: (target: string) => {
      setStaged(prev => normalizeDraft({...prev, target}, current, policy.activeTarget));
    },
    apply,
    applyMode,
    applyTarget
  };
}
