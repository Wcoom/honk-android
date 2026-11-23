import {Badge} from '@astryxdesign/core/Badge';
import {Button} from '@astryxdesign/core/Button';
import {Card} from '@astryxdesign/core/Card';
import {Divider} from '@astryxdesign/core/Divider';
import {Icon} from '@astryxdesign/core/Icon';
import {HStack, StackItem, VStack} from '@astryxdesign/core/Layout';
import {Link} from '@astryxdesign/core/Link';
import {Heading, Text} from '@astryxdesign/core/Text';
import {useToast} from '@astryxdesign/core/Toast';
import {useCallback, useContext, useEffect, useRef, useState} from 'react';
import {coreUpdateCapability} from '../../../src/api/coreUpdate';
import type {Connection} from './api';
import {docsHref} from './docs';
import {LocaleContext, useMessages} from './i18n';
import {invalidateReads, noticeText, useRead} from './state';
import {Feedback} from './ui';
import {BrandMark} from './ui/BrandMark';
import {EditDialog} from './ui/EditDialog';
import {RowEntrance} from './ui/motion';

const repository = 'https://github.com/Zakkaus/doona';

function CoreUpdateSection({connection}: {connection: Connection}) {
  const t = useMessages();
  const toast = useToast();
  const capability = coreUpdateCapability(connection.capabilities);
  const status = useRead(useCallback((signal?: AbortSignal) => connection.api.coreUpdateStatus(false, signal), [connection]));
  const [checking, setChecking] = useState(false);
  const [applying, setApplying] = useState(false);
  const [confirmOpen, setConfirmOpen] = useState(false);
  const [actionError, setActionError] = useState<unknown>();
  const mounted = useRef(true);
  useEffect(() => {
    mounted.current = true;
    return () => {
      mounted.current = false;
    };
  }, []);

  const current = status.data;
  const busy = checking || applying || !!current?.applying;
  const canApply = current ? current.can_apply : (capability?.can_apply ?? false);
  const forceReinstall = !!current && !current.has_update && !!current.latest_version;

  const check = async () => {
    setChecking(true);
    setActionError(undefined);
    try {
      const next = await connection.api.coreUpdateStatus(true);
      if (!mounted.current) return;
      status.setData(next);
    } catch (error) {
      if (mounted.current) setActionError(error);
    } finally {
      if (mounted.current) setChecking(false);
    }
  };

  const apply = async () => {
    setConfirmOpen(false);
    setApplying(true);
    setActionError(undefined);
    try {
      const result = await connection.api.applyCoreUpdate(forceReinstall);
      if (!mounted.current) return;
      if (result.restarting) {
        await new Promise(resolve => setTimeout(resolve, 1500));
        let recovered = false;
        for (let attempt = 0; attempt < 30 && mounted.current; attempt++) {
          try {
            await connection.api.version();
            recovered = true;
            break;
          } catch {
            await new Promise(resolve => setTimeout(resolve, 1500));
          }
        }
        if (!mounted.current) return;
        if (!recovered) {
          toast({body: t.coreUpdateRestartTimeout, type: 'info', isAutoHide: true});
          return;
        }
      }
      invalidateReads();
      await status.reload().catch(() => {});
      if (!mounted.current) return;
      toast({
        body: noticeText(t.coreUpdateDone, {version: result.installed_version ?? current?.latest_version ?? current?.current_version ?? ''}),
        type: 'info',
        isAutoHide: true
      });
    } catch (error) {
      if (mounted.current) setActionError(error);
    } finally {
      if (mounted.current) setApplying(false);
    }
  };

  const rows = [
    {label: t.coreUpdateCurrent, value: current?.current_version ?? (status.loading ? t.loading : t.unknown)},
    {label: t.coreUpdateLatest, value: current?.latest_version ?? (status.loading ? t.loading : t.coreUpdateUnchecked)}
  ];
  const failure = actionError ?? status.error;

  return (
    <VStack gap={3}>
      <HStack gap={2} vAlign="center" wrap="wrap">
        <Heading level={2}>{t.coreUpdate}</Heading>
        {current?.latest_version && (
          <Badge
            label={
              current.has_update
                ? noticeText(t.coreUpdateAvailable, {latest: current.latest_version})
                : noticeText(t.coreUpdateUpToDate, {version: current.current_version})
            }
            variant={current.has_update ? 'warning' : 'success'}
          />
        )}
      </HStack>
      <Card padding={0} width="100%" variant="muted">
        <VStack gap={0}>
          {rows.map((row, index) => (
            <VStack key={row.label} gap={0}>
              {index > 0 && <Divider variant="subtle" />}
              <HStack padding={4} gap={3} vAlign="start">
                <StackItem size="fill">
                  <Text type="label">{row.label}</Text>
                </StackItem>
                <Text color="secondary" style={{overflowWrap: 'anywhere', minWidth: 0}}>
                  {row.value}
                </Text>
              </HStack>
            </VStack>
          ))}
        </VStack>
      </Card>
      <HStack gap={2} wrap="wrap" vAlign="center">
        <Button label={checking ? t.coreUpdateChecking : t.coreUpdateCheck} variant="secondary" onClick={() => void check()} isDisabled={busy} />
        <Button
          label={applying ? t.coreUpdateApplying : forceReinstall ? t.coreUpdateReinstall : t.coreUpdateApply}
          onClick={() => setConfirmOpen(true)}
          isDisabled={busy || !canApply || !current?.latest_version}
        />
        {current?.release_url && (
          <Link href={current.release_url} isExternalLink isStandalone>
            {t.coreUpdateOpenRelease}
          </Link>
        )}
      </HStack>
      {!!failure && <Feedback notice={{error: failure}} />}
      <EditDialog
        open={confirmOpen}
        title={t.coreUpdateConfirmTitle}
        close={() => setConfirmOpen(false)}
        pending={applying}
        confirm={() => void apply()}
        label={forceReinstall ? t.coreUpdateReinstall : t.coreUpdateApply}
        cancelLabel={t.cancel}
      >
        <Text>{noticeText(t.coreUpdateConfirmHelp, {latest: current?.latest_version ?? current?.current_version ?? ''})}</Text>
      </EditDialog>
    </VStack>
  );
}

