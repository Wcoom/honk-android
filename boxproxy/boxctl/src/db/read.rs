use super::*;
use std::collections::HashSet;

pub(super) struct WifiSettings {
    pub(super) network_control_enabled: bool,
    pub(super) use_on_wifi_disconnect: bool,
    pub(super) use_on_wifi_connect: bool,
    pub(super) enable_ssid_matching: bool,
    pub(super) enable_network_control_log: bool,
    pub(super) list_mode: String,
}

pub(super) struct HotspotSettings {
    pub(super) mac_filter: bool,
    pub(super) mac_mode: String,
}

pub(super) fn load_profile(conn: &Connection) -> Result<RuntimeData> {
    let has_tproxy_port = conn
        .prepare("SELECT tproxy_port FROM runtime_profile WHERE id = 1")
        .is_ok();
    let sql = if has_tproxy_port {
        "SELECT
            core_name, mode, proxy_mode, auto_sync_config, performance_mode,
            clean_vendor_firewall, ipv6_mode,
            config_name, tproxy_port, redir_port, quic, mihomo_dns_forward,
            mihomo_dns_port,
            proxy_tcp, proxy_udp, dns_hijack_tcp, dns_hijack_udp, dns_hijack_mode,
            cgroup_memcg, memcg_limit, taskset_cpu, allow_cpu, cgroup_blkio, weight,
            bypass_cn, tun_device, fake_ip_range, fake_ip6_range,
            cn.bypass_ipv4, cn.bypass_ipv6, cn.ipv4_file, cn.ipv6_file,
            cn.cnip_mode
         FROM runtime_profile
         LEFT JOIN cnip_settings cn ON cn.id = 1
         WHERE runtime_profile.id = 1"
    } else {
        "SELECT
            core_name, mode, proxy_mode, auto_sync_config, performance_mode,
            clean_vendor_firewall, ipv6_mode,
            config_name, '9898' AS tproxy_port, '9797' AS redir_port, quic, mihomo_dns_forward,
            '' AS mihomo_dns_port,
            proxy_tcp, proxy_udp, dns_hijack_tcp, dns_hijack_udp, dns_hijack_mode,
            cgroup_memcg, memcg_limit, taskset_cpu, allow_cpu, cgroup_blkio, weight,
            bypass_cn, '' AS tun_device, '' AS fake_ip_range, '' AS fake_ip6_range,
            cn.bypass_ipv4, cn.bypass_ipv6, cn.ipv4_file, cn.ipv6_file,
            cn.cnip_mode
         FROM runtime_profile
         LEFT JOIN cnip_settings cn ON cn.id = 1
         WHERE runtime_profile.id = 1"
    };
    let row = conn
        .query_row(
            sql,
            [],
            |row| {
                Ok(RuntimeData {
                    log_language: "en".to_string(),
                    core_name: row.get(0)?,
                    mode: row.get(1)?,
                    proxy_mode: row.get(2)?,
                    auto_sync_config: row.get::<_, i64>(3)? != 0,
                    performance_mode: row.get::<_, i64>(4)? != 0,
                    clean_vendor_firewall: row.get::<_, i64>(5)? != 0,
                    ipv6_mode: normalize_ipv6_mode(row.get::<_, Option<String>>(6)?.as_deref()),
                    config_name: row.get(7)?,
                    tproxy_port: row.get(8)?,
                    redir_port: row.get(9)?,
                    quic: row.get(10)?,
                    mihomo_dns_forward: row.get(11)?,
                    mihomo_dns_port: row.get(12)?,
                    proxy_tcp: row.get::<_, i64>(13)? != 0,
                    proxy_udp: row.get::<_, i64>(14)? != 0,
                    dns_hijack_tcp: row.get::<_, i64>(15)? != 0,
                    dns_hijack_udp: row.get::<_, i64>(16)? != 0,
                    dns_hijack_mode: row.get(17)?,
                    cgroup_memcg: row.get::<_, i64>(18)? != 0,
                    memcg_limit: row.get(19)?,
                    taskset_cpu: row.get::<_, i64>(20)? != 0,
                    allow_cpu: row.get(21)?,
                    cgroup_blkio: row.get::<_, i64>(22)? != 0,
                    weight: row.get(23)?,
                    bypass_cn: row.get::<_, i64>(24)? != 0,
                    tun_device: row.get(25)?,
                    fake_ip_range: row.get(26)?,
                    fake_ip6_range: row.get(27)?,
                    bypass_cn_v4: row.get::<_, i64>(28)? != 0,
                    bypass_cn_v6: row.get::<_, i64>(29)? != 0,
                    cn_ip_file: row.get(30)?,
                    cn_ipv6_file: row.get(31)?,
                    cnip_mode: normalize_cnip_mode(row.get::<_, Option<String>>(32)?.as_deref()),
                    selected_uids: Vec::new(),
                    gid_list: Vec::new(),
                    cnip_force_uids: Vec::new(),
                    wifi_network_control_enabled: false,
                    wifi_use_on_disconnect: true,
                    wifi_use_on_connect: true,
                    wifi_enable_ssid_matching: false,
                    wifi_enable_log: true,
                    wifi_list_mode: "blacklist".to_string(),
                    wifi_ssids: Vec::new(),
                    wifi_bssids: Vec::new(),
                    network_rule_groups: Vec::new(),
                    hotspot_ap_interfaces: Vec::new(),
                    blocked_interfaces: Vec::new(),
                    mac_filter: false,
                    mac_mode: "blacklist".to_string(),
                    macs_list: Vec::new(),
                    intranet_cidrs4: Vec::new(),
                    intranet_cidrs6: Vec::new(),
                })
            },
        )
        .map_err(|err| format!("read database config failed: {err}"))?;
    Ok(row)
}

