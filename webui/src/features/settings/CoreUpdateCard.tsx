import {useCallback, useEffect, useRef, useState} from 'react';
import {coreUpdateCapability} from '../../api/coreUpdate';
import {getApi} from '../../api/index';
import type {CoreUpdateStatus} from '../../api/model';
import {LOCALE, useLang, useT} from '../../i18n';
import {formatBytes, localTime} from '../../i18n/format';
import {refetchAll, useCapabilities} from '../../store';
import {
  ActionHelp,
  Button,
  Card,
  ConfirmDialog,
  Disclosure,
  ErrorMessage,
  Kv,
  Light,
  Link,
  SkeletonBar,
  SkeletonGroup,
  toast,
  toastErrorDetail,
  type KvItem
} from '../../ui/ui';
import {settingsCard} from './nav';

const card = settingsCard('coreUpdate');
const RESTART_INITIAL_DELAY_MS = 1500;
const RESTART_POLL_INTERVAL_MS = 1500;
const RESTART_MAX_ATTEMPTS = 30;

const shortHash = (value: string | null | undefined) => (value ? value.slice(0, 12) : '—');

async function sleep(ms: number, signal?: AbortSignal) {
  await new Promise<void>((resolve, reject) => {
    const timer = setTimeout(() => {
      signal?.removeEventListener('abort', onAbort);
      resolve();
    }, ms);
    const onAbort = () => {
      clearTimeout(timer);
      reject(signal?.reason ?? new DOMException('Aborted', 'AbortError'));
    };
    signal?.addEventListener('abort', onAbort, {once: true});
  });
}

