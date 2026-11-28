//! Transactional repository for all 21 `box.db` schema v15 tables.

use crate::StoreResult;
use boxproxy_schema::{
    config_override_key, ensure_schema_v15, seed_default_rows, table_columns, CnipSettings,
    NetworkMatchCondition, NetworkMatchSettings, NetworkRuleAction, NetworkRuleGroup,
    RuntimeProfile, SharedNetworkSettings, CONFIG_OVERRIDE_OPTIONS, DEFAULT_INTRANET_IPV4_CIDRS,
    DEFAULT_INTRANET_IPV6_CIDRS,
};
use rusqlite::{params, Connection};
use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};
use std::time::Duration;

pub struct BoxDbRepo {
    db_path: PathBuf,
    box_home: PathBuf,
}

impl BoxDbRepo {
    pub fn open(db_path: impl Into<PathBuf>, box_home: impl Into<PathBuf>) -> StoreResult<Self> {
        let db_path = db_path.into();
        let box_home = box_home.into();
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("create db parent dir failed: {e}"))?;
        }
        let repo = Self { db_path, box_home };
        let conn = repo.connect()?;
        ensure_schema_v15(&conn)?;
        seed_default_rows(&conn, &repo.box_home)?;
        Ok(repo)
    }

    pub fn db_path(&self) -> &Path {
        &self.db_path
    }

    fn connect(&self) -> StoreResult<Connection> {
        let conn = Connection::open(&self.db_path)
            .map_err(|e| format!("open {} failed: {e}", self.db_path.display()))?;
        let _ = conn.busy_timeout(Duration::from_secs(5));
        let _ = conn.execute_batch("PRAGMA journal_mode = WAL; PRAGMA synchronous = NORMAL;");
        Ok(conn)
    }

    // -------------------------------------------------------------------------
    // 1. runtime_profile
    // -------------------------------------------------------------------------

    pub fn get_runtime_profile(&self) -> StoreResult<RuntimeProfile> {
        let conn = self.connect()?;
        let cols = table_columns(&conn, "runtime_profile")?;
        let has_legacy = cols.iter().any(|c| c.eq_ignore_ascii_case("tproxy_port"));
        let sql = if has_legacy {
            "SELECT core_name, mode, proxy_mode, auto_sync_config, performance_mode,
                    clean_vendor_firewall, ipv6_mode, config_name, quic, mihomo_dns_forward,
                    proxy_tcp, proxy_udp, dns_hijack_tcp, dns_hijack_udp, dns_hijack_mode,
                    cgroup_memcg, memcg_limit, taskset_cpu, allow_cpu, cgroup_blkio, weight,
                    bypass_cn, boot_auto_start,
                    tproxy_port, redir_port, mihomo_dns_port, tun_device, fake_ip_range, fake_ip6_range
             FROM runtime_profile WHERE id = 1"
        } else {
            "SELECT core_name, mode, proxy_mode, auto_sync_config, performance_mode,
                    clean_vendor_firewall, ipv6_mode, config_name, quic, mihomo_dns_forward,
                    proxy_tcp, proxy_udp, dns_hijack_tcp, dns_hijack_udp, dns_hijack_mode,
                    cgroup_memcg, memcg_limit, taskset_cpu, allow_cpu, cgroup_blkio, weight,
                    bypass_cn, boot_auto_start,
                    '', '', '', '', '', ''
             FROM runtime_profile WHERE id = 1"
        };
        conn.query_row(sql, [], |r| {
            Ok(RuntimeProfile {
                core_name: r.get(0)?,
                mode: r.get(1)?,
                proxy_mode: r.get(2)?,
                auto_sync_config: r.get::<_, i64>(3)? != 0,
                performance_mode: r.get::<_, i64>(4)? != 0,
                clean_vendor_firewall: r.get::<_, i64>(5)? != 0,
                ipv6_mode: r.get(6)?,
                config_name: r.get(7)?,
                quic: r.get(8)?,
                mihomo_dns_forward: r.get(9)?,
                proxy_tcp: r.get::<_, i64>(10)? != 0,
                proxy_udp: r.get::<_, i64>(11)? != 0,
                dns_hijack_tcp: r.get::<_, i64>(12)? != 0,
                dns_hijack_udp: r.get::<_, i64>(13)? != 0,
                dns_hijack_mode: r.get(14)?,
                cgroup_memcg: r.get::<_, i64>(15)? != 0,
                memcg_limit: r.get(16)?,
                taskset_cpu: r.get::<_, i64>(17)? != 0,
                allow_cpu: r.get(18)?,
                cgroup_blkio: r.get::<_, i64>(19)? != 0,
                weight: r.get(20)?,
                bypass_cn: r.get::<_, i64>(21)? != 0,
                boot_auto_start: r.get::<_, i64>(22)? != 0,
                tproxy_port: r.get(23)?,
                redir_port: r.get(24)?,
                mihomo_dns_port: r.get(25)?,
                tun_device: r.get(26)?,
                fake_ip_range: r.get(27)?,
                fake_ip6_range: r.get(28)?,
            })
        })
        .map_err(|e| format!("read runtime_profile failed: {e}"))
    }

    pub fn save_runtime_profile(&self, p: &RuntimeProfile) -> StoreResult<()> {
        let conn = self.connect()?;
        let cols = table_columns(&conn, "runtime_profile")?;
        let has_legacy = cols.iter().any(|c| c.eq_ignore_ascii_case("tproxy_port"));
        if has_legacy {
            conn.execute(
                "INSERT OR REPLACE INTO runtime_profile (
                    id, core_name, mode, proxy_mode, auto_sync_config, performance_mode,
                    clean_vendor_firewall, ipv6_mode, config_name,
                    tproxy_port, redir_port, quic, mihomo_dns_forward, mihomo_dns_port,
                    proxy_tcp, proxy_udp, dns_hijack_tcp, dns_hijack_udp, dns_hijack_mode,
                    cgroup_memcg, memcg_limit, taskset_cpu, allow_cpu, cgroup_blkio, weight,
                    bypass_cn, tun_device, fake_ip_range, fake_ip6_range, boot_auto_start
                ) VALUES (
                    1, ?1, ?2, ?3, ?4, ?5,
                    ?6, ?7, ?8,
                    ?9, ?10, ?11, ?12, ?13,
                    ?14, ?15, ?16, ?17, ?18,
                    ?19, ?20, ?21, ?22, ?23, ?24,
                    ?25, ?26, ?27, ?28, ?29
                )",
                params![
                    p.core_name,
                    p.mode,
                    p.proxy_mode,
                    i64::from(p.auto_sync_config),
                    i64::from(p.performance_mode),
                    i64::from(p.clean_vendor_firewall),
                    p.ipv6_mode,
                    p.config_name,
                    p.tproxy_port,
                    p.redir_port,
                    p.quic,
                    p.mihomo_dns_forward,
                    p.mihomo_dns_port,
                    i64::from(p.proxy_tcp),
                    i64::from(p.proxy_udp),
                    i64::from(p.dns_hijack_tcp),
                    i64::from(p.dns_hijack_udp),
                    p.dns_hijack_mode,
                    i64::from(p.cgroup_memcg),
                    p.memcg_limit,
                    i64::from(p.taskset_cpu),
                    p.allow_cpu,
                    i64::from(p.cgroup_blkio),
                    p.weight,
                    i64::from(p.bypass_cn),
                    p.tun_device,
                    p.fake_ip_range,
                    p.fake_ip6_range,
                    i64::from(p.boot_auto_start),
                ],
            )
            .map_err(|e| format!("save legacy runtime_profile failed: {e}"))?;
        } else {
            conn.execute(
                "INSERT OR REPLACE INTO runtime_profile (
                    id, core_name, mode, proxy_mode, auto_sync_config, performance_mode,
                    clean_vendor_firewall, ipv6_mode, config_name, quic, mihomo_dns_forward,
                    proxy_tcp, proxy_udp, dns_hijack_tcp, dns_hijack_udp, dns_hijack_mode,
                    cgroup_memcg, memcg_limit, taskset_cpu, allow_cpu, cgroup_blkio, weight,
                    bypass_cn, boot_auto_start
                ) VALUES (
                    1, ?1, ?2, ?3, ?4, ?5,
                    ?6, ?7, ?8, ?9, ?10,
                    ?11, ?12, ?13, ?14, ?15,
                    ?16, ?17, ?18, ?19, ?20, ?21,
                    ?22, ?23
                )",
                params![
                    p.core_name,
                    p.mode,
                    p.proxy_mode,
                    i64::from(p.auto_sync_config),
                    i64::from(p.performance_mode),
                    i64::from(p.clean_vendor_firewall),
                    p.ipv6_mode,
                    p.config_name,
                    p.quic,
                    p.mihomo_dns_forward,
                    i64::from(p.proxy_tcp),
                    i64::from(p.proxy_udp),
                    i64::from(p.dns_hijack_tcp),
                    i64::from(p.dns_hijack_udp),
                    p.dns_hijack_mode,
                    i64::from(p.cgroup_memcg),
                    p.memcg_limit,
                    i64::from(p.taskset_cpu),
                    p.allow_cpu,
                    i64::from(p.cgroup_blkio),
                    p.weight,
                    i64::from(p.bypass_cn),
                    i64::from(p.boot_auto_start),
                ],
            )
            .map_err(|e| format!("save runtime_profile failed: {e}"))?;
        }
        Ok(())
    }

    // -------------------------------------------------------------------------
    // 2. app_settings & 30 config_override.<core>.<option> keys
    // -------------------------------------------------------------------------

    pub fn get_app_setting(&self, key: &str) -> StoreResult<Option<String>> {
        let conn = self.connect()?;
        match conn.query_row(
            "SELECT value FROM app_settings WHERE key = ?1",
            [key],
            |r| r.get(0),
        ) {
            Ok(v) => Ok(Some(v)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(format!("read app_settings[{key}] failed: {e}")),
        }
    }

    pub fn set_app_setting(&self, key: &str, value: &str) -> StoreResult<()> {
        let conn = self.connect()?;
        conn.execute(
            "INSERT OR REPLACE INTO app_settings (key, value) VALUES (?1, ?2)",
            params![key, value],
        )
        .map_err(|e| format!("write app_settings[{key}] failed: {e}"))?;
        Ok(())
    }

    pub fn get_all_app_settings(&self) -> StoreResult<BTreeMap<String, String>> {
        let conn = self.connect()?;
        let mut stmt = conn
            .prepare("SELECT key, value FROM app_settings ORDER BY key")
            .map_err(|e| format!("prepare app_settings failed: {e}"))?;
        let rows = stmt
            .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
            .map_err(|e| format!("query app_settings failed: {e}"))?;
        let mut map = BTreeMap::new();
        for row in rows.flatten() {
            map.insert(row.0, row.1);
        }
        Ok(map)
    }

    pub fn get_config_overrides(&self, core: &str) -> StoreResult<BTreeMap<String, String>> {
        let all = self.get_all_app_settings()?;
        let mut out = BTreeMap::new();
        for &opt in CONFIG_OVERRIDE_OPTIONS {
            let key = config_override_key(core, opt);
            if let Some(val) = all.get(&key) {
                out.insert(opt.to_string(), val.clone());
            }
        }
        Ok(out)
    }

    pub fn set_config_override(&self, core: &str, option: &str, value: &str) -> StoreResult<()> {
        if !CONFIG_OVERRIDE_OPTIONS.contains(&option) {
            return Err(format!("unsupported config override option: {option}"));
        }
        self.set_app_setting(&config_override_key(core, option), value)
    }

    // -------------------------------------------------------------------------
    // 3. app_selection & app_gid_list
    // -------------------------------------------------------------------------

    pub fn get_app_selection(&self) -> StoreResult<Vec<u32>> {
        self.read_u32_table("app_selection", "uid")
    }

    pub fn set_app_selection(&self, uids: &[u32]) -> StoreResult<()> {
        self.replace_u32_table("app_selection", "uid", uids)
    }

    pub fn get_gid_list(&self) -> StoreResult<Vec<String>> {
        self.read_string_table("app_gid_list")
    }

    pub fn set_gid_list(&self, gids: &[String]) -> StoreResult<()> {
        self.replace_string_table("app_gid_list", gids)
    }

    // -------------------------------------------------------------------------
    // 4. cnip_settings, cnip_force_uids, cnip_force_apps
    // -------------------------------------------------------------------------

    pub fn get_cnip_settings(&self) -> StoreResult<CnipSettings> {
        let conn = self.connect()?;
        conn.query_row(
            "SELECT bypass_cnip, cnip_mode, bypass_ipv4, bypass_ipv6,
                    ipv4_file, ipv4_url, ipv6_file, ipv6_url,
                    ipv4_source, ipv6_source, ipv4_additions, ipv6_additions
             FROM cnip_settings WHERE id = 1",
            [],
            |r| {
                Ok(CnipSettings {
                    bypass_cnip: r.get::<_, i64>(0)? != 0,
                    cnip_mode: r.get(1)?,
                    bypass_ipv4: r.get::<_, i64>(2)? != 0,
                    bypass_ipv6: r.get::<_, i64>(3)? != 0,
                    ipv4_file: r.get(4)?,
                    ipv4_url: r.get(5)?,
                    ipv6_file: r.get(6)?,
                    ipv6_url: r.get(7)?,
                    ipv4_source: r.get(8)?,
                    ipv6_source: r.get(9)?,
                    ipv4_additions: r.get(10)?,
                    ipv6_additions: r.get(11)?,
                })
            },
        )
        .map_err(|e| format!("read cnip_settings failed: {e}"))
    }

    pub fn save_cnip_settings(&self, s: &CnipSettings) -> StoreResult<()> {
        let conn = self.connect()?;
        conn.execute(
            "INSERT OR REPLACE INTO cnip_settings (
                id, bypass_cnip, cnip_mode, bypass_ipv4, bypass_ipv6,
                ipv4_file, ipv4_url, ipv6_file, ipv6_url,
                ipv4_source, ipv6_source, ipv4_additions, ipv6_additions
            ) VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
            params![
                i64::from(s.bypass_cnip),
                s.cnip_mode,
                i64::from(s.bypass_ipv4),
                i64::from(s.bypass_ipv6),
                s.ipv4_file,
                s.ipv4_url,
                s.ipv6_file,
                s.ipv6_url,
                s.ipv4_source,
                s.ipv6_source,
                s.ipv4_additions,
                s.ipv6_additions,
            ],
        )
        .map_err(|e| format!("save cnip_settings failed: {e}"))?;
        // Mirror bypass_cn flag to runtime_profile.bypass_cn for boxctl parity.
        let _ = conn.execute(
            "UPDATE runtime_profile SET bypass_cn = ?1 WHERE id = 1",
            [i64::from(s.bypass_cnip)],
        );
        Ok(())
    }

    pub fn get_cnip_force_uids(&self) -> StoreResult<Vec<u32>> {
        self.read_u32_table("cnip_force_uids", "uid")
    }

    pub fn get_cnip_force_apps(&self) -> StoreResult<Vec<String>> {
        self.read_string_table("cnip_force_apps")
    }

    pub fn set_cnip_force_apps(&self, apps: &[String], resolved_uids: &[u32]) -> StoreResult<()> {
        self.replace_string_table("cnip_force_apps", apps)?;
        self.replace_u32_table("cnip_force_uids", "uid", resolved_uids)
    }

    // -------------------------------------------------------------------------
    // 5. network_match_* & network_rule_{groups,actions}
    // -------------------------------------------------------------------------

    pub fn get_network_match_settings(&self) -> StoreResult<NetworkMatchSettings> {
        let conn = self.connect()?;
        conn.query_row(
            "SELECT network_control_enabled, use_on_wifi_disconnect, use_on_wifi_connect,
                    network_enable_on_unmatched, network_enable_on_matched,
                    matched_action, unmatched_action,
                    enable_ssid_matching, enable_network_control_log, list_mode,
                    switch_config, matched_config_name, unmatched_config_name,
                    matched_script_name, unmatched_script_name
             FROM network_match_settings WHERE id = 1",
            [],
            |r| {
                Ok(NetworkMatchSettings {
                    network_control_enabled: r.get::<_, i64>(0)? != 0,
                    use_on_wifi_disconnect: r.get::<_, i64>(1)? != 0,
                    use_on_wifi_connect: r.get::<_, i64>(2)? != 0,
                    network_enable_on_unmatched: r.get::<_, i64>(3)? != 0,
                    network_enable_on_matched: r.get::<_, i64>(4)? != 0,
                    matched_action: r.get(5)?,
                    unmatched_action: r.get(6)?,
                    enable_ssid_matching: r.get::<_, i64>(7)? != 0,
                    enable_network_control_log: r.get::<_, i64>(8)? != 0,
                    list_mode: r.get(9)?,
                    switch_config: r.get::<_, i64>(10)? != 0,
                    matched_config_name: r.get(11)?,
                    unmatched_config_name: r.get(12)?,
                    matched_script_name: r.get(13)?,
                    unmatched_script_name: r.get(14)?,
                })
            },
        )
        .map_err(|e| format!("read network_match_settings failed: {e}"))
    }

    pub fn save_network_match_settings(&self, s: &NetworkMatchSettings) -> StoreResult<()> {
        let conn = self.connect()?;
        conn.execute(
            "INSERT OR REPLACE INTO network_match_settings (
                id, network_control_enabled, use_on_wifi_disconnect, use_on_wifi_connect,
                network_enable_on_unmatched, network_enable_on_matched,
                matched_action, unmatched_action,
                enable_ssid_matching, enable_network_control_log, list_mode,
                switch_config, matched_config_name, unmatched_config_name,
                matched_script_name, unmatched_script_name
            ) VALUES (1, ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15)",
            params![
                i64::from(s.network_control_enabled),
                i64::from(s.use_on_wifi_disconnect),
                i64::from(s.use_on_wifi_connect),
                i64::from(s.network_enable_on_unmatched),
                i64::from(s.network_enable_on_matched),
                s.matched_action,
                s.unmatched_action,
                i64::from(s.enable_ssid_matching),
                i64::from(s.enable_network_control_log),
                s.list_mode,
                i64::from(s.switch_config),
                s.matched_config_name,
                s.unmatched_config_name,
                s.matched_script_name,
                s.unmatched_script_name,
            ],
        )
        .map_err(|e| format!("save network_match_settings failed: {e}"))?;
        Ok(())
    }

    pub fn get_network_match_ssids(&self) -> StoreResult<Vec<String>> {
        self.read_string_table("network_match_ssids")
    }

    pub fn set_network_match_ssids(&self, values: &[String]) -> StoreResult<()> {
        self.replace_string_table("network_match_ssids", values)
    }

    pub fn get_network_match_bssids(&self) -> StoreResult<Vec<String>> {
        self.read_string_table("network_match_bssids")
    }

    pub fn set_network_match_bssids(&self, values: &[String]) -> StoreResult<()> {
        self.replace_string_table("network_match_bssids", values)
    }

    pub fn get_network_match_conditions(&self) -> StoreResult<Vec<NetworkMatchCondition>> {
        let conn = self.connect()?;
        let mut stmt = conn
            .prepare(
                "SELECT id, type, value, enabled, sort_order
                 FROM network_match_conditions ORDER BY sort_order ASC, id ASC",
            )
            .map_err(|e| format!("prepare network_match_conditions failed: {e}"))?;
        let rows = stmt
            .query_map([], |r| {
                Ok(NetworkMatchCondition {
                    id: r.get(0)?,
                    condition_type: r.get(1)?,
                    value: r.get(2)?,
                    enabled: r.get::<_, i64>(3)? != 0,
                    sort_order: r.get(4)?,
                })
            })
            .map_err(|e| format!("query network_match_conditions failed: {e}"))?;
        Ok(rows.flatten().collect())
    }

    pub fn set_network_match_conditions(&self, items: &[NetworkMatchCondition]) -> StoreResult<()> {
        let conn = self.connect()?;
        conn.execute_batch("BEGIN IMMEDIATE")
            .map_err(|e| format!("begin failed: {e}"))?;
        let res = (|| -> StoreResult<()> {
            conn.execute("DELETE FROM network_match_conditions", [])
                .map_err(|e| format!("clear conditions failed: {e}"))?;
            for (idx, item) in items.iter().enumerate() {
                conn.execute(
                    "INSERT INTO network_match_conditions (type, value, enabled, sort_order)
                     VALUES (?1, ?2, ?3, ?4)",
                    params![
                        item.condition_type,
                        item.value,
                        i64::from(item.enabled),
                        idx as i64,
                    ],
                )
                .map_err(|e| format!("insert condition failed: {e}"))?;
            }
            Ok(())
        })();
        finish_tx(&conn, res)
    }

    pub fn list_network_rule_groups(&self) -> StoreResult<Vec<NetworkRuleGroup>> {
        let conn = self.connect()?;
        let mut stmt = conn
            .prepare(
                "SELECT id, name, enabled, priority, expression
                 FROM network_rule_groups ORDER BY priority ASC, id ASC",
            )
            .map_err(|e| format!("prepare network_rule_groups failed: {e}"))?;
        let mut groups: Vec<NetworkRuleGroup> = stmt
            .query_map([], |r| {
                Ok(NetworkRuleGroup {
                    id: r.get(0)?,
                    name: r.get(1)?,
                    enabled: r.get::<_, i64>(2)? != 0,
                    priority: r.get(3)?,
                    expression: r.get(4)?,
                    actions: Vec::new(),
                })
            })
            .map_err(|e| format!("query network_rule_groups failed: {e}"))?
            .flatten()
            .collect();

        let mut act_stmt = conn
            .prepare(
                "SELECT id, group_id, sort_order, type, target, continue_on_error
                 FROM network_rule_actions WHERE group_id = ?1 ORDER BY sort_order ASC, id ASC",
            )
            .map_err(|e| format!("prepare network_rule_actions failed: {e}"))?;
        for group in &mut groups {
            group.actions = act_stmt
                .query_map([group.id], |r| {
                    Ok(NetworkRuleAction {
                        id: r.get(0)?,
                        group_id: r.get(1)?,
                        sort_order: r.get(2)?,
                        action_type: r.get(3)?,
                        target: r.get(4)?,
                        continue_on_error: r.get::<_, i64>(5)? != 0,
                    })
                })
                .map_err(|e| format!("query network_rule_actions failed: {e}"))?
                .flatten()
                .collect();
        }
        Ok(groups)
    }

    pub fn upsert_network_rule_group(&self, group: &NetworkRuleGroup) -> StoreResult<i64> {
        let conn = self.connect()?;
        conn.execute_batch("BEGIN IMMEDIATE")
            .map_err(|e| format!("begin failed: {e}"))?;
        let res = (|| -> StoreResult<i64> {
            let group_id = if group.id > 0 {
                conn.execute(
                    "UPDATE network_rule_groups
                     SET name = ?1, enabled = ?2, priority = ?3, expression = ?4
                     WHERE id = ?5",
                    params![
                        group.name,
                        i64::from(group.enabled),
                        group.priority,
                        group.expression,
                        group.id,
                    ],
                )
                .map_err(|e| format!("update network_rule_groups failed: {e}"))?;
                group.id
            } else {
                conn.execute(
                    "INSERT INTO network_rule_groups (name, enabled, priority, expression)
                     VALUES (?1, ?2, ?3, ?4)",
                    params![
                        group.name,
                        i64::from(group.enabled),
                        group.priority,
                        group.expression,
                    ],
                )
                .map_err(|e| format!("insert network_rule_groups failed: {e}"))?;
                conn.last_insert_rowid()
            };

            conn.execute(
                "DELETE FROM network_rule_actions WHERE group_id = ?1",
                [group_id],
            )
            .map_err(|e| format!("clear actions failed: {e}"))?;

            for (idx, act) in group.actions.iter().enumerate() {
                conn.execute(
                    "INSERT INTO network_rule_actions
                     (group_id, sort_order, type, target, continue_on_error)
                     VALUES (?1, ?2, ?3, ?4, ?5)",
                    params![
                        group_id,
                        idx as i64,
                        act.action_type,
                        act.target,
                        i64::from(act.continue_on_error),
                    ],
                )
                .map_err(|e| format!("insert action failed: {e}"))?;
            }
            Ok(group_id)
        })();
        match res {
            Ok(id) => {
                conn.execute_batch("COMMIT")
                    .map_err(|e| format!("commit failed: {e}"))?;
                Ok(id)
            }
            Err(err) => {
                let _ = conn.execute_batch("ROLLBACK");
                Err(err)
            }
        }
    }

    pub fn delete_network_rule_group(&self, group_id: i64) -> StoreResult<()> {
        let conn = self.connect()?;
        conn.execute_batch("BEGIN IMMEDIATE")
            .map_err(|e| format!("begin failed: {e}"))?;
        let res = (|| -> StoreResult<()> {
            conn.execute(
                "DELETE FROM network_rule_actions WHERE group_id = ?1",
                [group_id],
            )
            .map_err(|e| format!("delete actions failed: {e}"))?;
            conn.execute("DELETE FROM network_rule_groups WHERE id = ?1", [group_id])
                .map_err(|e| format!("delete group failed: {e}"))?;
            Ok(())
        })();
        finish_tx(&conn, res)
    }

    // -------------------------------------------------------------------------
    // 6. shared_network_* & blocked_interfaces
    // -------------------------------------------------------------------------

    pub fn get_shared_network_settings(&self) -> StoreResult<SharedNetworkSettings> {
        let conn = self.connect()?;
        conn.query_row(
            "SELECT mac_filter, mac_mode FROM shared_network_settings WHERE id = 1",
            [],
            |r| {
                Ok(SharedNetworkSettings {
                    mac_filter: r.get::<_, i64>(0)? != 0,
                    mac_mode: r.get(1)?,
                })
            },
        )
        .map_err(|e| format!("read shared_network_settings failed: {e}"))
    }

    pub fn save_shared_network_settings(&self, s: &SharedNetworkSettings) -> StoreResult<()> {
        let conn = self.connect()?;
        conn.execute(
            "INSERT OR REPLACE INTO shared_network_settings (id, mac_filter, mac_mode)
             VALUES (1, ?1, ?2)",
            params![i64::from(s.mac_filter), s.mac_mode],
        )
        .map_err(|e| format!("save shared_network_settings failed: {e}"))?;
        Ok(())
    }

    pub fn get_shared_network_interfaces(&self) -> StoreResult<Vec<String>> {
        self.read_string_table("shared_network_interfaces")
    }

    pub fn set_shared_network_interfaces(&self, values: &[String]) -> StoreResult<()> {
        self.replace_string_table("shared_network_interfaces", values)
    }

    pub fn get_shared_network_macs(&self) -> StoreResult<Vec<String>> {
        self.read_string_table("shared_network_macs")
    }

    pub fn set_shared_network_macs(&self, values: &[String]) -> StoreResult<()> {
        self.replace_string_table("shared_network_macs", values)
    }

    pub fn get_blocked_interfaces(&self) -> StoreResult<Vec<String>> {
        self.read_string_table("blocked_interfaces")
    }

    pub fn set_blocked_interfaces(&self, values: &[String]) -> StoreResult<()> {
        self.replace_string_table("blocked_interfaces", values)
    }

    // -------------------------------------------------------------------------
    // 7. intranet_ipv4_cidrs & intranet_ipv6_cidrs
    // -------------------------------------------------------------------------

    pub fn get_intranet_ipv4_cidrs(&self) -> StoreResult<Vec<String>> {
        self.read_string_table("intranet_ipv4_cidrs")
    }

    pub fn set_intranet_ipv4_cidrs(&self, values: &[String]) -> StoreResult<()> {
        self.replace_string_table("intranet_ipv4_cidrs", values)
    }

    pub fn reset_intranet_ipv4_cidrs(&self) -> StoreResult<Vec<String>> {
        let defaults: Vec<String> = DEFAULT_INTRANET_IPV4_CIDRS
            .iter()
            .map(|s| (*s).to_string())
            .collect();
        self.set_intranet_ipv4_cidrs(&defaults)?;
        Ok(defaults)
    }

    pub fn get_intranet_ipv6_cidrs(&self) -> StoreResult<Vec<String>> {
        self.read_string_table("intranet_ipv6_cidrs")
    }

    pub fn set_intranet_ipv6_cidrs(&self, values: &[String]) -> StoreResult<()> {
        self.replace_string_table("intranet_ipv6_cidrs", values)
    }

    pub fn reset_intranet_ipv6_cidrs(&self) -> StoreResult<Vec<String>> {
        let defaults: Vec<String> = DEFAULT_INTRANET_IPV6_CIDRS
            .iter()
            .map(|s| (*s).to_string())
            .collect();
        self.set_intranet_ipv6_cidrs(&defaults)?;
        Ok(defaults)
    }

    // -------------------------------------------------------------------------
    // Shared helpers
    // -------------------------------------------------------------------------

    fn read_u32_table(&self, table: &str, col: &str) -> StoreResult<Vec<u32>> {
        let conn = self.connect()?;
        let sql = format!("SELECT DISTINCT {col} FROM {table} ORDER BY {col}");
        let mut stmt = conn
            .prepare(&sql)
            .map_err(|e| format!("prepare {table} failed: {e}"))?;
        let rows = stmt
            .query_map([], |r| r.get::<_, u32>(0))
            .map_err(|e| format!("query {table} failed: {e}"))?;
        Ok(rows.flatten().collect())
    }

    fn replace_u32_table(&self, table: &str, col: &str, items: &[u32]) -> StoreResult<()> {
        let conn = self.connect()?;
        conn.execute_batch("BEGIN IMMEDIATE")
            .map_err(|e| format!("begin failed: {e}"))?;
        let res = (|| -> StoreResult<()> {
            conn.execute(&format!("DELETE FROM {table}"), [])
                .map_err(|e| format!("clear {table} failed: {e}"))?;
            let mut seen = HashSet::new();
            for &uid in items {
                if seen.insert(uid) {
                    conn.execute(
                        &format!("INSERT OR IGNORE INTO {table} ({col}) VALUES (?1)"),
                        [uid],
                    )
                    .map_err(|e| format!("insert {table} failed: {e}"))?;
                }
            }
            Ok(())
        })();
        finish_tx(&conn, res)
    }

    fn read_string_table(&self, table: &str) -> StoreResult<Vec<String>> {
        let conn = self.connect()?;
        let sql = format!("SELECT value FROM {table} ORDER BY id ASC");
        let mut stmt = conn
            .prepare(&sql)
            .map_err(|e| format!("prepare {table} failed: {e}"))?;
        let rows = stmt
            .query_map([], |r| r.get::<_, String>(0))
            .map_err(|e| format!("query {table} failed: {e}"))?;
        let mut seen = HashSet::new();
        let mut out = Vec::new();
        for val in rows.flatten() {
            let trimmed = val.trim().to_string();
            if !trimmed.is_empty() && seen.insert(trimmed.clone()) {
                out.push(trimmed);
            }
        }
        Ok(out)
    }

    fn replace_string_table(&self, table: &str, items: &[String]) -> StoreResult<()> {
        let conn = self.connect()?;
        conn.execute_batch("BEGIN IMMEDIATE")
            .map_err(|e| format!("begin failed: {e}"))?;
        let res = (|| -> StoreResult<()> {
            conn.execute(&format!("DELETE FROM {table}"), [])
                .map_err(|e| format!("clear {table} failed: {e}"))?;
            let mut seen = HashSet::new();
            for item in items {
                let trimmed = item.trim();
                if !trimmed.is_empty() && seen.insert(trimmed.to_string()) {
                    conn.execute(
                        &format!("INSERT INTO {table} (value) VALUES (?1)"),
                        [trimmed],
                    )
                    .map_err(|e| format!("insert {table} failed: {e}"))?;
                }
            }
            Ok(())
        })();
        finish_tx(&conn, res)
    }
}