pub(super) fn read_app_setting(conn: &Connection, key: &str, default: &str) -> String {
    conn.query_row(
        "SELECT value FROM app_settings WHERE key = ?1",
        [key],
        |row| row.get::<_, String>(0),
    )
    .ok()
    .map(|value| normalize_app_language(&value))
    .unwrap_or_else(|| default.to_string())
}

pub(super) fn read_uid_list(conn: &Connection, table: &str, column: &str) -> Vec<String> {
    let sql = format!("SELECT DISTINCT {column} FROM {table} ORDER BY {column}");
    let mut stmt = match conn.prepare(&sql) {
        Ok(stmt) => stmt,
        Err(_) => return Vec::new(),
    };
    let rows = match stmt.query_map([], |row| {
        let value: i64 = row.get(0)?;
        Ok(value.to_string())
    }) {
        Ok(rows) => rows,
        Err(_) => return Vec::new(),
    };

    rows.filter_map(|row| row.ok()).collect()
}

pub(super) fn read_wifi_settings(conn: &Connection) -> Result<WifiSettings> {
    for table in ["network_match_settings", "wifi_match_settings"] {
        let sql = format!(
            "SELECT network_control_enabled, use_on_wifi_disconnect, use_on_wifi_connect, \
             enable_ssid_matching, enable_network_control_log, list_mode \
             FROM {table} WHERE id = 1"
        );
        if let Ok((
            network_control_enabled,
            use_on_wifi_disconnect,
            use_on_wifi_connect,
            enable_ssid_matching,
            enable_network_control_log,
            list_mode,
        )) = conn.query_row(&sql, [], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, i64>(3)?,
                row.get::<_, i64>(4)?,
                row.get::<_, String>(5)?,
            ))
        }) {
            return Ok(WifiSettings {
                network_control_enabled: network_control_enabled != 0,
                use_on_wifi_disconnect: use_on_wifi_disconnect != 0,
                use_on_wifi_connect: use_on_wifi_connect != 0,
                enable_ssid_matching: enable_ssid_matching != 0,
                enable_network_control_log: enable_network_control_log != 0,
                list_mode,
            });
        }
    }
    Ok(WifiSettings {
        network_control_enabled: false,
        use_on_wifi_disconnect: true,
        use_on_wifi_connect: true,
        enable_ssid_matching: false,
        enable_network_control_log: true,
        list_mode: "blacklist".to_string(),
    })
}

