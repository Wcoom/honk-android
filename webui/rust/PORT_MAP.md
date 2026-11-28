# Port map: webui

14 of 100 units have a Rust port; no original changed since it was recorded.

## Ports

| Unit | Crate | In Rust | Kept in the original | Cases | Last diff | Original |
| --- | --- | --- | --- | ---: | ---: | --- |
| `apps/lite/src/describe.ts` | `rust/describe-rs` | `describeGroupFilter`, `describeGroupFilters`, `policyKey`, `describeCondition`, `describeTarget`, `describeRule`, `phraseText`, `describeGroupChanges` | - | 542 | 542/542 | unchanged |
| `apps/lite/src/fetchFailure.ts` | `rust/fetchfailure-rs` | `fetchFailureText` | - | 56 | 56/56 | unchanged |
| `apps/lite/src/groupBindings.ts` | `rust/groupbindings-rs` | `regexLiterals`, `nodeBinding`, `subscriptionFilter`, `honkPolicy`, `regionPatterns`, `subscriptionBinding`, `readSubscriptionBinding` | - | 998 | 998/998 | unchanged |
| `apps/lite/src/groupDraft.ts` | `rust/groupdraft-rs` | `groupDraftEntry`, `changeGroupText`, `deleteGroup`, `renameGroup`, `groupChanges`, `emptyGroups` | - | 371 | 371/371 | unchanged |
| `apps/lite/src/groupHealth.ts` | `rust/grouphealth-rs` | `memberName`, `healthFallback`, `observedMembers`, `groupStatus` | - | 115 | 115/115 | unchanged |
| `apps/lite/src/resourceDraft.ts` | `rust/resourcedraft-rs` | `draftResources`, `changeResource` | - | 201 | 201/201 | unchanged |
| `apps/lite/src/ruleDraft.ts` | `rust/ruledraft-rs` | `kindsFor`, `serializeBuilder`, `findBlocks`, `parseRuleList`, `writeRuleList`, `moveRule`, `ensureBlock`, `addDraftRule`, `editDraftRule`, `setFallback`, `changeCount`, `stageAuthoredRules` | - | 1027 | 1027/1027 | unchanged |
| `apps/lite/src/ruleReview.ts` | `rust/rulereview-rs` | `ruleReview`, `dnsReview` | - | 128 | 128/128 | unchanged |
| `apps/lite/src/searchIndex.ts` | `rust/searchindex-rs` | `resourceIndex`, `searchNameRank` | - | 103 | 103/103 | unchanged |
| `src/api/hash.ts` | `rust/hash-rs` | `sha256`, `sha256Bytes` | `uuid` | 57 | 57/57 | unchanged |
| `src/api/rings.ts` | `rust/rings-rs` | `mean` | `bucket`, `append`, `window` | 25 | 25/25 | unchanged |
| `src/api/selectors.ts` | `rust/selectors-rs` | `safeHttpUrl`, `measured`, `preferredObservation`, `foldFamilies`, `preferredHealth`, `probeKinds`, `healthMillis`, `compareLatency`, `outboundUsage`, `simulatedAddress`, `ipLiteral`, `pseudoOwner`, `noNodeSources`, `pseudoOwnerId`, `nodeOwner`, `sourceIp`, `addressPort`, `chainNames`, `nodeSubscription`, `lifecycleTone`, `memoryTone`, `closedAllTone`, `connectionRows`, `routineGap`, `shortId`, `resolveSelectedLeaf`, `interestingNotice` | `outboundLabel`, `chainLabel`, `nodeLabel`, `chainPath`, `eventSummary` | 1086 | 1086/1086 | unchanged |
| `src/dae/` | `rust/dae-rs` | `serializeConditions`, `parseConditions`, `flagKey`, `validFlagName`, `hasEmbeddedFlag`, `validFlagChoice`, `flagChoices`, `matchPreset`, `missingCategories`, `geodataPreset`, `validGeodataUrl`, `conditionFamily`, `newGroupCondition`, `conditionValues`, `parseConditionValues`, `parseGroupConditions`, `serializeGroupConditions`, `groupFilterDraft`, `groupFilterText`, `groupFilterTexts`, `reconcileGroupFilters`, `newGroupFilter`, `groupAdmits`, `readGroupEntries`, `quoteName`, `isWritableName`, `groupNameProblem`, `writeGroupEntry`, `nameText`, `nestedIn`, `exactTokens`, `namedIn`, `classifyFilters`, `describeFilters`, `citingGroups`, `namedInExpression`, `groupsNamingTag`, `addSubtagsToGroup`, `addNodesToGroup`, `removeSubtagsFromGroup`, `splitTop`, `ruleCondition`, `includePatterns`, `isGlob`, `globMatch`, `includedBy`, `resolveIncludePath`, `includeDirectory`, `newSourcePathProblem`, `namePatterns`, `newSourceNameProblem`, `sourceAt`, `isProtocol`, `protocolName`, `emptyNode`, `parseNodeLink`, `nodeLink`, `securityOptions`, `usesTls`, `manualFields`, `advancedFields`, `settle`, `manualInvalid`, `manualLink`, `isNodeLink`, `readNodeEntries`, `readChainEntries`, `nodeReferenced`, `nodeReferrers`, `removeChainEntry`, `chainHopCandidate`, `tunnelDialable`, `chainHopProblem`, `chainUdpProblem`, `writeChainEntry`, `appendChainEntry`, `writeNodeEntry`, `readRuleFallback`, `writeRuleFallback`, `readMode`, `writeMode`, `sameMode`, `modeRuleLines`, `nodeProbeSupport`, `supportsNodeProbe`, `regionFlag`, `matchRegion`, `sourceFor`, `ruleAnchor`, `dnsRuleTarget`, `dnsRuleAnchor`, `dnsListEnd`, `dnsUpstreams`, `dnsUpstreamNames`, `ruleLine`, `addRule`, `replaceRuleTarget`, `removeRule`, `replaceRule`, `addFallback`, `numberSetting`, `settingValue`, `serializeSetting`, `writeSettings`, `holdsDns`, `holdsRouting`, `templateGroups`, `allGroupNames`, `allUpstreamNames`, `redacted`, `fileName`, `validationSources`, `restartSettings`, `restartRequired`, `unwrittenRestartSettings`, `completeSubscriptionUrl`, `isSubscriptionUrl`, `parseInterval`, `urlHost`, `agentProblem`, `readSubscriptionEntries`, `writeSubscriptionEntry`, `templateRules`, `detectTemplate`, `routingOptions`, `isPresetRouting`, `isBareName`, `quote`, `isQuotable`, `scanConfig`, `isFragment`, `unquote`, `uncomment`, `blockFields`, `blockBody`, `blockEntries`, `policyKind`, `isBuiltinOutbound`, `readXhttp`, `duplicateXhttpHeaders`, `writeXhttp` | `compileFilters`, `includeCheck`, `rustRegexToJs`, `writeTemplate`, `templateGroupLabel` | 7727 | 7703/7727 | unchanged |
| `src/features/shared/policyTarget.ts` | `rust/policytarget-rs` | `findPolicyHub`, `resolvePolicyState` | - | 57 | 57/57 | unchanged |

