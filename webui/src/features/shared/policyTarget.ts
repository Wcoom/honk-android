import type {Group, GroupSummary} from '../../api/model';
import type {OutboundMode} from '../../dae/outboundMode';

type PolicyHubState = {
  hubId: string | null;
  hubMembers: Array<{id: string; name: string}>;
  options: Array<{id: string; label: string}>;
  activeTarget: string;
  usesHub: boolean;
};

/**
 * Finds the selector hub group that downstream groups and rules delegate to (e.g. `总模式` or `proxy`),
 * or falls back to the routing fallback selector when no nested hub exists.
 */
export function findPolicyHub(groups: readonly GroupSummary[], fallbackName: string): GroupSummary | null {
  const selectors = groups.filter(group => group.policy.kind === 'selector');
  if (selectors.length === 0) return null;

  const inDegree = new Map<string, number>();
  for (const selector of selectors) {
    let count = 0;
    for (const group of groups) {
      if (group.id === selector.id) continue;
      if (group.selection.tcp_member_id === selector.id || group.selection.udp_member_id === selector.id) count++;
    }
    inDegree.set(selector.id, count);
  }

  // Follow selector selections starting from the routing fallback to break ties along the active routing path.
  const fallbackChain = new Set<string>();
  const byId = new Map(selectors.map(group => [group.id, group]));
  let cursor = groups.find(group => group.name === fallbackName);
  while (cursor && cursor.policy.kind === 'selector' && !fallbackChain.has(cursor.id)) {
    fallbackChain.add(cursor.id);
    const nextId = cursor.selection.tcp_member_id ?? cursor.selection.udp_member_id;
    cursor = nextId ? byId.get(nextId) : undefined;
  }

  let best: GroupSummary | null = null;
  let bestDegree = 0;
  let bestOnChain = false;
  for (const selector of selectors) {
    const degree = inDegree.get(selector.id) ?? 0;
    if (degree < 1) continue;
    const onChain = fallbackChain.has(selector.id);
    if (degree > bestDegree || (degree === bestDegree && onChain && !bestOnChain)) {
      best = selector;
      bestDegree = degree;
      bestOnChain = onChain;
    }
  }
  if (best) return best;

  return selectors.find(group => group.name === fallbackName) ?? null;
}

/**
 * Computes the available policy targets and currently active policy for the Node Policy selector card
 * across rule, direct, and global outbound modes.
 */
export function resolvePolicyState({
  current,
  groups,
  hub,
  hubDetail,
  ruleFallback,
  savedTarget
}: {
  current: OutboundMode;
  groups: readonly GroupSummary[];
  hub: GroupSummary | null;
  hubDetail?: Group;
  ruleFallback: string;
  savedTarget: string;
}): PolicyHubState {
  const hubGroupMembers = (hubDetail?.members ?? [])
    .filter(member => member.kind === 'group')
    .map(member => ({id: member.id, name: member.name}));

  if (hub && hubGroupMembers.length > 0) {
    const selectedId =
      hubDetail?.runtime.selection.tcp?.member_id ??
      hubDetail?.runtime.selection.udp?.member_id ??
      hub.selection.tcp_member_id ??
      hub.selection.udp_member_id;
    const hubActive = hubGroupMembers.find(member => member.id === selectedId)?.name ?? groups.find(group => group.id === selectedId)?.name ?? '';
    const options = hubGroupMembers.map(member => ({id: member.name, label: member.name}));
    if (current.mode === 'global' && current.target && !options.some(option => option.id === current.target)) {
      options.push({id: current.target, label: current.target});
    }
    const validSaved = options.some(option => option.id === savedTarget) ? savedTarget : '';
    const activeTarget = current.mode === 'global' ? current.target : hubActive || validSaved || options[0]?.id || '';
    return {
      hubId: hub.id,
      hubMembers: hubGroupMembers,
      options,
      activeTarget,
      usesHub: true
    };
  }

  const options = groups.map(group => ({id: group.name, label: group.name}));
  if (current.mode === 'global' && current.target && !options.some(option => option.id === current.target)) {
    options.push({id: current.target, label: current.target});
  }
  if (hub) {
    const selectedId = hub.selection.tcp_member_id ?? hub.selection.udp_member_id;
    const hubActive = groups.find(group => group.id === selectedId)?.name ?? '';
    const validSaved = options.some(option => option.id === savedTarget) ? savedTarget : '';
    const activeTarget = current.mode === 'global' ? current.target : hubActive || validSaved || options[0]?.id || '';
    return {
      hubId: hub.id,
      hubMembers: [],
      options,
      activeTarget,
      usesHub: false
    };
  }

  const validFallback = ruleFallback && groups.some(group => group.name === ruleFallback) ? ruleFallback : '';
  const validSaved = options.some(option => option.id === savedTarget) ? savedTarget : '';
  const activeTarget = current.mode === 'global' ? current.target : validFallback || validSaved || options[0]?.id || '';
  return {
    hubId: null,
    hubMembers: [],
    options,
    activeTarget,
    usesHub: false
  };
}
