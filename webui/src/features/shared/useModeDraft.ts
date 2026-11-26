import type {OutboundMode} from '../../dae/outboundMode';
import {pinnedProfile, profileOwner} from '../../api/profiles';
import {storageKeys} from '../../api/storage';
import {useSharedControl} from '../../store/sharedControl';

export type ModeDraft = {
  mode?: OutboundMode['mode'];
  target?: string;
};

export const useModeDraft = () => useSharedControl<ModeDraft | null>('mode-draft', null);

type SavedTargets = Record<string, string>;
const owner = () => {
  const profile = pinnedProfile();
  return profileOwner(profile?.id ?? '', profile?.api ?? '');
};

export function readSavedGlobalTarget(): string {
  try {
    const value: unknown = JSON.parse(localStorage.getItem(storageKeys.globalOutbound) ?? '{}');
    if (!value || typeof value !== 'object' || Array.isArray(value)) return '';
    const target = (value as SavedTargets)[owner()];
    return typeof target === 'string' ? target : '';
  } catch {
    return '';
  }
}

export function saveGlobalTarget(target: string) {
  let saved: SavedTargets = {};
  try {
    const value: unknown = JSON.parse(localStorage.getItem(storageKeys.globalOutbound) ?? '{}');
    if (value && typeof value === 'object' && !Array.isArray(value)) saved = value as SavedTargets;
  } catch {
    /* Start from an empty map when storage cannot be parsed. */
  }
  const key = owner();
  if (target) saved[key] = target;
  else delete saved[key];
  try {
    localStorage.setItem(storageKeys.globalOutbound, JSON.stringify(saved));
  } catch {
    /* Retain this session's target when browser storage is unavailable. */
  }
}

// Per-backend preferred global outbound shared across Activity cards and the widget panel.
export const useSavedGlobalTarget = () => useSharedControl<string | null>('global-outbound', null);