## Not ported yet

In the order `analyze` suggests.

| Unit | Status | Lines | Exports | Readiness |
| --- | --- | ---: | ---: | ---: |
| `apps/lite/src/config.ts` | ready | 135 | 11 | 61 |
| `apps/lite/src/routingTest.ts` | ready | 69 | 4 | 89 |
| `apps/lite/src/demo/` | ready | 52 | 1 | 100 |
| `apps/lite/src/operations.ts` | ready | 54 | 5 | 93 |
| `tools/e2e-shards-fixture/` | ready | 10 | 0 | 100 |
| `apps/lite/src/nodeProbeState.ts` | caution | 96 | 1 | 92 |
| `src/features/events/` | keep | 239 | 0 | 95 |
| `apps/lite/src/subscriptionDraft.ts` | caution | 47 | 2 | 95 |
| `apps/lite/src/useFeatureDraft.tsx` | keep | 82 | 0 | 92 |
| `apps/lite/src/preload.ts` | keep | 75 | 0 | 92 |
| `apps/lite/tools/` | ready | 150 | 0 | 76 |
| `apps/lite/src/connectionStartup.ts` | caution | 23 | 1 | 100 |
| `apps/lite/src/i18n/settings/` | ready | 133 | 0 | 99 |
| `public/` | ready | 74 | 0 | 85 |
| `src/features/logs/` | keep | 453 | 0 | 80 |
| `apps/lite/src/regions.ts` | ready | 5 | 1 | 100 |
| `src/shell/search/` | keep | 427 | 0 | 77 |
| `apps/lite/src/NodeProbes.tsx` | keep | 72 | 0 | 95 |
| `apps/lite/src/SubscriptionFeatureNodes.tsx` | keep | 101 | 0 | 93 |
| `apps/lite/src/SubscriptionFormInterval.tsx` | keep | 103 | 0 | 90 |
| `apps/lite/src/groupMembers.ts` | caution | 54 | 1 | 100 |
| `apps/lite/src/state.ts` | keep | 107 | 0 | 78 |
| `apps/lite/src/themes/shared/` | keep | 187 | 0 | 94 |
| `apps/lite/src/ui.tsx` | keep | 128 | 0 | 77 |
| `src/features/overview/` | keep | 632 | 0 | 88 |
| `apps/lite/src/AddNodeToGroup.tsx` | keep | 120 | 0 | 91 |
| `apps/lite/src/ConfigExport.tsx` | keep | 88 | 0 | 87 |
| `apps/lite/src/DnsUpstreams.tsx` | keep | 292 | 0 | 70 |
| `apps/lite/src/GroupTextDialog.tsx` | keep | 67 | 0 | 89 |
| `apps/lite/src/CopyMembers.tsx` | keep | 64 | 0 | 94 |
| `apps/lite/src/GroupEditor.tsx` | keep | 70 | 0 | 91 |
| `apps/lite/src/TemplatePicker.tsx` | keep | 71 | 0 | 91 |
| `src/api/engines/` | caution | 387 | 3 | 70 |
| `src/main.tsx` | keep | 135 | 0 | 89 |
| `apps/lite/src/AboutPanel.tsx` | keep | 219 | 0 | 84 |
| `apps/lite/src/GroupBindPopover.tsx` | keep | 165 | 0 | 87 |
| `apps/lite/src/RuleTester.tsx` | keep | 200 | 0 | 86 |
| `apps/lite/src/NodeForm.tsx` | keep | 364 | 0 | 78 |
| `apps/lite/src/ShareDialog.tsx` | keep | 67 | 0 | 89 |
| `apps/lite/src/api.ts` | caution | 196 | 16 | 65 |
| `apps/lite/src/Commands.tsx` | keep | 284 | 0 | 75 |
| `apps/lite/src/Demo.tsx` | keep | 36 | 0 | 93 |
| `apps/lite/src/Dns.tsx` | keep | 231 | 0 | 82 |
| `apps/lite/src/App.tsx` | keep | 319 | 0 | 79 |
| `apps/lite/src/systemFonts.ts` | ready | 1 | 1 | 100 |
| `apps/lite/src/Shell.tsx` | keep | 348 | 0 | 78 |
| `apps/lite/src/connectionStore.ts` | ready | 19 | 3 | 94 |
| `apps/lite/src/main.tsx` | keep | 13 | 0 | 96 |
| `apps/lite/src/Routing.tsx` | keep | 405 | 0 | 77 |
| `apps/lite/src/app/` | keep | 10 | 0 | 94 |
| `apps/lite/src/docs.ts` | ready | 6 | 1 | 95 |
| `apps/lite/src/ui/icons/` | keep | 561 | 0 | 76 |
| `src/fonts.ts` | blocked | 18 | 0 | 93 |
| `apps/lite/src/ui/motion/` | keep | 350 | 0 | 69 |
| `apps/lite/src/Connect.tsx` | keep | 305 | 0 | 70 |
| `apps/lite/src/GroupEditPanel.tsx` | keep | 471 | 0 | 63 |
| `apps/lite/src/themes/site/` | ready | 786 | 0 | 77 |
| `apps/lite/src/RuleList.tsx` | keep | 576 | 0 | 61 |
| `apps/lite/src/Subscriptions.tsx` | keep | 599 | 0 | 61 |
| `src/ui/icons/` | keep | 711 | 0 | 71 |
| `apps/lite/src/SettingsDialog.tsx` | keep | 532 | 0 | 56 |
| `src/features/activity/` | keep | 1348 | 0 | 49 |
| `src/features/flows/` | keep | 1155 | 0 | 43 |
| `src/features/policies/group/` | keep | 1186 | 0 | 47 |
| `apps/lite/src/GroupInspector.tsx` | keep | 616 | 0 | 63 |
| `src/i18n/` | keep | 619 | 0 | 43 |
| `apps/lite/src/Groups.tsx` | keep | 608 | 0 | 57 |
| `src/features/connections/` | keep | 1273 | 0 | 42 |
| `apps/lite/src/i18n/` | keep | 2231 | 0 | 39 |
| `src/features/policies/` | keep | 1601 | 0 | 37 |
| `apps/lite/src/Nodes.tsx` | keep | 786 | 0 | 49 |
| `src/features/dns/` | keep | 1421 | 0 | 33 |
| `src/ui/code/` | keep | 624 | 0 | 16 |
| `src/ui/charts/` | keep | 1868 | 0 | 17 |
| `tools/` | blocked | 2713 | 0 | 0 |
| `mock/` | caution | 3383 | 38 | 0 |
| `src/features/config/` | keep | 2071 | 0 | 14 |
| `src/features/rules/` | keep | 3288 | 0 | 0 |
| `src/store/` | keep | 2687 | 0 | 0 |
| `apps/lite/src/ui/` | keep | 1761 | 0 | 0 |
| `src/features/nodes/` | keep | 3487 | 0 | 0 |
| `src/features/settings/` | keep | 2762 | 0 | 0 |
| `src/shell/` | keep | 3154 | 0 | 0 |
| `src/shell/widgets/` | keep | 2760 | 0 | 0 |
| `src/ui/` | keep | 7309 | 0 | 0 |
| `apps/lite/scripts/` | caution | 4702 | 1 | 0 |

