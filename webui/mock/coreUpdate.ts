import type {Api} from '../src/api/api';
import {coreUpdateCapability} from '../src/api/coreUpdate';
import {ApiError} from '../src/api/error';
import type {Capabilities, CoreUpdateApplyResult, CoreUpdateStatus} from '../src/api/model';
import {observedAt} from './fixtures/clock';

export function createCoreUpdate(capabilities: Capabilities, faults: boolean) {
  const capability = coreUpdateCapability(capabilities);
  const available = capability?.available === true;
  let currentVersion = 'v1.0.5';
  let latestVersion: string | null = 'v1.0.6';
  let checkedAt: string | null = observedAt;
  let source: CoreUpdateStatus['source'] = 'github_api';

  const status = (): CoreUpdateStatus => ({
    observed_at: new Date().toISOString(),
    repo: capability?.repo ?? 'Wcoom/honk-android',
    current_version: currentVersion,
    current_revision: 'b304679',
    target: capability?.target ?? 'aarch64-unknown-linux-musl',
    asset_name: capability?.asset ?? 'honk-android-arm64',
    executable_path: '/data/user/0/com.boxproxy.honk/files/box/bin/honk',
    can_apply: capability?.can_apply ?? true,
    cannot_apply_reason: capability?.cannot_apply_reason ?? null,
    applying: false,
    checked_at: checkedAt,
    source,
    latest_version: latestVersion,
    has_update: latestVersion !== null && latestVersion !== currentVersion,
    release_url: `https://github.com/${capability?.repo ?? 'Wcoom/honk-android'}/releases/tag/${latestVersion ?? currentVersion}`,
    published_at: observedAt,
    notes: 'Demo core update release notes.',
    asset_size_bytes: '26214400',
    asset_sha256: '0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef',
    last_error: faults ? {code: 'fetch_failed', message: 'GitHub release check failed in faults scenario'} : null
  });

  const requireCapability = () => {
    if (!available) throw new ApiError(404, 'capability_not_supported', 'Core update is unavailable');
  };

  const api: Pick<Api, 'coreUpdateStatus' | 'applyCoreUpdate'> = {
    coreUpdateStatus: async (refresh = false, signal) => {
      signal?.throwIfAborted();
      requireCapability();
      if (refresh && faults) throw new ApiError(502, 'fetch_failed', 'GitHub release check failed in faults scenario');
      if (refresh) {
        checkedAt = new Date().toISOString();
        source = 'github_api';
      }
      return status();
    },
    applyCoreUpdate: async (force = false, signal) => {
      signal?.throwIfAborted();
      requireCapability();
      if (faults) throw new ApiError(502, 'fetch_failed', 'Core binary download failed in faults scenario');
      const target = latestVersion ?? currentVersion;
      if (!force && target === currentVersion) {
        return {
          status: 'up_to_date',
          restarting: false,
          current_version: currentVersion,
          latest_version: target,
          has_update: false,
          release_url: status().release_url
        } satisfies CoreUpdateApplyResult;
      }
      const previous = currentVersion;
      currentVersion = target;
      return {
        status: 'restarting',
        restarting: true,
        previous_version: previous,
        installed_version: target,
        sha256: status().asset_sha256 ?? undefined,
        bytes: status().asset_size_bytes ?? undefined,
        release_url: status().release_url
      } satisfies CoreUpdateApplyResult;
    }
  };

  return {api};
}
