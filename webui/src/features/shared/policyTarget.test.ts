import {expect, it} from 'vitest';
import type {Group, GroupSummary} from '../../api/model';
import {findPolicyHub, resolvePolicyState} from './policyTarget';

const summary = (
  id: string,
  name: string,
  kind: GroupSummary['policy']['kind'],
  tcpMemberId: string | null = null,
  udpMemberId: string | null = tcpMemberId
): GroupSummary => ({
  id,
  name,
  icon: null,
  config_revision: 'rev-1',
  policy: {kind, native: kind},
  member_count: 1,
  selection: {tcp_member_id: tcpMemberId, udp_member_id: udpMemberId}
});

const phoneGroups = (hubSelectedId = 'all-id'): GroupSummary[] => [
  summary('cdn-id', '自建 CDN 优选', 'score'),
  summary('all-id', '智能策略 / ALL', 'score'),
  summary('hk-id', '香港节点', 'score'),
  summary('manual-id', '手动选择', 'selector'),
  summary('hub-id', '总模式', 'selector', hubSelectedId),
  summary('youtube-id', 'YouTube', 'selector', 'hub-id'),
  summary('google-id', 'Google', 'selector', 'hub-id'),
  summary('fallback-id', '漏网之鱼', 'selector', 'hub-id')
];

const phoneHubDetail = (hubSelectedId = 'all-id'): Group => ({
  id: 'hub-id',
  name: '总模式',
  icon: null,
  config_revision: 'rev-1',
  policy: {kind: 'selector', native: 'select'},
  capabilities: {
    can_select: true,
    can_override: false,
    mutable_config: ['policy', 'default_member_id'],
    probe_transports: ['tcp', 'udp'],
    supports_nested_groups: true
  },
  config: {
    check_interval: null,
    check_url: null,
    default_member_id: 'all-id',
    final_outbound: null,
    idle_timeout: null,
    interrupt_connections: null,
    tolerance: null
  },
  members: [
    {id: 'cdn-node-1', name: '🇯🇵 东京 CDN', kind: 'node'},
    {id: 'cdn-id', name: '自建 CDN 优选', kind: 'group'},
    {id: 'all-id', name: '智能策略 / ALL', kind: 'group'},
    {id: 'hk-id', name: '香港节点', kind: 'group'},
    {id: 'manual-id', name: '手动选择', kind: 'group'}
  ],
  runtime: {
    selection: {
      tcp: {member_id: hubSelectedId, resolved_leaf_node_id: 'leaf-1', source: 'runtime'},
      udp: {member_id: hubSelectedId, resolved_leaf_node_id: 'leaf-1', source: 'runtime'}
    },
    health: []
  }
});

it('identifies the central selector hub in a multi-service routing hierarchy', () => {
  const groups = phoneGroups('cdn-id');
  expect(findPolicyHub(groups, '漏网之鱼')?.name).toBe('总模式');
  const resolved = resolvePolicyState({
    current: {mode: 'rule'},
    groups,
    hub: groups.find(g => g.name === '总模式')!,
    hubDetail: phoneHubDetail('cdn-id'),
    ruleFallback: '漏网之鱼',
    savedTarget: '智能策略 / ALL'
  });
  expect(resolved).toMatchObject({
    hubId: 'hub-id',
    usesHub: true,
    activeTarget: '自建 CDN 优选'
  });
  // Exposes only nested group policies of the hub (nodes excluded, non-hub service groups excluded).
  expect(resolved.options.map(option => option.id)).toEqual(['自建 CDN 优选', '智能策略 / ALL', '香港节点', '手动选择']);
});

it('tracks switching the hub policy between CDN and ALL while keeping rule mode', () => {
  const groups = phoneGroups('all-id');
  const resolved = resolvePolicyState({
    current: {mode: 'rule'},
    groups,
    hub: groups.find(g => g.name === '总模式')!,
    hubDetail: phoneHubDetail('all-id'),
    ruleFallback: '漏网之鱼',
    savedTarget: '自建 CDN 优选'
  });
  expect(resolved.activeTarget).toBe('智能策略 / ALL');
});

it('falls back cleanly on flat group topologies without a selector hub', () => {
  const flat = [summary('proxy', 'proxy', 'selector', 'n1'), summary('auto', 'auto', 'urltest'), summary('gaming', 'gaming', 'urltest')];
  expect(findPolicyHub(flat, 'proxy')?.name).toBe('proxy');
  // Without nested group members inside the flat selector, usesHub stays false and lists all groups.
  const detail: Group = {
    ...phoneHubDetail('n1'),
    id: 'proxy',
    name: 'proxy',
    members: [{id: 'n1', name: 'hk-01', kind: 'node'}]
  };
  const resolved = resolvePolicyState({
    current: {mode: 'rule'},
    groups: flat,
    hub: flat[0],
    hubDetail: detail,
    ruleFallback: 'gaming',
    savedTarget: 'auto'
  });
  expect(resolved).toMatchObject({usesHub: false, activeTarget: 'auto'});
  expect(resolved.options.map(option => option.id)).toEqual(['proxy', 'auto', 'gaming']);
});

it('preserves an external global target when already in global mode', () => {
  const groups = phoneGroups('cdn-id');
  const resolved = resolvePolicyState({
    current: {mode: 'global', target: '漏网之鱼'},
    groups,
    hub: groups.find(g => g.name === '总模式')!,
    hubDetail: phoneHubDetail('cdn-id'),
    ruleFallback: '漏网之鱼',
    savedTarget: ''
  });
  expect(resolved.activeTarget).toBe('漏网之鱼');
  expect(resolved.options.some(option => option.id === '漏网之鱼')).toBe(true);
});
