import {Suspense, useEffect, useState, type ComponentProps} from 'react';
import {useT, withCatalogue} from '../../i18n';
import type {EngineReason} from '../../api/engines';
import {Button, Card, ConfirmDialog, ErrorMessage, Light, ChoiceMenu, PopoverDialog, Segmented, WidestLabel} from '../../ui/ui';
import {LoadBoundary} from '../../ui/LoadBoundary';
import {preloadable} from '../../ui/preloadable';
import type {ModeReadOnlyHint} from './ModeReadOnlyHint';
import Shuffle from '../../ui/icons/Shuffle';
import Filter from '../../ui/icons/Filter';
// The read-only explanation and its messages load once the status shows, so the popover opens with its text at once.
const hint = preloadable<ComponentProps<typeof ModeReadOnlyHint>>(
  withCatalogue('rest', () => import('./ModeReadOnlyHint').then(module => ({default: module.ModeReadOnlyHint})))
);
const preloadHint = () => void hint.preload().catch(() => undefined);
type ModeCardsModel = {
  mode: string;
  target: string;
  targetText: string;
  writable: boolean;
  targetWritable?: boolean;
  dirty: boolean;
  modeDirty?: boolean;
  targetDirty?: boolean;
  incomplete: boolean;
  targetIncomplete?: boolean;
  status: string;
  readOnly: boolean;
  reason?: EngineReason;
  modes: Array<[string, string]>;
  targets: Array<{id: string; label: string}>;
  busy: boolean;
  reasons: {mode: string | null; global: string | null};
  error: Error | null;
  retry: () => void;
  pick: (mode: string) => void;
  pickTarget: (target: string) => void;
  apply: () => void;
  applyMode?: () => void;
  applyTarget?: () => void;
};

type ApplyScope = 'mode' | 'global' | 'both';

export function ModeApply({
  model: vm,
  scope = 'both'
}: {
  model: Pick<
    ModeCardsModel,
    'mode' | 'modes' | 'writable' | 'targetWritable' | 'dirty' | 'modeDirty' | 'targetDirty' | 'incomplete' | 'targetIncomplete' | 'busy' | 'apply' | 'applyMode' | 'applyTarget'
  >;
  scope?: ApplyScope;
}) {
  const t = useT();
  const [asking, setAsking] = useState<string | null>(null);
  const canWrite = scope === 'global' ? (vm.targetWritable ?? vm.writable) : vm.writable;
  const dirty = scope === 'mode' ? (vm.modeDirty ?? vm.dirty) : scope === 'global' ? (vm.targetDirty ?? vm.dirty) : vm.dirty;
  const incomplete = scope === 'global' ? (vm.targetIncomplete ?? vm.incomplete) : vm.incomplete;
  const needsConfirm = scope !== 'global' && vm.mode !== 'rule';
  const confirmMode = vm.mode;
  const submit = scope === 'mode' ? (vm.applyMode ?? vm.apply) : scope === 'global' ? (vm.applyTarget ?? vm.apply) : vm.apply;
  const confirmLabel = vm.modes.find(([id]) => id === asking)?.[1] ?? '';
  return canWrite ? (
    <>
      <Button accent isDisabled={!dirty || incomplete} isPending={vm.busy} onPress={() => (needsConfirm ? setAsking(confirmMode) : submit())}>
        {t('config.save')}
      </Button>
      <ConfirmDialog
        title={t('act.modeConfirm', {mode: confirmLabel})}
        confirmLabel={t('config.save')}
        isOpen={asking !== null}
        isDisabled={vm.busy || !dirty || incomplete || asking !== confirmMode}
        onCancel={() => setAsking(null)}
        onConfirm={() => {
          setAsking(null);
          submit();
        }}
      >
        <p className="rp-label">{t(asking === 'direct' ? 'act.modeDirectHelp' : 'act.modeGlobalHelp')}</p>
      </ConfirmDialog>
    </>
  ) : null;
}

// The mode choice with its apply action, or why it cannot change: Activity's mode card and the widget panel share it.
export function ModeSwitch({model: vm, fill, scope = 'mode'}: {model: ModeCardsModel; fill?: boolean; scope?: ApplyScope}) {
  const t = useT();
  useEffect(() => {
    if (vm.readOnly) preloadHint();
  }, [vm.readOnly]);
  return (
    <span className="rp-cluster">
      <Segmented label={t('act.mode')} value={vm.mode} onChange={vm.pick} isDisabled={vm.busy || !vm.writable} items={vm.modes} fill={fill} />
      {vm.writable ? (
        <ModeApply model={vm} scope={scope} />
      ) : vm.readOnly ? (
        <PopoverDialog
          label={t('act.modeWhyReadOnly')}
          placement="bottom end"
          trigger={
            <Button quiet label={t('act.modeWhyReadOnly')}>
              {vm.status}
            </Button>
          }
        >
          {() => (
            <LoadBoundary>
              <Suspense fallback={null}>
                <hint.Component reason={vm.reason} />
              </Suspense>
            </LoadBoundary>
          )}
        </PopoverDialog>
      ) : (
        <Light small tone="muted">
          {vm.status}
        </Light>
      )}
    </span>
  );
}

export function ModeCards({model: vm, part}: {model: ModeCardsModel; part?: 'mode' | 'global'}) {
  const t = useT();
  return (
    <>
      {part !== 'global' && vm.error && <ErrorMessage error={vm.error} onRetry={vm.retry} />}
      {part !== 'global' && (
        <Card
          title={t('act.mode')}
          aria-label={t('act.mode')}
          tile={{icon: <Shuffle />, tint: 3, kind: 'control'}}
          reason={vm.reasons.mode}
          aside={<ModeSwitch model={vm} scope="mode" />}
        />
      )}
      {part !== 'mode' && (
        <Card
          title={t('act.global')}
          tile={{icon: <Filter />, tint: 2, kind: 'control'}}
          reason={vm.reasons.global}
          aside={
            <span className="rp-cluster rp-global-controls">
              <ChoiceMenu
                quiet
                isDisabled={vm.busy || !(vm.targetWritable ?? vm.writable)}
                label={t('act.global')}
                value={vm.target}
                onChange={vm.pickTarget}
                items={vm.targets}
                searchLabel={t('ui.filterOutbounds')}
              >
                <WidestLabel labels={vm.targets.map(target => target.label)}>{vm.targetText}</WidestLabel>
              </ChoiceMenu>
              <ModeApply model={vm} scope="global" />
            </span>
          }
        />
      )}
    </>
  );
}
