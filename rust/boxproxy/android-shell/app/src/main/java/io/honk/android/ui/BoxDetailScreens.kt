package io.honk.android.ui

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.ExperimentalLayoutApi
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Checkbox
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import org.json.JSONArray
import org.json.JSONObject

@Composable
private fun MiuixPillBtn(
    label: String,
    primary: Boolean = false,
    enabled: Boolean = true,
    onClick: () -> Unit,
) {
    val p = LocalBoxPalette.current
    Box(
        modifier = Modifier
            .clip(RoundedCornerShape(14.dp))
            .background(if (primary) p.accentSoft else p.card)
            .clickable(enabled = enabled, onClick = onClick)
            .padding(horizontal = 14.dp, vertical = 10.dp),
        contentAlignment = Alignment.Center,
    ) {
        Text(
            text = label,
            fontSize = 14.sp,
            fontWeight = if (primary) FontWeight.Medium else FontWeight.Normal,
            color = if (!enabled) p.textMuted else if (primary) p.accent else p.textPrimary,
            maxLines = 1,
        )
    }
}

@Composable
fun OverviewDetailScreen(state: JSONObject) {
    val p = LocalBoxPalette.current
    val mem = state.optJSONObject("overview_memory") ?: JSONObject()
    val dp = state.optJSONObject("overview_datapath") ?: JSONObject()
    val procMem = mem.optJSONObject("process") ?: JSONObject()
    val kernMem = mem.optJSONObject("kernel") ?: JSONObject()
    val cgroupMem = mem.optJSONObject("cgroup") ?: JSONObject()
    val rssBytes = procMem.optString("rss_bytes", "0").toLongOrNull() ?: procMem.optLong("rss_bytes", 0L)
    val ebpfBytes = kernMem.optString("ebpf_bytes", "0").toLongOrNull() ?: kernMem.optLong("bpf_map_bytes", 0L)
    val cgBytes = cgroupMem.optString("current_bytes", "0").toLongOrNull() ?: 0L

    Column(modifier = Modifier.fillMaxSize().background(p.background)) {
        MiuixSubPageTopBar(title = "运行概览", collapsed = false, onBack = { dispatchAsync("pop_back") })
        LazyColumn(contentPadding = PaddingValues(16.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
            item {
                Row(horizontalArrangement = Arrangement.spacedBy(10.dp)) {
                    MiuixPillBtn("刷新概览", primary = true) { dispatchAsync("refresh_overview") }
                    MiuixPillBtn("清空历史") { dispatchAsync("clear_clash_api_history") }
                }
            }
            item {
                MiuixCard(modifier = Modifier.fillMaxWidth()) {
                    Column(modifier = Modifier.padding(18.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
                        Text("内存与 eBPF 数据面 (M09)", fontSize = 16.sp, fontWeight = FontWeight.Medium, color = p.textPrimary)
                        Text("进程 RSS: ${formatBytesCompact(rssBytes)} · Cgroup: ${formatBytesCompact(cgBytes)}", fontSize = 14.sp, color = p.textSecondary)
                        Text("内核 eBPF 映射: ${formatBytesCompact(ebpfBytes)}", fontSize = 14.sp, color = p.textSecondary)
                        Text("数据路径: ${dp.strOr("kind")} · 状态: ${dp.strOr("state", dp.strOr("status"))}", fontSize = 14.sp, color = p.textSecondary)
                    }
                }
            }
        }
    }
}

@Composable
fun ProxySelectorDetailScreen(state: JSONObject) {
    val p = LocalBoxPalette.current
    Column(modifier = Modifier.fillMaxSize().background(p.background)) {
        MiuixSubPageTopBar(title = "策略选择", collapsed = false, onBack = { dispatchAsync("pop_back") })
        ProxySelectorInlineBody(state)
    }
}

@OptIn(ExperimentalLayoutApi::class)
@Composable
fun ProxySelectorInlineBody(state: JSONObject) {
    val p = LocalBoxPalette.current
    val mode = state.strOr("outbound_mode", "rule")
    val proxiesMap = state.optJSONObject("proxies_data")
    val honkGroups = state.optJSONArray("honk_groups_data")

    val selectorGroups = remember(proxiesMap?.toString()) {
        val list = mutableListOf<JSONObject>()
        if (proxiesMap != null) {
            val keys = proxiesMap.keys()
            while (keys.hasNext()) {
                val k = keys.next()
                val obj = proxiesMap.optJSONObject(k) ?: continue
                val all = obj.optJSONArray("all")
                if (all != null && all.length() > 0) {
                    list.add(JSONObject(obj.toString()).put("_key", k))
                }
            }
        }
        list
    }

    LazyColumn(
        contentPadding = PaddingValues(
            start = 16.dp,
            end = 16.dp,
            top = 12.dp,
            bottom = LocalLiquidDockBottomPadding.current,
        ),
        verticalArrangement = Arrangement.spacedBy(12.dp),
        modifier = Modifier.fillMaxSize(),
    ) {
        item {
            MiuixCard(modifier = Modifier.fillMaxWidth()) {
                Row(
                    modifier = Modifier.fillMaxWidth().padding(horizontal = 14.dp, vertical = 12.dp),
                    verticalAlignment = Alignment.CenterVertically,
                    horizontalArrangement = Arrangement.spacedBy(10.dp),
                ) {
                    val modes = listOf("rule", "direct", "global")
                    IosSegmentedControl(
                        items = listOf("规则", "直连", "全局"),
                        selectedIndex = modes.indexOfFirst { it.equals(mode, true) }.coerceAtLeast(0),
                        onSelect = { idx -> dispatchAsync("switch_outbound_mode") { put("mode", modes[idx]) } },
                        modifier = Modifier.weight(1f),
                        height = 40.dp,
                        trackColor = p.background,
                    )
                    MiuixPillBtn("刷新", primary = true) { dispatchAsync("refresh_proxies") }
                }
            }
        }

        if (honkGroups != null && honkGroups.length() > 0) {
            items(honkGroups.length()) { idx ->
                val g = honkGroups.optJSONObject(idx) ?: JSONObject()
                val gId = g.strOr("id", "")
                val gName = g.strOr("name", gId)
                val count = g.optInt("member_count", 0)
                val policyKind = g.optJSONObject("policy")?.strOr("kind", "") ?: ""
                val members = g.optJSONArray("members")
                val activeMemberId = g.optJSONObject("runtime")
                    ?.optJSONObject("selection")
                    ?.optJSONObject("tcp")
                    ?.strOr("member_id", "")
                    ?.ifEmpty { null }
                    ?: g.optJSONObject("selection")?.strOr("tcp_member_id", "").orEmpty()
                val canSelect = g.optJSONObject("capabilities")?.let {
                    it.optBoolean("can_select", true) || it.optBoolean("can_override", false)
                } ?: true

                MiuixCard(modifier = Modifier.fillMaxWidth()) {
                    Column(
                        modifier = Modifier.fillMaxWidth().padding(16.dp),
                        verticalArrangement = Arrangement.spacedBy(10.dp),
                    ) {
                        Row(
                            modifier = Modifier.fillMaxWidth(),
                            verticalAlignment = Alignment.CenterVertically,
                        ) {
                            Column(modifier = Modifier.weight(1f)) {
                                Text(gName, fontSize = 16.sp, fontWeight = FontWeight.SemiBold, color = p.textPrimary)
                                Text(
                                    "策略: ${policyKind.ifEmpty { "selector" }} · $count 个成员",
                                    fontSize = 12.sp,
                                    color = p.textSecondary,
                                )
                            }
                            Row(horizontalArrangement = Arrangement.spacedBy(6.dp)) {
                                if (gId.isNotEmpty()) {
                                    MiuixPillBtn("测速") {
                                        dispatchAsync("test_group_delay") { put("group", gId) }
                                        dispatchAsync("refresh_proxies")
                                    }
                                }
                                MiuixPillBtn("WebUI", primary = true) { navigateTo("web_ui") }
                            }
                        }
                        if (members != null && members.length() > 0) {
                            FlowRow(
                                horizontalArrangement = Arrangement.spacedBy(6.dp),
                                verticalArrangement = Arrangement.spacedBy(6.dp),
                            ) {
                                for (mIdx in 0 until members.length().coerceAtMost(48)) {
                                    val m = members.optJSONObject(mIdx) ?: continue
                                    val mId = m.strOr("id", "")
                                    val mName = m.strOr("name", mId)
                                    val active = mId.isNotEmpty() && mId == activeMemberId
                                    Box(
                                        modifier = Modifier
                                            .clip(RoundedCornerShape(10.dp))
                                            .background(if (active) p.accentSoft else p.background)
                                            .clickable(enabled = canSelect && gId.isNotEmpty() && mId.isNotEmpty()) {
                                                dispatchAsync("select_proxy") {
                                                    put("group", gId)
                                                    put("node", mId)
                                                }
                                            }
                                            .padding(horizontal = 10.dp, vertical = 6.dp),
                                    ) {
                                        Text(
                                            text = mName,
                                            fontSize = 13.sp,
                                            fontWeight = if (active) FontWeight.SemiBold else FontWeight.Normal,
                                            color = if (active) p.accent else p.textPrimary,
                                            maxLines = 1,
                                            overflow = TextOverflow.Ellipsis,
                                        )
                                    }
                                }
                            }
                        }
                    }
                }
            }
        } else if (selectorGroups.isNotEmpty()) {
            items(selectorGroups.size) { idx ->
                val g = selectorGroups[idx]
                val gName = g.strOr("_key", g.strOr("name"))
                val now = g.strOr("now", "")
                val all = g.optJSONArray("all") ?: JSONArray()
                MiuixCard(modifier = Modifier.fillMaxWidth()) {
                    Column(modifier = Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(10.dp)) {
                        Row(modifier = Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
                            Column(modifier = Modifier.weight(1f)) {
                                Text(gName, fontSize = 16.sp, fontWeight = FontWeight.Medium, color = p.textPrimary)
                                Text("当前: ${now.ifEmpty { "-" }} · ${all.length()} 节点", fontSize = 12.sp, color = p.textSecondary)
                            }
                            MiuixPillBtn("测速") { dispatchAsync("test_group_delay") { put("group", gName) } }
                        }
                        FlowRow(horizontalArrangement = Arrangement.spacedBy(6.dp), verticalArrangement = Arrangement.spacedBy(6.dp)) {
                            for (mIdx in 0 until all.length().coerceAtMost(48)) {
                                val node = all.optString(mIdx)
                                val active = node == now
                                Box(
                                    modifier = Modifier
                                        .clip(RoundedCornerShape(10.dp))
                                        .background(if (active) p.accentSoft else p.background)
                                        .clickable {
                                            dispatchAsync("select_proxy") {
                                                put("group", gName)
                                                put("node", node)
                                            }
                                        }
                                        .padding(horizontal = 10.dp, vertical = 6.dp),
                                ) {
                                    Text(node, fontSize = 13.sp, color = if (active) p.accent else p.textPrimary, maxLines = 1, overflow = TextOverflow.Ellipsis)
                                }
                            }
                        }
                    }
                }
            }
        } else {
            item {
                MiuixCard(modifier = Modifier.fillMaxWidth()) {
                    Text(
                        "点击上方“刷新”拉取策略组列表",
                        fontSize = 14.sp,
                        color = p.textSecondary,
                        modifier = Modifier.padding(20.dp),
                    )
                }
            }
        }
    }
}

@Composable
fun ConnectionsDetailScreen(state: JSONObject) {
    val p = LocalBoxPalette.current
    Column(modifier = Modifier.fillMaxSize().background(p.background)) {
        MiuixSubPageTopBar(title = "实时连接", collapsed = false, onBack = { dispatchAsync("pop_back") })
        ConnectionsInlineBody(state)
    }
}

@Composable
fun ConnectionsInlineBody(state: JSONObject) {
    val p = LocalBoxPalette.current
    val snap = state.optJSONObject("connections_snapshot") ?: JSONObject()
    val conns = snap.optJSONArray("connections")
    val paused = state.optBoolean("connections_paused", false)
    var showCloseAllConfirm by remember { mutableStateOf(false) }

    LazyColumn(
        contentPadding = PaddingValues(
            start = 16.dp,
            end = 16.dp,
            top = 12.dp,
            bottom = LocalLiquidDockBottomPadding.current,
        ),
        verticalArrangement = Arrangement.spacedBy(10.dp),
        modifier = Modifier.fillMaxSize(),
    ) {
        item {
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                MiuixPillBtn("刷新", primary = true) { dispatchAsync("refresh_connections") }
                MiuixPillBtn(if (paused) "恢复" else "暂停") { dispatchAsync("set_connections_paused") { put("paused", !paused) } }
                MiuixPillBtn("全部断开", enabled = conns != null && conns.length() > 0) {
                    showCloseAllConfirm = true
                }
            }
        }
        if (conns != null && conns.length() > 0) {
            items(conns.length().coerceAtMost(60)) { idx ->
                val c = conns.optJSONObject(idx) ?: JSONObject()
                val meta = c.optJSONObject("metadata") ?: JSONObject()
                val host = meta.optString("host").ifEmpty { meta.optString("destinationIP", "-") }
                MiuixCard(modifier = Modifier.fillMaxWidth()) {
                    Row(modifier = Modifier.fillMaxWidth().padding(14.dp), verticalAlignment = Alignment.CenterVertically) {
                        Column(modifier = Modifier.weight(1f)) {
                            Text("$host:${meta.optString("destinationPort", "")}", fontSize = 15.sp, color = p.textPrimary, maxLines = 1, overflow = TextOverflow.Ellipsis)
                            Text("${c.strOr("rule")} · ${meta.strOr("network")}", fontSize = 12.sp, color = p.textSecondary)
                        }
                        MiuixPillBtn("断开") { dispatchAsync("close_connection") { put("id", c.optString("id")) } }
                    }
                }
            }
        } else {
            item {
                MiuixCard(modifier = Modifier.fillMaxWidth()) {
                    Text(
                        "当前活跃连接: ${state.optInt("active_connections_count", 0)}（点击“刷新”加载详情）",
                        fontSize = 14.sp,
                        color = p.textSecondary,
                        modifier = Modifier.padding(18.dp),
                    )
                }
            }
        }
    }

    if (showCloseAllConfirm) {
        MiuixConfirmDialog(
            title = "断开全部连接？",
            message = "将立即关闭当前所有活跃的 TCP 与 UDP 会话。",
            confirmLabel = "全部断开",
            destructive = true,
            onConfirm = { dispatchAsync("close_all_connections") },
            onDismiss = { showCloseAllConfirm = false },
        )
    }
}

@Composable
fun SubscriptionsDetailScreen(state: JSONObject) {
    val p = LocalBoxPalette.current
    Column(modifier = Modifier.fillMaxSize().background(p.background)) {
        MiuixSubPageTopBar(title = "订阅详情", collapsed = false, onBack = { dispatchAsync("pop_back") })
        SubscriptionsInlineBody(state)
    }
}

@Composable
fun SubscriptionsInlineBody(state: JSONObject) {
    val p = LocalBoxPalette.current
    val dataObj = state.optJSONObject("proxy_providers_data")
    val parsedCards = remember(dataObj?.toString()) {
        val out = mutableListOf<JSONObject>()
        if (dataObj != null) {
            val honkArr = dataObj.optJSONArray("providers")
            val clashMap = dataObj.optJSONObject("providers")
            if (honkArr != null) {
                for (i in 0 until honkArr.length()) {
                    val item = honkArr.optJSONObject(i) ?: continue
                    val traffic = item.optJSONObject("traffic")
                    val upBytes = traffic?.strOr("upload_bytes", "")?.toLongOrNull() ?: 0L
                    val downBytes = traffic?.strOr("download_bytes", "")?.toLongOrNull() ?: 0L
                    val totalBytes = traffic?.strOr("total_bytes", "")?.toLongOrNull() ?: 0L
                    out.add(
                        JSONObject()
                            .put("id", item.strOr("id", ""))
                            .put("name", item.strOr("name", item.strOr("id")))
                            .put("kind", item.strOr("kind", "subscription"))
                            .put("count", item.optInt("node_count", 0))
                            .put("updated", item.strOr("updated_at", item.strOr("last_refresh_at", "")))
                            .put("used", upBytes + downBytes)
                            .put("total", totalBytes),
                    )
                }
            } else if (clashMap != null) {
                val keys = clashMap.keys()
                while (keys.hasNext()) {
                    val k = keys.next()
                    val item = clashMap.optJSONObject(k) ?: continue
                    val vType = item.optString("vehicleType", "")
                    if (vType.equals("Compatible", true)) continue
                    val subInfo = item.optJSONObject("subscriptionInfo")
                    val used = (subInfo?.optLong("Upload", 0L) ?: 0L) + (subInfo?.optLong("Download", 0L) ?: 0L)
                    val total = subInfo?.optLong("Total", 0L) ?: 0L
                    out.add(
                        JSONObject()
                            .put("id", k)
                            .put("name", k)
                            .put("kind", vType.ifEmpty { "HTTP" })
                            .put("count", item.optJSONArray("proxies")?.length() ?: 0)
                            .put("updated", item.optString("updatedAt", ""))
                            .put("used", used)
                            .put("total", total),
                    )
                }
            }
        }
        out
    }

    LazyColumn(
        contentPadding = PaddingValues(
            start = 16.dp,
            end = 16.dp,
            top = 12.dp,
            bottom = LocalLiquidDockBottomPadding.current,
        ),
        verticalArrangement = Arrangement.spacedBy(12.dp),
        modifier = Modifier.fillMaxSize(),
    ) {
        item {
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                MiuixPillBtn("刷新订阅配额", primary = true) { dispatchAsync("refresh_subscriptions") }
                MiuixPillBtn("在 WebUI 中添加订阅") { navigateTo("web_ui") }
            }
        }
        if (parsedCards.isEmpty()) {
            item {
                MiuixCard(modifier = Modifier.fillMaxWidth()) {
                    Column(
                        modifier = Modifier.padding(20.dp),
                        verticalArrangement = Arrangement.spacedBy(6.dp),
                    ) {
                        Text("暂无外部订阅源", fontSize = 16.sp, fontWeight = FontWeight.Medium, color = p.textPrimary)
                        Text(
                            "当前配置使用内联节点（Inline Nodes）或尚未拉取订阅。可点击“刷新订阅配额”或前往 WebUI 添加远程订阅。",
                            fontSize = 13.sp,
                            color = p.textSecondary,
                        )
                    }
                }
            }
        } else {
            items(parsedCards.size) { idx ->
                val item = parsedCards[idx]
                val rawPid = item.strOr("id")
                val rawName = item.strOr("name")
                val rawKind = item.strOr("kind")
                val isInline = rawKind.equals("inline", ignoreCase = true) || rawPid.equals("inline", ignoreCase = true)
                val pid = if (isInline) "" else rawPid
                val name = if (isInline && rawName.equals("inline", ignoreCase = true)) "内置节点 (Inline)" else rawName
                val kind = if (isInline) "本地配置" else rawKind
                val count = item.optInt("count", 0)
                val used = item.optLong("used", 0L)
                val total = item.optLong("total", 0L)
                val updated = item.strOr("updated", "")
                MiuixCard(modifier = Modifier.fillMaxWidth()) {
                    Column(modifier = Modifier.padding(18.dp), verticalArrangement = Arrangement.spacedBy(10.dp)) {
                        Row(modifier = Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically) {
                            Column(modifier = Modifier.weight(1f)) {
                                Text(name, fontSize = 16.sp, fontWeight = FontWeight.SemiBold, color = p.textPrimary)
                                Text(
                                    "$kind · $count 个节点" + if (updated.isNotEmpty()) " · $updated" else "",
                                    fontSize = 12.sp,
                                    color = p.textSecondary,
                                    maxLines = 1,
                                    overflow = TextOverflow.Ellipsis,
                                )
                            }
                            if (pid.isNotEmpty()) {
                                MiuixPillBtn("更新", primary = true) {
                                    dispatchAsync("update_proxy_provider") { put("name", pid) }
                                    dispatchAsync("refresh_subscriptions")
                                }
                            }
                        }
                        if (total > 0L) {
                            val frac = (used.toDouble() / total.toDouble()).toFloat().coerceIn(0.02f, 1f)
                            Column(verticalArrangement = Arrangement.spacedBy(4.dp)) {
                                Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
                                    Text("已用 ${formatBytesCompact(used)}", fontSize = 12.sp, color = p.textSecondary)
                                    Text("总计 ${formatBytesCompact(total)}", fontSize = 12.sp, color = p.textSecondary)
                                }
                                Box(
                                    modifier = Modifier
                                        .fillMaxWidth()
                                        .height(6.dp)
                                        .clip(RoundedCornerShape(50))
                                        .background(p.accentSoft),
                                ) {
                                    Box(
                                        modifier = Modifier
                                            .fillMaxWidth(frac)
                                            .height(6.dp)
                                            .clip(RoundedCornerShape(50))
                                            .background(p.accent),
                                    )
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

@Composable
fun RulesDetailScreen(state: JSONObject) {
    val p = LocalBoxPalette.current
    var domain by remember { mutableStateOf("www.google.com") }
    var ruleQuery by remember { mutableStateOf("") }
    val traceObj = state.optJSONObject("route_trace_result")
    val rulesObj = state.optJSONObject("rules_data")
    val rulesArr = rulesObj?.optJSONArray("rules") ?: state.optJSONArray("rules_data")

    val filteredRules = remember(rulesArr?.toString(), ruleQuery) {
        val out = mutableListOf<JSONObject>()
        if (rulesArr != null) {
            for (i in 0 until rulesArr.length()) {
                val r = rulesArr.optJSONObject(i) ?: continue
                if (ruleQuery.isBlank() || r.toString().contains(ruleQuery, ignoreCase = true)) {
                    out.add(r)
                }
            }
        }
        out
    }

    Column(modifier = Modifier.fillMaxSize().background(p.background)) {
        MiuixSubPageTopBar(title = "分流规则与追踪", collapsed = false, onBack = { dispatchAsync("pop_back") })
        LazyColumn(contentPadding = PaddingValues(16.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
            item {
                MiuixCard(modifier = Modifier.fillMaxWidth()) {
                    Column(modifier = Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(10.dp)) {
                        OutlinedTextField(
                            value = domain,
                            onValueChange = { domain = it },
                            label = { Text("路由追踪目标域名") },
                            singleLine = true,
                            shape = RoundedCornerShape(14.dp),
                            modifier = Modifier.fillMaxWidth(),
                        )
                        Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                            MiuixPillBtn("追踪路由", primary = true) {
                                dispatchAsync("trace_route") {
                                    put("request", JSONObject().put("domain", domain.trim()).put("dst_port", 443).put("protocol", "tcp"))
                                }
                            }
                            MiuixPillBtn("刷新规则") { dispatchAsync("refresh_rules") }
                        }
                    }
                }
            }
            if (traceObj != null) {
                item {
                    val eval0 = traceObj.optJSONArray("evaluations")?.optJSONObject(0)
                    val dec = eval0 ?: traceObj.optJSONObject("decision") ?: traceObj
                    val outObj = dec.optJSONObject("outbound")
                    val outbound = outObj?.strOr("name", "")?.ifEmpty { null }
                        ?: dec.strOr("outbound", "").ifEmpty { traceObj.strOr("outbound", "-") }
                    var matchedExpr: String? = null
                    val evalRules = eval0?.optJSONArray("rules")
                    if (evalRules != null) {
                        for (ri in 0 until evalRules.length()) {
                            val ro = evalRules.optJSONObject(ri) ?: continue
                            val resStr = ro.strOr("result", ro.strOr("outcome", ""))
                            if (ro.optBoolean("matched", false) || resStr == "matched" || resStr == "match") {
                                matchedExpr = ro.strOr("expression", "")
                                break
                            }
                        }
                    }
                    val ruleObj = traceObj.optJSONObject("matched_rule") ?: dec.optJSONObject("matched_rule")
                    val matchedRule = matchedExpr?.ifEmpty { null }
                        ?: ruleObj?.strOr("expression", "")?.ifEmpty { null }
                        ?: traceObj.strOr("matched_rule", "fallback")
                    val selNode = dec.optJSONObject("selected_node")?.strOr("name", "-")
                        ?: dec.strOr("selected_node", dec.strOr("dst_ip", "-"))
                    MiuixCard(modifier = Modifier.fillMaxWidth(), backgroundColor = p.accentSoft) {
                        Column(modifier = Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(6.dp)) {
                            Text("出站决策: $outbound", fontSize = 16.sp, fontWeight = FontWeight.SemiBold, color = p.accent)
                            Text("命中规则: $matchedRule", fontSize = 13.sp, fontFamily = FontFamily.Monospace, color = p.textPrimary)
                            if (selNode != "-" && selNode.isNotEmpty()) {
                                Text("选定节点: $selNode", fontSize = 13.sp, color = p.textSecondary)
                            }
                        }
                    }
                }
            }
            item {
                OutlinedTextField(
                    value = ruleQuery,
                    onValueChange = { ruleQuery = it },
                    placeholder = { Text("搜索分流规则 (${filteredRules.size})", color = p.textMuted) },
                    singleLine = true,
                    shape = RoundedCornerShape(14.dp),
                    modifier = Modifier.fillMaxWidth(),
                )
            }
            if (filteredRules.isNotEmpty()) {
                items(filteredRules.size.coerceAtMost(120)) { idx ->
                    val r = filteredRules[idx]
                    MiuixCard(modifier = Modifier.fillMaxWidth()) {
                        Row(
                            modifier = Modifier.fillMaxWidth().padding(horizontal = 16.dp, vertical = 14.dp),
                            horizontalArrangement = Arrangement.SpaceBetween,
                            verticalAlignment = Alignment.CenterVertically,
                        ) {
                            Text(
                                text = r.strOr("expression", r.strOr("payload")),
                                fontSize = 13.sp,
                                fontFamily = FontFamily.Monospace,
                                color = p.textPrimary,
                                modifier = Modifier.weight(1f).padding(end = 10.dp),
                                maxLines = 2,
                                overflow = TextOverflow.Ellipsis,
                            )
                            Text(
                                text = r.strOr("outbound", r.strOr("proxy")),
                                fontSize = 13.sp,
                                color = p.accent,
                                fontWeight = FontWeight.SemiBold,
                            )
                        }
                    }
                }
            }
        }
    }
}

@Composable
fun RuleProvidersDetailScreen(state: JSONObject) {
    val p = LocalBoxPalette.current
    val dataObj = state.optJSONObject("rule_providers_data")
    val providers = remember(dataObj?.toString()) {
        val list = mutableListOf<JSONObject>()
        val map = dataObj?.optJSONObject("providers")
        if (map != null) {
            val keys = map.keys()
            while (keys.hasNext()) {
                val k = keys.next()
                val obj = map.optJSONObject(k) ?: continue
                list.add(JSONObject(obj.toString()).put("_name", k))
            }
        }
        list
    }
    Column(modifier = Modifier.fillMaxSize().background(p.background)) {
        MiuixSubPageTopBar(title = "规则集", collapsed = false, onBack = { dispatchAsync("pop_back") })
        LazyColumn(contentPadding = PaddingValues(16.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
            item {
                Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    MiuixPillBtn("刷新规则集", primary = true) { dispatchAsync("refresh_rule_providers") }
                    if (providers.isNotEmpty()) {
                        MiuixPillBtn("全部更新") { dispatchAsync("update_all_rule_providers") }
                    }
                    MiuixPillBtn("GeoData 管理") { navigateTo("web_ui") }
                }
            }
            if (providers.isEmpty()) {
                item {
                    MiuixCard(modifier = Modifier.fillMaxWidth()) {
                        Column(modifier = Modifier.padding(18.dp), verticalArrangement = Arrangement.spacedBy(6.dp)) {
                            Text("未配置 Clash 外部规则集", fontSize = 16.sp, fontWeight = FontWeight.Medium, color = p.textPrimary)
                            Text(
                                "Honk 核心直接通过高性能 eBPF 与 GeoSite / GeoIP 编译匹配分流规则，可在 WebUI -> 设置 -> GeoData 中一键更新规则库。",
                                fontSize = 13.sp,
                                color = p.textSecondary,
                            )
                        }
                    }
                }
            } else {
                items(providers.size) { idx ->
                    val rp = providers[idx]
                    val name = rp.strOr("_name", rp.strOr("name"))
                    val behavior = rp.strOr("behavior", "classical")
                    val cnt = rp.optInt("ruleCount", 0)
                    MiuixCard(modifier = Modifier.fillMaxWidth()) {
                        Row(
                            modifier = Modifier.fillMaxWidth().padding(16.dp),
                            verticalAlignment = Alignment.CenterVertically,
                        ) {
                            Column(modifier = Modifier.weight(1f)) {
                                Text(name, fontSize = 15.sp, fontWeight = FontWeight.SemiBold, color = p.textPrimary)
                                Text("$behavior · $cnt 条规则", fontSize = 12.sp, color = p.textSecondary)
                            }
                            MiuixPillBtn("更新") {
                                dispatchAsync("update_rule_provider") { put("name", name) }
                                dispatchAsync("refresh_rule_providers")
                            }
                        }
                    }
                }
            }
        }
    }
}

@OptIn(ExperimentalLayoutApi::class)
@Composable
fun AppManagementScreen(state: JSONObject) {
    val p = LocalBoxPalette.current
    val profile = state.optJSONObject("runtime_profile") ?: JSONObject()
    val proxyMode = profile.strOr("proxy_mode", "core")
    val apps = state.optJSONArray("installed_apps") ?: JSONArray()
    val selectedArr = state.optJSONArray("selected_uids") ?: JSONArray()
    val selectedSet = remember(selectedArr.toString()) {
        buildSet { for (i in 0 until selectedArr.length()) add(selectedArr.optInt(i)) }
    }
    var query by remember { mutableStateOf("") }

    Column(modifier = Modifier.fillMaxSize().background(p.background)) {
        MiuixSubPageTopBar(title = "应用管理", collapsed = false, onBack = { dispatchAsync("pop_back") })
        LazyColumn(contentPadding = PaddingValues(16.dp), verticalArrangement = Arrangement.spacedBy(10.dp)) {
            item {
                MiuixCard(modifier = Modifier.fillMaxWidth()) {
                    Column(modifier = Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(10.dp)) {
                        val appModes = listOf("core", "blacklist", "whitelist")
                        IosSegmentedControl(
                            items = listOf("核心接管", "黑名单", "白名单"),
                            selectedIndex = appModes.indexOf(proxyMode).coerceAtLeast(0),
                            onSelect = { idx ->
                                dispatchAsync("save_app_selection") {
                                    put("uids", selectedArr)
                                    put("gids", state.optJSONArray("gid_list") ?: JSONArray())
                                    put("proxy_mode", appModes[idx])
                                }
                            },
                            modifier = Modifier.fillMaxWidth(),
                            height = 40.dp,
                            trackColor = p.background,
                        )
                        FlowRow(horizontalArrangement = Arrangement.spacedBy(8.dp), verticalArrangement = Arrangement.spacedBy(6.dp)) {
                            MiuixPillBtn("智能选国内常用") { dispatchAsync("run_smart_select") { put("mode", "cn_common_apps") } }
                            MiuixPillBtn("选系统应用") { dispatchAsync("run_smart_select") { put("mode", "system_apps") } }
                            MiuixPillBtn("刷新列表", primary = true) { dispatchAsync("refresh_installed_apps") }
                        }
                        OutlinedTextField(
                            value = query,
                            onValueChange = { query = it },
                            label = { Text("搜索应用 (${apps.length()})") },
                            singleLine = true,
                            shape = RoundedCornerShape(14.dp),
                            modifier = Modifier.fillMaxWidth(),
                        )
                    }
                }
            }
            val filtered = (0 until apps.length()).mapNotNull { apps.optJSONObject(it) }.filter {
                query.isBlank() || it.optString("label").contains(query, true) || it.optString("package_name").contains(query, true)
            }.take(60)
            items(filtered.size) { idx ->
                val app = filtered[idx]
                val uid = app.optInt("uid")
                val checked = uid in selectedSet
                MiuixCard(modifier = Modifier.fillMaxWidth()) {
                    Row(
                        modifier = Modifier.fillMaxWidth().padding(horizontal = 14.dp, vertical = 10.dp),
                        verticalAlignment = Alignment.CenterVertically,
                    ) {
                        Checkbox(checked = checked, onCheckedChange = { nowChecked ->
                            val next = selectedSet.toMutableSet().apply { if (nowChecked) add(uid) else remove(uid) }
                            dispatchAsync("save_app_selection") {
                                put("uids", JSONArray(next.toList()))
                                put("gids", state.optJSONArray("gid_list") ?: JSONArray())
                                put("proxy_mode", proxyMode)
                            }
                        })
                        Column {
                            Text(app.strOr("label"), fontSize = 15.sp, color = p.textPrimary)
                            Text("${app.strOr("package_name")} (uid=$uid)", fontSize = 12.sp, color = p.textSecondary)
                        }
                    }
                }
            }
        }
    }
}

@Composable
fun CnipSettingsScreen(state: JSONObject) {
    val p = LocalBoxPalette.current
    val cnip = state.optJSONObject("cnip_settings") ?: JSONObject()
    val enabled = cnip.optBoolean("bypass_cnip", false)
    val mode = cnip.strOr("cnip_mode", "ipset")
    val v4 = cnip.optBoolean("bypass_ipv4", true)
    val v6 = cnip.optBoolean("bypass_ipv6", false)
    var showModePicker by remember { mutableStateOf(false) }

    Column(modifier = Modifier.fillMaxSize().background(p.background)) {
        MiuixSubPageTopBar(title = "CNIP 设置", collapsed = false, onBack = { dispatchAsync("pop_back") })
        Column(modifier = Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(14.dp)) {
            MiuixCard(modifier = Modifier.fillMaxWidth()) {
                MiuixSettingSwitchRow("绕过中国大陆 IP", enabled) {
                    dispatchAsync("save_cnip_settings") { put("settings", JSONObject(cnip.toString()).put("bypass_cnip", it)) }
                }
                MiuixSettingTextRow("CNIP 模式", trailingText = mode.uppercase(), isDropdown = true) { showModePicker = true }
                MiuixSettingSwitchRow("绕过 IPv4", v4) {
                    dispatchAsync("save_cnip_settings") { put("settings", JSONObject(cnip.toString()).put("bypass_ipv4", it)) }
                }
                MiuixSettingSwitchRow("绕过 IPv6", v6) {
                    dispatchAsync("save_cnip_settings") { put("settings", JSONObject(cnip.toString()).put("bypass_ipv6", it)) }
                }
            }
        }
    }
    if (showModePicker) {
        MiuixOptionSheetDialog(
            title = "CNIP 模式",
            options = listOf("ipset" to "IPSet", "ebpf" to "eBPF"),
            selectedValue = mode,
            onSelect = { m -> dispatchAsync("save_cnip_settings") { put("settings", JSONObject(cnip.toString()).put("cnip_mode", m)) } },
            onDismiss = { showModePicker = false },
        )
    }
}

@Composable
fun BypassRulesScreen(state: JSONObject) {
    val p = LocalBoxPalette.current
    val v4 = state.optJSONArray("intranet_ipv4") ?: JSONArray()
    val v6 = state.optJSONArray("intranet_ipv6") ?: JSONArray()
    Column(modifier = Modifier.fillMaxSize().background(p.background)) {
        MiuixSubPageTopBar(title = "绕过规则", collapsed = false, onBack = { dispatchAsync("pop_back") })
        LazyColumn(contentPadding = PaddingValues(16.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
            item { MiuixPillBtn("重置默认内网网段 (16 IPv4 / 12 IPv6)", primary = true) { dispatchAsync("reset_default_intranet_cidrs") } }
            item {
                MiuixCard(modifier = Modifier.fillMaxWidth()) {
                    Column(modifier = Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(6.dp)) {
                        Text("IPv4 内网绕过 (${v4.length()} 条)", fontSize = 16.sp, fontWeight = FontWeight.Medium, color = p.textPrimary)
                        Text((0 until v4.length()).map { v4.optString(it) }.joinToString(", "), fontSize = 12.sp, fontFamily = FontFamily.Monospace, color = p.textSecondary)
                    }
                }
            }
            item {
                MiuixCard(modifier = Modifier.fillMaxWidth()) {
                    Column(modifier = Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(6.dp)) {
                        Text("IPv6 内网绕过 (${v6.length()} 条)", fontSize = 16.sp, fontWeight = FontWeight.Medium, color = p.textPrimary)
                        Text((0 until v6.length()).map { v6.optString(it) }.joinToString(", "), fontSize = 12.sp, fontFamily = FontFamily.Monospace, color = p.textSecondary)
                    }
                }
            }
        }
    }
}

@Composable
fun SharedNetworkScreen(state: JSONObject) {
    val p = LocalBoxPalette.current
    val arp = state.optJSONArray("arp_clients") ?: JSONArray()
    val ifaces = state.optJSONArray("shared_network_interfaces") ?: JSONArray()
    Column(modifier = Modifier.fillMaxSize().background(p.background)) {
        MiuixSubPageTopBar(title = "共享网络", collapsed = false, onBack = { dispatchAsync("pop_back") })
        LazyColumn(contentPadding = PaddingValues(16.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
            item {
                MiuixPillBtn("扫描热点客户端 (ARP)", primary = true) { dispatchAsync("scan_arp_clients") }
                Text("共享网卡: ${(0 until ifaces.length()).map { ifaces.optString(it) }.joinToString(", ").ifEmpty { "默认热点网卡" }}", fontSize = 13.sp, color = p.textSecondary, modifier = Modifier.padding(top = 8.dp))
            }
            items(arp.length()) { idx ->
                val c = arp.optJSONObject(idx) ?: JSONObject()
                MiuixCard(modifier = Modifier.fillMaxWidth()) {
                    Text("${c.strOr("ip")} · ${c.strOr("mac")} · ${c.strOr("device")}", modifier = Modifier.padding(16.dp), fontSize = 14.sp, color = p.textPrimary)
                }
            }
        }
    }
}

@Composable
fun NetworkMatchScreen(state: JSONObject) {
    val p = LocalBoxPalette.current
    val nm = state.optJSONObject("network_match_settings") ?: JSONObject()
    val enabled = nm.optBoolean("network_control_enabled", false)
    val groups = state.optJSONArray("network_rule_groups") ?: JSONArray()
    Column(modifier = Modifier.fillMaxSize().background(p.background)) {
        MiuixSubPageTopBar(title = "网络匹配", collapsed = false, onBack = { dispatchAsync("pop_back") })
        LazyColumn(contentPadding = PaddingValues(16.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
            item {
                MiuixCard(modifier = Modifier.fillMaxWidth()) {
                    MiuixSettingSwitchRow("启用网络匹配自动控制", enabled) { v ->
                        dispatchAsync("save_network_match_settings") {
                            put("settings", JSONObject(nm.toString()).put("network_control_enabled", v))
                            put("ssids", state.optJSONArray("network_match_ssids") ?: JSONArray())
                            put("bssids", state.optJSONArray("network_match_bssids") ?: JSONArray())
                            put("conditions", state.optJSONArray("network_match_conditions") ?: JSONArray())
                        }
                    }
                    MiuixSettingTextRow("立即评估并应用", trailingText = "${groups.length()} 规则组") {
                        dispatchAsync("apply_network_match")
                    }
                }
            }
        }
    }
}

@Composable
fun ScriptsHubScreen(state: JSONObject) {
    val p = LocalBoxPalette.current
    val shells = state.optJSONArray("shell_scripts") ?: JSONArray()
    val js = state.optJSONArray("js_scripts") ?: JSONArray()
    val lastHook = state.optJSONObject("last_hook_result")
    Column(modifier = Modifier.fillMaxSize().background(p.background)) {
        MiuixSubPageTopBar(title = "脚本与订阅覆写", collapsed = false, onBack = { dispatchAsync("pop_back") })
        LazyColumn(contentPadding = PaddingValues(16.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
            item {
                Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                    MiuixPillBtn("刷新 Shell", primary = true) { dispatchAsync("list_shell_scripts") }
                    MiuixPillBtn("刷新 JS (${js.length()})") { dispatchAsync("list_js_scripts") { put("core", "mihomo") } }
                }
            }
            if (lastHook != null) {
                item {
                    MiuixCard(modifier = Modifier.fillMaxWidth()) {
                        Text("退出码: ${lastHook.optInt("exit_code")} · ${lastHook.optString("stdout").take(300)}", modifier = Modifier.padding(16.dp), fontFamily = FontFamily.Monospace, fontSize = 12.sp, color = p.textSecondary)
                    }
                }
            }
            items(shells.length()) { idx ->
                val s = shells.optJSONObject(idx) ?: JSONObject()
                val name = s.strOr("name")
                MiuixCard(modifier = Modifier.fillMaxWidth()) {
                    Row(modifier = Modifier.fillMaxWidth().padding(16.dp), verticalAlignment = Alignment.CenterVertically) {
                        Text(name, fontSize = 16.sp, color = p.textPrimary, modifier = Modifier.weight(1f))
                        MiuixPillBtn("执行", primary = true) { dispatchAsync("run_shell_script") { put("name", name); put("event", "manual") } }
                    }
                }
            }
        }
    }
}

@Composable
fun RuntimeFilesScreen(state: JSONObject) {
    val p = LocalBoxPalette.current
    val dir = state.optString("runtime_files_dir", "")
    val files = state.optJSONArray("runtime_files") ?: JSONArray()
    Column(modifier = Modifier.fillMaxSize().background(p.background)) {
        MiuixSubPageTopBar(
            title = "文件管理",
            collapsed = false,
            onBack = { dispatchAsync("pop_back") },
            trailing = if (dir.isNotEmpty()) {
                {
                    MiuixPillBtn("上级") {
                        dispatchAsync("list_runtime_files") { put("dir", dir.substringBeforeLast('/', "")) }
                    }
                }
            } else null,
        )
        LazyColumn(contentPadding = PaddingValues(16.dp), verticalArrangement = Arrangement.spacedBy(10.dp)) {
            item { Text("/box/$dir", fontSize = 13.sp, color = p.textSecondary, modifier = Modifier.padding(horizontal = 4.dp)) }
            items(files.length()) { idx ->
                val f = files.optJSONObject(idx) ?: JSONObject()
                val isDir = f.optBoolean("is_dir", false)
                val rel = f.optString("relative_path", "")
                MiuixCard(
                    modifier = Modifier.fillMaxWidth(),
                    onClick = {
                        if (isDir) dispatchAsync("list_runtime_files") { put("dir", rel) }
                        else dispatchAsync("open_editor_file") { put("relative_path", rel) }
                    },
                ) {
                    Row(modifier = Modifier.fillMaxWidth().padding(16.dp), verticalAlignment = Alignment.CenterVertically) {
                        Text(if (isDir) "📁  ${f.strOr("name")}" else "📄  ${f.strOr("name")}", fontSize = 15.sp, color = p.textPrimary, modifier = Modifier.weight(1f))
                        Text(if (isDir) "目录" else formatBytesCompact(f.optLong("size_bytes", 0L)), fontSize = 12.sp, color = p.textSecondary)
                    }
                }
            }
        }
    }
}

@Composable
fun FileEditorScreen(state: JSONObject) {
    val p = LocalBoxPalette.current
    val path = state.optString("editor_path", "")
    var text by remember(path, state.optString("editor_content")) { mutableStateOf(state.optString("editor_content", "")) }
    val diags = state.optJSONArray("editor_diagnostics")
    Column(modifier = Modifier.fillMaxSize().background(p.background)) {
        MiuixSubPageTopBar(
            title = path.substringAfterLast('/').ifEmpty { "编辑器" },
            collapsed = true,
            onBack = { dispatchAsync("pop_back") },
            trailing = {
                MiuixPillBtn("保存", primary = true) {
                    dispatchAsync("save_editor_file") { put("relative_path", path); put("content", text) }
                }
            },
        )
        Column(modifier = Modifier.fillMaxSize().padding(12.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
            if (diags != null && diags.length() > 0) {
                for (i in 0 until diags.length()) {
                    val d = diags.optJSONObject(i) ?: continue
                    Text("L${d.optInt("line", 1)} [${d.strOr("severity")}]: ${d.strOr("message")}", color = p.stopRed, fontSize = 12.sp)
                }
            }
            OutlinedTextField(
                value = text,
                onValueChange = { text = it },
                modifier = Modifier.fillMaxSize(),
                shape = RoundedCornerShape(16.dp),
            )
        }
    }
}

@OptIn(ExperimentalLayoutApi::class)
@Composable
fun LogViewerScreen(state: JSONObject) {
    val p = LocalBoxPalette.current
    val logTail = state.optString("log_tail", "")
    val files = state.optJSONArray("log_files")
    Column(modifier = Modifier.fillMaxSize().background(p.background)) {
        MiuixSubPageTopBar(title = "日志查看", collapsed = false, onBack = { dispatchAsync("pop_back") })
        Column(modifier = Modifier.fillMaxSize().padding(horizontal = 16.dp, vertical = 8.dp), verticalArrangement = Arrangement.spacedBy(10.dp)) {
            FlowRow(horizontalArrangement = Arrangement.spacedBy(8.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
                if (files != null) {
                    for (i in 0 until files.length()) {
                        val name = files.optJSONObject(i)?.optString("name").orEmpty()
                        MiuixPillBtn(name, primary = name == "honk.log") {
                            dispatchAsync("read_log_tail") { put("filename", name); put("max_lines", 300) }
                        }
                    }
                }
            }
            MiuixCard(modifier = Modifier.fillMaxSize()) {
                LazyColumn(modifier = Modifier.fillMaxSize().padding(14.dp)) {
                    item {
                        Text(
                            logTail.ifEmpty { "(点击上方日志文件加载)" },
                            fontFamily = FontFamily.Monospace,
                            fontSize = 12.sp,
                            color = p.textPrimary,
                        )
                    }
                }
            }
        }
    }
}

@Composable
fun CoreUpdateSourcesScreen(state: JSONObject) {
    val p = LocalBoxPalette.current
    val sources = state.optJSONArray("builtin_update_sources") ?: JSONArray()
    val results = state.optJSONObject("update_check_results") ?: JSONObject()
    Column(modifier = Modifier.fillMaxSize().background(p.background)) {
        MiuixSubPageTopBar(title = "更新核心", collapsed = false, onBack = { dispatchAsync("pop_back") })
        LazyColumn(contentPadding = PaddingValues(16.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
            items(sources.length()) { idx ->
                val s = sources.optJSONObject(idx) ?: JSONObject()
                val id = s.strOr("id")
                val res = results.optJSONObject(id)
                MiuixCard(modifier = Modifier.fillMaxWidth()) {
                    Column(modifier = Modifier.padding(18.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
                        Text(s.strOr("name"), fontSize = 16.sp, fontWeight = FontWeight.Medium, color = p.textPrimary)
                        Text("${s.strOr("repo")} · ${s.strOr("target_core")}", fontSize = 13.sp, color = p.textSecondary)
                        if (res != null) {
                            Text(
                                "本地: ${res.strOr("local_version")} → 远端: ${res.strOr("remote_version")}",
                                fontSize = 13.sp,
                                color = if (res.optBoolean("update_available", false)) p.upGreen else p.textSecondary,
                            )
                        }
                        Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                            MiuixPillBtn("检查更新", primary = true) {
                                dispatchAsync("check_core_update") { put("source_id", id); put("refresh", true) }
                            }
                            if (res != null && res.optBoolean("update_available", false)) {
                                MiuixPillBtn("下载并安装", primary = true) {
                                    dispatchAsync("install_core_update") { put("source_id", id) }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

@OptIn(ExperimentalLayoutApi::class)
@Composable
fun NetworkDiagnosticsScreen(state: JSONObject) {
    val p = LocalBoxPalette.current
    val probes = state.optJSONArray("platform_probes")
    val ipQuality = state.optJSONObject("ip_quality")
    val speedTest = state.optJSONObject("last_speed_test")
    val speedRunning = state.optBoolean("speed_test_running", false)

    Column(modifier = Modifier.fillMaxSize().background(p.background)) {
        MiuixSubPageTopBar(title = "网络测试", collapsed = false, onBack = { dispatchAsync("pop_back") })
        LazyColumn(contentPadding = PaddingValues(16.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
            item {
                FlowRow(horizontalArrangement = Arrangement.spacedBy(8.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
                    MiuixPillBtn("29 平台连通性探测", primary = true) { dispatchAsync("run_platform_probes") }
                    MiuixPillBtn("IP 质量") { dispatchAsync("run_ip_quality") }
                    MiuixPillBtn(if (speedRunning) "测速中…" else "Cloudflare 测速", enabled = !speedRunning) {
                        dispatchAsync("run_speed_test") { put("source", "cloudflare") }
                    }
                }
            }
            if (ipQuality != null && ipQuality.optString("exit_ip").isNotEmpty()) {
                item {
                    MiuixCard(modifier = Modifier.fillMaxWidth()) {
                        Column(modifier = Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(4.dp)) {
                            Text("出口 IP 质量 (Net.Coffee)", fontSize = 15.sp, fontWeight = FontWeight.Medium, color = p.textPrimary)
                            Text("IP: ${ipQuality.strOr("exit_ip")} · ${ipQuality.strOr("location")} · ${ipQuality.strOr("colo")} · ${ipQuality.strOr("verdict")}", fontSize = 13.sp, color = p.textSecondary)
                        }
                    }
                }
            }
            if (speedTest != null && speedTest.optDouble("download_mbps", 0.0) > 0.0) {
                item {
                    MiuixCard(modifier = Modifier.fillMaxWidth()) {
                        Column(modifier = Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(4.dp)) {
                            Text("测速结果 (${speedTest.strOr("source")})", fontSize = 15.sp, fontWeight = FontWeight.Medium, color = p.textPrimary)
                            Text("↓ ${"%.2f".format(speedTest.optDouble("download_mbps", 0.0))} Mbps · ↑ ${"%.2f".format(speedTest.optDouble("upload_mbps", 0.0))} Mbps · RTT ${"%.1f".format(speedTest.optDouble("latency_ms", 0.0))} ms", fontSize = 13.sp, color = p.textSecondary)
                        }
                    }
                }
            }
            if (probes != null) {
                items(probes.length()) { idx ->
                    val pr = probes.optJSONObject(idx) ?: JSONObject()
                    val region = pr.optString("region", "").trim()
                    val title = if (region.isNotEmpty()) "${pr.strOr("name")} ($region)" else pr.strOr("name")
                    MiuixCard(modifier = Modifier.fillMaxWidth()) {
                        Row(modifier = Modifier.fillMaxWidth().padding(horizontal = 18.dp, vertical = 14.dp), horizontalArrangement = Arrangement.SpaceBetween) {
                            Text(title, fontSize = 15.sp, color = p.textPrimary)
                            Text(
                                if (pr.optBoolean("reachable", false)) "${pr.optLong("latency_ms", 0)} ms" else "失败",
                                color = if (pr.optBoolean("reachable", false)) p.upGreen else p.stopRed,
                                fontSize = 15.sp,
                                fontWeight = FontWeight.Medium,
                            )
                        }
                    }
                }
            }
        }
    }
}