fn finish_tx(conn: &Connection, res: StoreResult<()>) -> StoreResult<()> {
    match res {
        Ok(()) => conn
            .execute_batch("COMMIT")
            .map_err(|e| format!("commit failed: {e}")),
        Err(err) => {
            let _ = conn.execute_batch("ROLLBACK");
            Err(err)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn full_crud_roundtrip_across_all_table_groups() {
        let dir = tempdir().unwrap();
        let repo = BoxDbRepo::open(dir.path().join("box.db"), dir.path()).unwrap();

        let mut profile = repo.get_runtime_profile().unwrap();
        assert_eq!(profile.core_name, "honk");
        profile.core_name = "meow".into();
        profile.boot_auto_start = true;
        repo.save_runtime_profile(&profile).unwrap();
        assert_eq!(repo.get_runtime_profile().unwrap().core_name, "meow");

        repo.set_config_override("mihomo", "override_enabled", "1")
            .unwrap();
        assert_eq!(
            repo.get_config_overrides("mihomo")
                .unwrap()
                .get("override_enabled")
                .map(String::as_str),
            Some("1")
        );

        repo.set_app_selection(&[10123, 10456, 10123]).unwrap();
        assert_eq!(repo.get_app_selection().unwrap(), vec![10123, 10456]);

        repo.set_cnip_force_apps(&["0:com.example".into()], &[10999])
            .unwrap();
        assert_eq!(repo.get_cnip_force_uids().unwrap(), vec![10999]);

        let gid = repo
            .upsert_network_rule_group(&NetworkRuleGroup {
                id: 0,
                name: "Work".into(),
                enabled: true,
                priority: 1,
                expression: r#"{"kind":"leaf","type":"wifi_ssid","value":"Corp"}"#.into(),
                actions: vec![NetworkRuleAction {
                    id: 0,
                    group_id: 0,
                    sort_order: 0,
                    action_type: "stop".into(),
                    target: String::new(),
                    continue_on_error: false,
                }],
            })
            .unwrap();
        assert_eq!(repo.list_network_rule_groups().unwrap().len(), 1);
        repo.delete_network_rule_group(gid).unwrap();
        assert!(repo.list_network_rule_groups().unwrap().is_empty());

        let v4 = repo.reset_intranet_ipv4_cidrs().unwrap();
        assert_eq!(v4.len(), 16);
        let v6 = repo.reset_intranet_ipv6_cidrs().unwrap();
        assert_eq!(v6.len(), 12);
    }
}