export function CoreUpdateCard() {
  const t = useT();
  const lang = useLang();
  const locale = LOCALE[lang];
  const api = getApi();
  const caps = useCapabilities();
  const capability = coreUpdateCapability(caps.data);
  const available = capability?.available === true;

  const [status, setStatus] = useState<CoreUpdateStatus | null>(null);
  const [loading, setLoading] = useState(true);
  const [checking, setChecking] = useState(false);
  const [applying, setApplying] = useState(false);
  const [confirmOpen, setConfirmOpen] = useState(false);
  const [error, setError] = useState<Error | null>(null);
  const mounted = useRef(true);
  const active = useRef<AbortController | null>(null);

  useEffect(() => {
    mounted.current = true;
    return () => {
      mounted.current = false;
      active.current?.abort();
    };
  }, []);

  const load = useCallback(
    async (refresh: boolean, announce = false) => {
      active.current?.abort();
      const controller = new AbortController();
      active.current = controller;
      if (refresh) setChecking(true);
      else setLoading(prev => prev && !status);
      setError(null);
      try {
        const next = await api.coreUpdateStatus(refresh, controller.signal);
        if (!mounted.current || controller.signal.aborted) return;
        setStatus(next);
        if (announce && next.latest_version) {
          toast(
            next.has_update ? 'neutral' : 'positive',
            next.has_update
              ? t('settings.coreUpdateAvailable', {latest: next.latest_version})
              : t('settings.coreUpdateUpToDateToast', {version: next.current_version})
          );
        }
      } catch (failure) {
        if (!mounted.current || controller.signal.aborted) return;
        setError(failure instanceof Error ? failure : new Error(String(failure)));
        if (refresh) toast('negative', t('settings.coreUpdateCheckFailed'), toastErrorDetail(failure, t));
      } finally {
        if (mounted.current && !controller.signal.aborted) {
          setLoading(false);
          setChecking(false);
        }
      }
    },
    [api, status, t]
  );

  useEffect(() => {
    if (!available) return;
    void load(false);
    // Initial read once when capability becomes available or api changes.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [available, api]);

  if (!available) return null;

  const busy = checking || applying || !!status?.applying;
  const canApply = status ? status.can_apply : (capability?.can_apply ?? false);
  const cannotApplyReason = status?.cannot_apply_reason ?? capability?.cannot_apply_reason ?? null;
  const forceReinstall = status !== null && !status.has_update && !!status.latest_version;

  const apply = async () => {
    setConfirmOpen(false);
    const controller = new AbortController();
    active.current?.abort();
    active.current = controller;
    setApplying(true);
    setError(null);
    try {
      const result = await api.applyCoreUpdate(forceReinstall, controller.signal);
      if (!mounted.current || controller.signal.aborted) return;
      if (result.status === 'up_to_date') {
        toast('positive', t('settings.coreUpdateUpToDateToast', {version: result.current_version ?? status?.current_version ?? ''}));
        await load(false);
        return;
      }
      if (result.restarting) {
        await sleep(RESTART_INITIAL_DELAY_MS, controller.signal);
        let recovered = false;
        for (let attempt = 0; attempt < RESTART_MAX_ATTEMPTS; attempt++) {
          if (controller.signal.aborted || !mounted.current) return;
          try {
            await api.version(controller.signal);
            recovered = true;
            break;
          } catch {
            if (controller.signal.aborted || !mounted.current) return;
            await sleep(RESTART_POLL_INTERVAL_MS, controller.signal);
          }
        }
        if (!mounted.current || controller.signal.aborted) return;
        if (!recovered) {
          toast('neutral', t('settings.coreUpdateRestartTimeout'));
          return;
        }
      }
      await refetchAll();
      if (!mounted.current || controller.signal.aborted) return;
      await load(false);
      toast(
        'positive',
        t('settings.coreUpdateDone', {
          version: result.installed_version ?? status?.latest_version ?? status?.current_version ?? ''
        })
      );
    } catch (failure) {
      if (!mounted.current || controller.signal.aborted) return;
      setError(failure instanceof Error ? failure : new Error(String(failure)));
      toast('negative', t('settings.coreUpdateApplyFailed'), toastErrorDetail(failure, t));
    } finally {
      if (mounted.current && !controller.signal.aborted) setApplying(false);
    }
  };

  const sourceLabel =
    status?.source === 'github_api'
      ? t('settings.coreUpdateSourceApi')
      : status?.source === 'github_web'
        ? t('settings.coreUpdateSourceWeb')
        : '—';

  const currentDisplay = status ? [status.current_version, shortHash(status.current_revision)].filter(Boolean).join(' · ') : '—';

  const items: KvItem[] = status
    ? [
        [t('settings.coreUpdateCurrent'), currentDisplay],
        [t('settings.coreUpdateTarget'), status.target],
        [t('settings.coreUpdateAsset'), status.asset_name],
        [t('settings.coreUpdateLatest'), status.latest_version ?? t('settings.coreUpdateUnchecked')],
        [t('settings.coreUpdateCheckedAt'), status.checked_at ? localTime(status.checked_at, locale) : t('settings.coreUpdateUnchecked')],
        [t('settings.coreUpdateSource'), sourceLabel],
        ...(status.published_at ? ([[t('settings.coreUpdatePublished'), localTime(status.published_at, locale)]] as KvItem[]) : []),
        ...(status.asset_size_bytes ? ([[t('settings.coreUpdateSize'), formatBytes(status.asset_size_bytes, locale)]] as KvItem[]) : []),
        ...(status.asset_sha256
          ? ([
              {
                label: t('settings.coreUpdateSha256'),
                value: shortHash(status.asset_sha256),
                full: status.asset_sha256
              }
            ] as KvItem[])
          : [])
      ]
    : [];

  return (
    <Card level={2} title={t(card.titleKey)} titleId={card.headingId}>
      <span className="rp-label">{t('settings.coreUpdateNote', {repo: status?.repo ?? capability?.repo ?? 'Wcoom/honk-android'})}</span>
      <ErrorMessage error={error} onRetry={() => void load(true)} />
      {loading && !status ? (
        <SkeletonGroup>
          <SkeletonBar width={220} />
          <SkeletonBar width={180} />
        </SkeletonGroup>
      ) : (
        status && (
          <>
            <div className="rp-cluster">
              {status.latest_version ? (
                <Light tone={status.has_update ? 'warn' : 'ok'}>
                  {status.has_update
                    ? t('settings.coreUpdateAvailable', {latest: status.latest_version})
                    : t('settings.coreUpdateUpToDate', {version: status.current_version})}
                </Light>
              ) : (
                <Light tone="muted">{t('settings.coreUpdateUnchecked')}</Light>
              )}
              {status.release_url && (
                <Link appearance="link" href={status.release_url} external>
                  {t('settings.coreUpdateOpenRelease')}
                </Link>
              )}
            </div>
            <Kv detail items={items} />
            {status.last_error && (
              <span className="rp-geodata-note negative" role="status">
                {t('ui.valuePair', {label: t('settings.coreUpdateLastError'), value: status.last_error.message})}
              </span>
            )}
            {status.notes && (
              <Disclosure title={t('settings.coreUpdateNotes')}>
                <pre className="rp-limit-snippet">{status.notes}</pre>
              </Disclosure>
            )}
          </>
        )
      )}
      <div className="rp-cluster">
        <Button isPending={checking} isDisabled={busy} onPress={() => void load(true, true)}>
          {checking ? t('settings.coreUpdateChecking') : t('settings.coreUpdateCheck')}
        </Button>
        <ActionHelp reason={!canApply ? cannotApplyReason : null}>
          <Button
            accent={status?.has_update}
            isPending={applying}
            isDisabled={busy || !canApply || !status?.latest_version}
            onPress={() => setConfirmOpen(true)}
          >
            {applying
              ? t('settings.coreUpdateApplying')
              : forceReinstall
                ? t('settings.coreUpdateReinstall')
                : t('settings.coreUpdateApply')}
          </Button>
        </ActionHelp>
      </div>
      {confirmOpen && status && (
        <ConfirmDialog
          title={t('settings.coreUpdateConfirmTitle')}
          tone="accent"
          isOpen
          onCancel={() => setConfirmOpen(false)}
          confirmLabel={forceReinstall ? t('settings.coreUpdateReinstall') : t('settings.coreUpdateApply')}
          isPending={applying}
          onConfirm={() => void apply()}
        >
          <p className="rp-label">
            {t('settings.coreUpdateConfirmHelp', {
              latest: status.latest_version ?? status.current_version,
              asset: status.asset_name,
              path: status.executable_path
            })}
          </p>
        </ConfirmDialog>
      )}
    </Card>
  );
}
