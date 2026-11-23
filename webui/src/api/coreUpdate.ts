import type {Capabilities} from './model';

export type CoreUpdateCapability = {
  available: boolean;
  can_apply?: boolean;
  cannot_apply_reason?: string | null;
  repo?: string;
  asset?: string;
  target?: string;
};

// Reads x-honk.core_update from either a normalized Capabilities object or a raw Capabilities['resources'] map.
export function coreUpdateCapability(
  source: Pick<Capabilities, 'extensions'> | Capabilities['resources'] | undefined
): CoreUpdateCapability | null {
  if (!source || typeof source !== 'object') return null;
  const honk =
    'extensions' in source
      ? (source.extensions?.['x-honk'] as Record<string, unknown> | undefined)
      : ((source as Record<string, unknown>)['x-honk'] as Record<string, unknown> | undefined);
  const raw = honk?.core_update;
  if (!raw || typeof raw !== 'object') return null;
  const entry = raw as Record<string, unknown>;
  if (typeof entry.available !== 'boolean') return null;
  return {
    available: entry.available,
    can_apply: typeof entry.can_apply === 'boolean' ? entry.can_apply : undefined,
    cannot_apply_reason: typeof entry.cannot_apply_reason === 'string' || entry.cannot_apply_reason === null ? entry.cannot_apply_reason : undefined,
    repo: typeof entry.repo === 'string' ? entry.repo : undefined,
    asset: typeof entry.asset === 'string' ? entry.asset : undefined,
    target: typeof entry.target === 'string' ? entry.target : undefined
  };
}