65 of these units have no port on purpose (2 blocked, 63 kept in the original), so 21 units are left to port:

- `src/features/events/`: keep, UI code: keep in the original language
- `apps/lite/src/useFeatureDraft.tsx`: keep, UI code: keep in the original language
- `apps/lite/src/preload.ts`: keep, UI code: keep in the original language
- `src/features/logs/`: keep, UI code: keep in the original language
- `src/shell/search/`: keep, UI code: keep in the original language
- `apps/lite/src/NodeProbes.tsx`: keep, UI code: keep in the original language
- `apps/lite/src/SubscriptionFeatureNodes.tsx`: keep, UI code: keep in the original language
- `apps/lite/src/SubscriptionFormInterval.tsx`: keep, UI code: keep in the original language
- `apps/lite/src/state.ts`: keep, UI code: keep in the original language
- `apps/lite/src/themes/shared/`: keep, UI code: keep in the original language
- `apps/lite/src/ui.tsx`: keep, UI code: keep in the original language
- `src/features/overview/`: keep, UI code: keep in the original language
- `apps/lite/src/AddNodeToGroup.tsx`: keep, UI code: keep in the original language
- `apps/lite/src/ConfigExport.tsx`: keep, UI code: keep in the original language
- `apps/lite/src/DnsUpstreams.tsx`: keep, UI code: keep in the original language
- `apps/lite/src/GroupTextDialog.tsx`: keep, UI code: keep in the original language
- `apps/lite/src/CopyMembers.tsx`: keep, UI code: keep in the original language
- `apps/lite/src/GroupEditor.tsx`: keep, UI code: keep in the original language
- `apps/lite/src/TemplatePicker.tsx`: keep, UI code: keep in the original language
- `src/main.tsx`: keep, UI code: keep in the original language
- `apps/lite/src/AboutPanel.tsx`: keep, UI code: keep in the original language
- `apps/lite/src/GroupBindPopover.tsx`: keep, UI code: keep in the original language
- `apps/lite/src/RuleTester.tsx`: keep, UI code: keep in the original language
- `apps/lite/src/NodeForm.tsx`: keep, UI code: keep in the original language
- `apps/lite/src/ShareDialog.tsx`: keep, UI code: keep in the original language
- `apps/lite/src/Commands.tsx`: keep, UI code: keep in the original language
- `apps/lite/src/Demo.tsx`: keep, UI code: keep in the original language
- `apps/lite/src/Dns.tsx`: keep, UI code: keep in the original language
- `apps/lite/src/App.tsx`: keep, UI code: keep in the original language
- `apps/lite/src/Shell.tsx`: keep, UI code: keep in the original language
- `apps/lite/src/main.tsx`: keep, UI code: keep in the original language
- `apps/lite/src/Routing.tsx`: keep, UI code: keep in the original language
- `apps/lite/src/app/`: keep, UI code: keep in the original language
- `apps/lite/src/ui/icons/`: keep, UI code: keep in the original language
- `src/fonts.ts`: blocked, Uses dynamic code (eval, new Function or computed imports)
- `apps/lite/src/ui/motion/`: keep, UI code: keep in the original language
- `apps/lite/src/Connect.tsx`: keep, UI code: keep in the original language
- `apps/lite/src/GroupEditPanel.tsx`: keep, UI code: keep in the original language
- `apps/lite/src/RuleList.tsx`: keep, UI code: keep in the original language
- `apps/lite/src/Subscriptions.tsx`: keep, UI code: keep in the original language
- `src/ui/icons/`: keep, UI code: keep in the original language
- `apps/lite/src/SettingsDialog.tsx`: keep, UI code: keep in the original language
- `src/features/activity/`: keep, UI code: keep in the original language
- `src/features/flows/`: keep, UI code: keep in the original language
- `src/features/policies/group/`: keep, UI code: keep in the original language
- `apps/lite/src/GroupInspector.tsx`: keep, UI code: keep in the original language
- `src/i18n/`: keep, UI code: keep in the original language
- `apps/lite/src/Groups.tsx`: keep, UI code: keep in the original language
- `src/features/connections/`: keep, UI code: keep in the original language
- `apps/lite/src/i18n/`: keep, UI code: keep in the original language
- `src/features/policies/`: keep, UI code: keep in the original language
- `apps/lite/src/Nodes.tsx`: keep, UI code: keep in the original language
- `src/features/dns/`: keep, UI code: keep in the original language
- `src/ui/code/`: keep, UI code: keep in the original language
- `src/ui/charts/`: keep, UI code: keep in the original language
- `tools/`: blocked, Uses dynamic code (eval, new Function or computed imports)
- `src/features/config/`: keep, UI code: keep in the original language
- `src/features/rules/`: keep, UI code: keep in the original language
- `src/store/`: keep, UI code: keep in the original language
- `apps/lite/src/ui/`: keep, UI code: keep in the original language
- `src/features/nodes/`: keep, UI code: keep in the original language
- `src/features/settings/`: keep, UI code: keep in the original language
- `src/shell/`: keep, UI code: keep in the original language
- `src/shell/widgets/`: keep, UI code: keep in the original language
- `src/ui/`: keep, UI code: keep in the original language

Regenerate with `rust-anything map --write`. Commit this file: when the original changes upstream, the next map marks every port whose recording it outdated.
