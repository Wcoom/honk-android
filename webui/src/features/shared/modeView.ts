import type {Group} from '../../api/model';
import type {Translator as LabelFn} from '../../i18n';
import type {OutboundMode} from '../../dae/outboundMode';
import type {EngineReason} from '../../api/engines';
import type {ModeDraft} from './useModeDraft';

export const modeLabels = {rule: 'mode.rule', direct: 'mode.direct', global: 'mode.global'} as const;
export function modeView(
  current: OutboundMode,
  staged: ModeDraft | null,
  groups: Array<Pick<Group, 'name'>>,
  writable: boolean,
  configAvailable: boolean,
  t: LabelFn,
  known = true,
  reason?: EngineReason,
  savedTarget = '',
  policy?: {
    options?: Array<{id: string; label: string}>;
    activeTarget?: string;
    targetWritable?: boolean;
  }
) {
  const stagedMode = writable ? staged?.mode : undefined;
  const shownMode = stagedMode ?? current.mode;
  const targets = policy?.options ?? groups.map(group => ({id: group.name, label: group.name}));
  const targetWritable = policy?.targetWritable ?? writable;
  const stagedTarget = targetWritable ? staged?.target : undefined;
  const validSaved = savedTarget && (targets.length === 0 || targets.some(item => item.id === savedTarget)) ? savedTarget : '';
  const baseTarget = current.mode === 'global' ? current.target : policy?.activeTarget || validSaved || targets[0]?.id || '';
  const target = stagedTarget ?? baseTarget;
  const modeDirty = writable && stagedMode !== undefined && stagedMode !== current.mode;
  const targetDirty = targetWritable && stagedTarget !== undefined && stagedTarget !== baseTarget;
  return {
    mode: known ? shownMode : '',
    target: known ? target : '',
    targetText: known ? target || '—' : '—',
    writable,
    targetWritable,
    modeDirty,
    targetDirty,
    dirty: modeDirty || targetDirty,
    // Global mode needs a group to send everything to; without one there is nothing valid to write.
    incomplete: shownMode === 'global' && !target,
    targetIncomplete: !target,
    status: t(configAvailable ? 'act.modeReadOnly' : 'act.modeUnavailable'),
    readOnly: configAvailable && !writable,
    // Why the read-only mode cannot be written, when the engine gives a reason; the popover words it as the overview does.
    reason: configAvailable && !writable ? reason : undefined,
    modes: (['rule', 'direct', 'global'] as const).map(mode => [mode, t(modeLabels[mode])] as [string, string]),
    targets
  };
}
// Why the mode card's Apply and the policy target are disabled, each under its own card; an Apply with nothing to apply
// needs no line, the disabled button says so. A read-only backend's status
// already sits beside the mode switch, so only the policy selector repeats it. Null while they can be used or a change is
// being applied.
export function modeReasons(
  {writable, targetWritable = writable, incomplete, status}: Pick<ReturnType<typeof modeView>, 'writable' | 'incomplete' | 'status'> & {targetWritable?: boolean},
  busy: boolean,
  t: LabelFn
): {mode: string | null; global: string | null} {
  if (busy) return {mode: null, global: null};
  return {
    mode: writable && incomplete ? t('act.globalMissing') : null,
    global: !targetWritable ? status : null
  };
}