pub(super) fn read_hotspot_settings(conn: &Connection) -> Result<HotspotSettings> {
    for table in ["shared_network_settings", "hotspot_settings"] {
        let sql = format!("SELECT mac_filter, mac_mode FROM {table} WHERE id = 1");
        if let Ok((mac_filter, mac_mode)) =
            conn.query_row(&sql, [], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?)))
        {
            return Ok(HotspotSettings {
                mac_filter: mac_filter != 0,
                mac_mode,
            });
        }
    }
    Ok(HotspotSettings {
        mac_filter: false,
        mac_mode: "blacklist".to_string(),
    })
}

pub(super) fn read_string_list_preferred(
    conn: &Connection,
    preferred: &str,
    legacy: &str,
    column: &str,
) -> Result<Vec<String>> {
    let primary = read_string_list(conn, preferred, column)?;
    if !primary.is_empty() {
        return Ok(primary);
    }
    read_string_list(conn, legacy, column)
}

pub(super) fn read_network_rule_groups(conn: &Connection) -> Vec<boxproxy_schema::NetworkRuleGroup> {
    let Ok(mut stmt) = conn.prepare(
        "SELECT id, name, enabled, priority, expression FROM network_rule_groups ORDER BY priority ASC, id ASC",
    ) else {
        return Vec::new();
    };
    let Ok(rows) = stmt.query_map([], |row| {
        Ok(boxproxy_schema::NetworkRuleGroup {
            id: row.get(0)?,
            name: row.get(1)?,
            enabled: row.get::<_, i64>(2)? != 0,
            priority: row.get(3)?,
            expression: row.get(4)?,
            actions: Vec::new(),
        })
    }) else {
        return Vec::new();
    };
    let mut groups: Vec<boxproxy_schema::NetworkRuleGroup> = rows.filter_map(|r| r.ok()).collect();
    if let Ok(mut act_stmt) = conn.prepare(
        "SELECT id, group_id, sort_order, type, target, continue_on_error \
         FROM network_rule_actions WHERE group_id = ?1 ORDER BY sort_order ASC, id ASC",
    ) {
        for group in &mut groups {
            if let Ok(act_rows) = act_stmt.query_map([group.id], |row| {
                Ok(boxproxy_schema::NetworkRuleAction {
                    id: row.get(0)?,
                    group_id: row.get(1)?,
                    sort_order: row.get(2)?,
                    action_type: row.get(3)?,
                    target: row.get(4)?,
                    continue_on_error: row.get::<_, i64>(5)? != 0,
                })
            }) {
                group.actions = act_rows.filter_map(|r| r.ok()).collect();
            }
        }
    }
    groups
}

pub(super) fn read_string_list(
    conn: &Connection,
    table: &str,
    column: &str,
) -> Result<Vec<String>> {
    let sql = format!("SELECT {column} FROM {table} ORDER BY id");
    let Ok(mut stmt) = conn.prepare(&sql) else {
        return Ok(Vec::new());
    };
    let rows = stmt
        .query_map([], |row| {
            let value: String = row.get(0)?;
            Ok(value)
        })
        .map_err(|err| format!("read {table} failed: {err}"))?;

    let mut seen = HashSet::new();
    let mut values = Vec::new();
    for value in rows.filter_map(|row| row.ok()) {
        let value = value.trim().to_string();
        if !value.is_empty() && seen.insert(value.clone()) {
            values.push(value);
        }
    }
    Ok(values)
}

pub(super) fn normalize_app_language(value: &str) -> String {
    if value.trim().eq_ignore_ascii_case("en") {
        "en".to_string()
    } else {
        "zh-CN".to_string()
    }
}

pub(super) fn normalize_ipv6_mode(value: Option<&str>) -> String {
    match value
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "enable" | "enabled" | "true" | "1" => "enable".to_string(),
        "disable" | "disabled" | "system_disable" | "off" => "disable".to_string(),
        "bypass" | "bypassed" | "false" | "0" => "bypass".to_string(),
        _ => "enable".to_string(),
    }
}

pub(super) fn normalize_cnip_mode(value: Option<&str>) -> String {
    match value
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "ebpf" => "ebpf".to_string(),
        _ => "ipset".to_string(),
    }
}
