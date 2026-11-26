import {useMemo} from 'react';
import {useCapabilities, useGroups, useVersion} from '../../store';
import {useConfig} from '../../store/config';
import {useMainSourceEdit} from '../../store/mainSource';
import {editProblem, editToast} from './pending';
import {useT} from '../../i18n';
import {toast} from '../../ui/ui';
import {useAction} from '../../store/action';
import {readSavedGlobalTarget, saveGlobalTarget, useModeDraft, useSavedGlobalTarget, type ModeDraft} from './useModeDraft';
import {readMode, writeMode, type OutboundMode} from '../../dae/outboundMode';
import {modeLabels, modeReasons, modeView} from './modeView';
import {offered} from '../../api/capabilities';
import {LocalError} from '../../api/error';
import {engineOf} from '../../api/engines';

function normalizeDraft(draft: ModeDraft, current: OutboundMode): ModeDraft | null {
  const next: ModeDraft = {};
  if (draft.mode !== undefined && draft.mode !== current.mode) next.mode = draft.mode;
  if (current.mode === 'global' && draft.target !== undefined && draft.target !== current.target) next.target = draft.target;
  return next.mode !== undefined || next.target !== undefined ? next : null;
}

export function useMode() {
  const t = useT();
  const capabilities = useCapabilities().data;
  const resources = capabilities?.resources;
  const groups = useGroups(offered(resources, 'groups', {whileLoading: false}));
  const config = useConfig(offered(resources, 'config', {whileLoading: false}));
  const {main, checking, writable, busy, error, retry, apply: write} = useMainSourceEdit();
  // The mode is a rule doona writes into dae text, so another engine's configuration neither shows nor takes one.
  // Until the version names the engine the card waits as it does for the configuration.
  const version = useVersion().data;
  const daeText = engineOf(version).daeText;
  const content = daeText ? (main ?? config.data?.sources.find(source => source.kind === 'main'))?.content : undefined;
  const current = useMemo<OutboundMode>(() => (content == null ? {mode: 'rule'} : readMode(content)), [content]);
  const [staged, setStaged] = useModeDraft();
  const [memoryTarget, setMemoryTarget] = useSavedGlobalTarget();
  const savedTarget = memoryTarget ?? readSavedGlobalTarget();
  const action = useAction<'apply'>({shared: 'mode-action'});
  const configAvailable = (daeText || !version) && !!resources?.config.available;
  // Each write changes the source, which is then checked again. The card keeps its controls meanwhile instead of
  // flashing the read-only status, so the Apply button a screen reader was just told about stays in place.
  const reason = capabilities && engineOf(version).reason('mode', capabilities);
  const view = modeView(
    current,
    staged,
    groups.data ?? [],
    daeText && writable && (!!main || checking),
    configAvailable,
    t,
    content != null,
    reason,
    savedTarget
  );
  const persistTarget = (target: string) => {
    if (!target) return;
    saveGlobalTarget(target);
    setMemoryTarget(target);
  };
  const runWrite = (submitted: OutboundMode, onSuccess: () => void) =>
    action.run('apply', async () => {
      const result = await write(text => writeMode(text, submitted));
      if (result.kind === 'ok') {
        onSuccess();
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
    void runWrite(submitted, () => {
      if (target) persistTarget(target);
      setStaged(prev => {
        if (!prev) return null;
        if (mode !== 'global') return prev.mode === mode ? null : prev;
        const rest: ModeDraft = prev.mode === mode ? {target: prev.target === target ? undefined : prev.target} : prev;
        return rest.mode !== undefined || rest.target !== undefined ? rest : null;
      });
      toast('positive', t('act.modeApplied', {mode: t(modeLabels[mode])}));
    });
  };
  const applyTarget = () => {
    if (current.mode !== 'global') return;
    const target = staged?.target;
    if (!target || target === current.target) return;
    void runWrite({mode: 'global', target}, () => {
      persistTarget(target);
      setStaged(prev => {
        if (!prev) return null;
        const rest: ModeDraft = prev.target === target ? {mode: prev.mode} : prev;
        return rest.mode !== undefined || rest.target !== undefined ? rest : null;
      });
      toast('positive', t('act.globalApplied', {target}));
    });
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
        setStaged(prev => normalizeDraft({...prev, mode}, current));
      }
    },
    pickTarget: (target: string) => {
      if (current.mode === 'global') {
        setStaged(prev => normalizeDraft({...prev, target}, current));
        return;
      }
      // Outside global mode, choosing a global outbound only updates the remembered target without switching outbound mode.
      if (target && target !== view.target) {
        persistTarget(target);
        toast('positive', t('act.globalSaved', {target}));
      }
    },
    apply,
    applyMode,
    applyTarget
  };
}