export function AboutPanel({connection}: {connection: Connection}) {
  const t = useMessages();
  const locale = useContext(LocaleContext);
  const version = useRead(useCallback(() => connection.api.version(), [connection]));
  const engine = version.data?.engine;
  const rows = [
    {label: t.address, value: connection.address === 'mock' ? t.demoName : connection.address},
    {label: t.aboutBackend, value: engine ? `${engine.name} ${engine.version}` : version.loading ? t.loading : t.unknown}
  ];
  const coreUpdateAvailable = coreUpdateCapability(connection.capabilities)?.available === true;

  return (
    <VStack gap={4}>
      <HStack gap={3} vAlign="center">
        <Icon icon={BrandMark} size="lg" style={{width: 'var(--spacing-12)', height: 'var(--spacing-12)'}} />
        <VStack gap={1}>
          <HStack gap={2} vAlign="center" wrap="wrap">
            <Heading level={2}>{t.app}</Heading>
            <Badge label={`v${import.meta.env.VITE_DOONA_VERSION}`} variant="neutral" />
          </HStack>
          <Text type="supporting" color="secondary">
            {t.aboutTagline}
          </Text>
        </VStack>
      </HStack>
      <HStack gap={4} wrap="wrap">
        <Link href={docsHref(locale)} isExternalLink isStandalone>
          {t.aboutGuide}
        </Link>
        <Link href={repository} isExternalLink isStandalone>
          GitHub
        </Link>
        <Link href={`${repository}/releases`} isExternalLink isStandalone>
          {t.aboutReleases}
        </Link>
      </HStack>
      <VStack gap={3}>
        <Heading level={2}>{t.aboutConnection}</Heading>
        <Card padding={0} width="100%" variant="muted">
          <RowEntrance>
            <VStack as="ul" role="list" gap={0}>
              {rows.map((row, index) => (
                <VStack key={row.label} as="li" gap={0}>
                  {index > 0 && <Divider variant="subtle" />}
                  <HStack padding={4} gap={3} vAlign="start">
                    <StackItem size="fill">
                      <Text type="label">{row.label}</Text>
                    </StackItem>
                    <Text color="secondary" style={{overflowWrap: 'anywhere', minWidth: 0}}>
                      {row.value}
                    </Text>
                  </HStack>
                </VStack>
              ))}
            </VStack>
          </RowEntrance>
        </Card>
        {!!version.error && <Feedback notice={{error: version.error}} />}
      </VStack>
      {coreUpdateAvailable && <CoreUpdateSection connection={connection} />}
    </VStack>
  );
}
