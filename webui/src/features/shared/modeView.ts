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
  savedTarget = ''
) {
  const stagedMode = writable ? staged?.mode : undefined;
  const stagedTarget = writable ? staged?.target : undefined;
  const shownMode = stagedMode ?? current.mode;
  const validSaved = savedTarget && (groups.length === 0 || groups.some(group => group.name === savedTarget)) ? savedTarget : '';
  const baseTarget = current.mode === 'global' ? current.target : validSaved || groups[0]?.name || '';
  const target = stagedTarget ?? baseTarget;
  const modeDirty = writable && stagedMode !== undefined && stagedMode !== current.mode;
  const targetDirty = writable && current.mode === 'global' && stagedTarget !== undefined && stagedTarget !== current.target;
  return {
    mode: known ? shownMode : '',
    target: known ? target : '',
    targetText: known ? target || '—' : '—',
    writable,
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
    targets: groups.map(group => ({id: group.name, label: group.name}))
  };
}
// Why the mode card's Apply and the global target are disabled, each under its own card; an Apply with nothing to apply
// needs no line, the disabled button says so. A read-only backend's status
// already sits beside the mode switch, so only the global target repeats it. Null while they can be used or a change is
// being applied.
export function modeReasons(
  {writable, incomplete, status}: Pick<ReturnType<typeof modeView>, 'writable' | 'incomplete' | 'status'>,
  busy: boolean,
  t: LabelFn
): {mode: string | null; global: string | null} {
  if (busy) return {mode: null, global: null};
  if (!writable) return {mode: null, global: status};
  return {mode: incomplete ? t('act.globalMissing') : null, global: null};
}
