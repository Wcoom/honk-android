//! Idempotent SQLite DDL & v0..v15 migration engine for `box.db`.
//!
//! Guarantees zero data loss when opening a legacy v1..v3 `boxctl` database (`wifi_match_*`,
//! `hotspot_*`) OR an existing v15 `com.boxproxy.honk` database.

use crate::defaults::{
    DEFAULT_CNIP_IPV4_URL, DEFAULT_CNIP_IPV6_URL, DEFAULT_INTRANET_IPV4_CIDRS,
    DEFAULT_INTRANET_IPV6_CIDRS, DEFAULT_SHARED_NETWORK_INTERFACES,
};
use crate::{SchemaResult, SCHEMA_VERSION};
use rusqlite::{params, Connection};
use std::path::Path;

/// Ensure all 21 v15 tables and columns exist and migrate legacy rows transactionally.
pub fn ensure_schema_v15(conn: &Connection) -> SchemaResult<()> {
    let current: i64 = conn
        .query_row("PRAGMA user_version", [], |row| row.get(0))
        .map_err(|err| format!("read database schema version failed: {err}"))?;

    if current >= SCHEMA_VERSION && all_v15_tables_present(conn) {
        return Ok(());
    }

    conn.execute_batch("BEGIN IMMEDIATE")
        .map_err(|err| format!("begin database schema migration failed: {err}"))?;

    let migration = (|| -> SchemaResult<()> {
        create_all_v15_tables(conn)?;
        ensure_upgraded_columns(conn)?;
        migrate_legacy_wifi_tables(conn)?;
        migrate_legacy_hotspot_tables(conn)?;
        let target_version = current.max(SCHEMA_VERSION);
        conn.execute_batch(&format!("PRAGMA user_version = {target_version}"))
            .map_err(|err| format!("update database schema version failed: {err}"))?;
        Ok(())
    })();

    match migration {
        Ok(()) => conn
            .execute_batch("COMMIT")
            .map_err(|err| format!("commit database schema migration failed: {err}")),
        Err(err) => {
            let _ = conn.execute_batch("ROLLBACK");
            Err(err)
        }
    }
}

/// Seed default singleton rows and default intranet/shared-network lists when missing.
pub fn seed_default_rows(conn: &Connection, box_home: &Path) -> SchemaResult<()> {
    ensure_schema_v15(conn)?;

    let cnip_v4_file = box_home
        .join("run/state/cnip/cn_ip.txt")
        .to_string_lossy()
        .to_string();
    let cnip_v6_file = box_home
        .join("run/state/cnip/cn_ipv6.txt")
        .to_string_lossy()
        .to_string();

    conn.execute_batch("BEGIN IMMEDIATE")
        .map_err(|err| format!("begin default seed failed: {err}"))?;

    let seed = (|| -> SchemaResult<()> {
        // 1. runtime_profile singleton
        let has_profile: i64 = conn
            .query_row("SELECT COUNT(*) FROM runtime_profile WHERE id = 1", [], |r| {
                r.get(0)
            })
            .unwrap_or(0);
        if has_profile == 0 {
            let cols = table_columns(conn, "runtime_profile")?;
            let has_legacy = cols.iter().any(|c| c.eq_ignore_ascii_case("tproxy_port"));
            if has_legacy {
                conn.execute(
                    "INSERT INTO runtime_profile (
                        id, core_name, mode, proxy_mode, auto_sync_config, performance_mode,
                        clean_vendor_firewall, ipv6_mode, config_name,
                        tproxy_port, redir_port, quic, mihomo_dns_forward, mihomo_dns_port,
                        proxy_tcp, proxy_udp, dns_hijack_tcp, dns_hijack_udp, dns_hijack_mode,
                        cgroup_memcg, memcg_limit, taskset_cpu, allow_cpu, cgroup_blkio, weight,
                        bypass_cn, tun_device, fake_ip_range, fake_ip6_range, boot_auto_start
                    ) VALUES (
                        1, 'honk', 'ebpf', 'core', 1, 0,
                        0, 'enable', 'config.dae',
                        '7893', '7892', 'enable', 'enable', '1053',
                        1, 1, 1, 1, 'tproxy',
                        0, '100M', 0, '0-7', 0, '',
                        0, '', '', '', 0
                    )",
                    [],
                )
                .map_err(|err| format!("seed legacy runtime_profile failed: {err}"))?;
            } else {
                conn.execute(
                    "INSERT INTO runtime_profile (
                        id, core_name, mode, proxy_mode, auto_sync_config, performance_mode,
                        clean_vendor_firewall, ipv6_mode, config_name, quic, mihomo_dns_forward,
                        proxy_tcp, proxy_udp, dns_hijack_tcp, dns_hijack_udp, dns_hijack_mode,
                        cgroup_memcg, memcg_limit, taskset_cpu, allow_cpu, cgroup_blkio, weight,
                        bypass_cn, boot_auto_start
                    ) VALUES (
                        1, 'honk', 'ebpf', 'core', 1, 0,
                        0, 'enable', 'config.dae', 'enable', 'enable',
                        1, 1, 1, 1, 'tproxy',
                        0, '100M', 0, '0-7', 0, '',
                        0, 0
                    )",
                    [],
                )
                .map_err(|err| format!("seed runtime_profile failed: {err}"))?;
            }
        }

        // 2. cnip_settings singleton
        conn.execute(
            "INSERT OR IGNORE INTO cnip_settings (
                id, bypass_cnip, cnip_mode, bypass_ipv4, bypass_ipv6,
                ipv4_file, ipv4_url, ipv6_file, ipv6_url,
                ipv4_source, ipv6_source, ipv4_additions, ipv6_additions
            ) VALUES (
                1, 0, 'ipset', 1, 0,
                ?1, ?2, ?3, ?4,
                'url', 'url', '', ''
            )",
            params![
                cnip_v4_file,
                DEFAULT_CNIP_IPV4_URL,
                cnip_v6_file,
                DEFAULT_CNIP_IPV6_URL
            ],
        )
        .map_err(|err| format!("seed cnip_settings failed: {err}"))?;

        // 3. network_match_settings singleton
        conn.execute(
            "INSERT OR IGNORE INTO network_match_settings (
                id, network_control_enabled, use_on_wifi_disconnect, use_on_wifi_connect,
                network_enable_on_unmatched, network_enable_on_matched,
                matched_action, unmatched_action,
                enable_ssid_matching, enable_network_control_log, list_mode,
                switch_config, matched_config_name, unmatched_config_name,
                matched_script_name, unmatched_script_name
            ) VALUES (
                1, 0, 1, 1,
                1, 1,
                'enable', 'stop',
                0, 1, 'blacklist',
                0, '', '', '', ''
            )",
            [],
        )
        .map_err(|err| format!("seed network_match_settings failed: {err}"))?;

        // 4. shared_network_settings singleton
        conn.execute(
            "INSERT OR IGNORE INTO shared_network_settings (id, mac_filter, mac_mode)
             VALUES (1, 0, 'blacklist')",
            [],
        )
        .map_err(|err| format!("seed shared_network_settings failed: {err}"))?;

        // Clean up any legacy iptables wildcard interfaces (e.g. "ap+") rejected by boxctl validation.
        let _ = conn.execute(
            "DELETE FROM shared_network_interfaces WHERE value LIKE '%+%'",
            [],
        );

        // 5. shared_network_interface_settings + default AP interfaces
        let shared_init: i64 = conn
            .query_row(
                "SELECT initialized FROM shared_network_interface_settings WHERE id = 1",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);
        if shared_init == 0 {
            let count: i64 = conn
                .query_row("SELECT COUNT(*) FROM shared_network_interfaces", [], |r| {
                    r.get(0)
                })
                .unwrap_or(0);
            if count == 0 {
                for iface in DEFAULT_SHARED_NETWORK_INTERFACES {
                    conn.execute(
                        "INSERT INTO shared_network_interfaces (value) VALUES (?1)",
                        [iface],
                    )
                    .map_err(|err| format!("seed shared_network_interfaces failed: {err}"))?;
                }
            }
            conn.execute(
                "INSERT OR REPLACE INTO shared_network_interface_settings (id, initialized)
                 VALUES (1, 1)",
                [],
            )
            .map_err(|err| format!("seed shared_network_interface_settings failed: {err}"))?;
        }

        // 6. intranet_bypass_settings + 16 IPv4 / 12 IPv6 default CIDRs
        let intranet_init: i64 = conn
            .query_row(
                "SELECT initialized FROM intranet_bypass_settings WHERE id = 1",
                [],
                |r| r.get(0),
            )
            .unwrap_or(0);
        if intranet_init == 0 {
            let v4_count: i64 = conn
                .query_row("SELECT COUNT(*) FROM intranet_ipv4_cidrs", [], |r| r.get(0))
                .unwrap_or(0);
            if v4_count == 0 {
                for cidr in DEFAULT_INTRANET_IPV4_CIDRS {
                    conn.execute("INSERT INTO intranet_ipv4_cidrs (value) VALUES (?1)", [cidr])
                        .map_err(|err| format!("seed intranet_ipv4_cidrs failed: {err}"))?;
                }
            }
            let v6_count: i64 = conn
                .query_row("SELECT COUNT(*) FROM intranet_ipv6_cidrs", [], |r| r.get(0))
                .unwrap_or(0);
            if v6_count == 0 {
                for cidr in DEFAULT_INTRANET_IPV6_CIDRS {
                    conn.execute("INSERT INTO intranet_ipv6_cidrs (value) VALUES (?1)", [cidr])
                        .map_err(|err| format!("seed intranet_ipv6_cidrs failed: {err}"))?;
                }
            }
            conn.execute(
                "INSERT OR REPLACE INTO intranet_bypass_settings (id, initialized)
                 VALUES (1, 1)",
                [],
            )
            .map_err(|err| format!("seed intranet_bypass_settings failed: {err}"))?;
        }

        Ok(())
    })();

    match seed {
        Ok(()) => conn
            .execute_batch("COMMIT")
            .map_err(|err| format!("commit default seed failed: {err}")),
        Err(err) => {
            let _ = conn.execute_batch("ROLLBACK");
            Err(err)
        }
    }
}

const V15_TABLES: &[&str] = &[
    "runtime_profile",
    "app_selection",
    "app_settings",
    "app_gid_list",
    "blocked_interfaces",
    "cnip_force_apps",
    "cnip_force_uids",
    "cnip_settings",
    "intranet_bypass_settings",
    "intranet_ipv4_cidrs",
    "intranet_ipv6_cidrs",
    "network_match_bssids",
    "network_match_conditions",
    "network_match_settings",
    "network_match_ssids",
    "network_rule_actions",
    "network_rule_groups",
    "shared_network_interface_settings",
    "shared_network_interfaces",
    "shared_network_macs",
    "shared_network_settings",
];

fn all_v15_tables_present(conn: &Connection) -> bool {
    V15_TABLES.iter().all(|table| table_exists(conn, table))
}

fn create_all_v15_tables(conn: &Connection) -> SchemaResult<()> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS runtime_profile (
            id INTEGER PRIMARY KEY CHECK(id = 1),
            core_name TEXT NOT NULL DEFAULT 'honk',
            mode TEXT NOT NULL DEFAULT 'ebpf',
            proxy_mode TEXT NOT NULL DEFAULT 'core',
            auto_sync_config INTEGER NOT NULL DEFAULT 1,
            performance_mode INTEGER NOT NULL DEFAULT 0,
            clean_vendor_firewall INTEGER NOT NULL DEFAULT 0,
            ipv6_mode TEXT NOT NULL DEFAULT 'enable',
            config_name TEXT NOT NULL DEFAULT '',
            quic TEXT NOT NULL DEFAULT 'enable',
            mihomo_dns_forward TEXT NOT NULL DEFAULT 'enable',
            proxy_tcp INTEGER NOT NULL DEFAULT 1,
            proxy_udp INTEGER NOT NULL DEFAULT 1,
            dns_hijack_tcp INTEGER NOT NULL DEFAULT 1,
            dns_hijack_udp INTEGER NOT NULL DEFAULT 1,
            dns_hijack_mode TEXT NOT NULL DEFAULT 'tproxy',
            cgroup_memcg INTEGER NOT NULL DEFAULT 0,
            memcg_limit TEXT NOT NULL DEFAULT '100M',
            taskset_cpu INTEGER NOT NULL DEFAULT 0,
            allow_cpu TEXT NOT NULL DEFAULT '0-7',
            cgroup_blkio INTEGER NOT NULL DEFAULT 0,
            weight TEXT NOT NULL DEFAULT '',
            bypass_cn INTEGER NOT NULL DEFAULT 0,
            boot_auto_start INTEGER NOT NULL DEFAULT 0
        );
        CREATE TABLE IF NOT EXISTS app_selection (
            uid INTEGER PRIMARY KEY
        );
        CREATE TABLE IF NOT EXISTS app_settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS app_gid_list (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            value TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS blocked_interfaces (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            value TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS cnip_force_apps (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            value TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS cnip_force_uids (
            uid INTEGER PRIMARY KEY
        );
        CREATE TABLE IF NOT EXISTS cnip_settings (
            id INTEGER PRIMARY KEY CHECK(id = 1),
            bypass_cnip INTEGER NOT NULL DEFAULT 0,
            cnip_mode TEXT NOT NULL DEFAULT 'ipset',
            bypass_ipv4 INTEGER NOT NULL DEFAULT 1,
            bypass_ipv6 INTEGER NOT NULL DEFAULT 0,
            ipv4_file TEXT NOT NULL DEFAULT '',
            ipv4_url TEXT NOT NULL DEFAULT '',
            ipv6_file TEXT NOT NULL DEFAULT '',
            ipv6_url TEXT NOT NULL DEFAULT '',
            ipv4_source TEXT NOT NULL DEFAULT 'url',
            ipv6_source TEXT NOT NULL DEFAULT 'url',
            ipv4_additions TEXT NOT NULL DEFAULT '',
            ipv6_additions TEXT NOT NULL DEFAULT ''
        );
        CREATE TABLE IF NOT EXISTS intranet_bypass_settings (
            id INTEGER PRIMARY KEY CHECK(id = 1),
            initialized INTEGER NOT NULL DEFAULT 0
        );
        CREATE TABLE IF NOT EXISTS intranet_ipv4_cidrs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            value TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS intranet_ipv6_cidrs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            value TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS network_match_bssids (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            value TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS network_match_conditions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            type TEXT NOT NULL,
            value TEXT NOT NULL DEFAULT '',
            enabled INTEGER NOT NULL DEFAULT 1,
            sort_order INTEGER NOT NULL DEFAULT 0
        );
        CREATE TABLE IF NOT EXISTS network_match_settings (
            id INTEGER PRIMARY KEY CHECK(id = 1),
            network_control_enabled INTEGER NOT NULL DEFAULT 0,
            use_on_wifi_disconnect INTEGER NOT NULL DEFAULT 1,
            use_on_wifi_connect INTEGER NOT NULL DEFAULT 1,
            network_enable_on_unmatched INTEGER NOT NULL DEFAULT 1,
            network_enable_on_matched INTEGER NOT NULL DEFAULT 1,
            matched_action TEXT NOT NULL DEFAULT 'enable',
            unmatched_action TEXT NOT NULL DEFAULT 'stop',
            enable_ssid_matching INTEGER NOT NULL DEFAULT 0,
            enable_network_control_log INTEGER NOT NULL DEFAULT 1,
            list_mode TEXT NOT NULL DEFAULT 'blacklist',
            switch_config INTEGER NOT NULL DEFAULT 0,
            matched_config_name TEXT NOT NULL DEFAULT '',
            unmatched_config_name TEXT NOT NULL DEFAULT '',
            matched_script_name TEXT NOT NULL DEFAULT '',
            unmatched_script_name TEXT NOT NULL DEFAULT ''
        );
        CREATE TABLE IF NOT EXISTS network_match_ssids (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            value TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS network_rule_actions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            group_id INTEGER NOT NULL,
            sort_order INTEGER NOT NULL DEFAULT 0,
            type TEXT NOT NULL,
            target TEXT NOT NULL DEFAULT '',
            continue_on_error INTEGER NOT NULL DEFAULT 0
        );
        CREATE TABLE IF NOT EXISTS network_rule_groups (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL DEFAULT '',
            enabled INTEGER NOT NULL DEFAULT 1,
            priority INTEGER NOT NULL DEFAULT 0,
            expression TEXT NOT NULL DEFAULT ''
        );
        CREATE TABLE IF NOT EXISTS shared_network_interface_settings (
            id INTEGER PRIMARY KEY CHECK(id = 1),
            initialized INTEGER NOT NULL DEFAULT 0
        );
        CREATE TABLE IF NOT EXISTS shared_network_interfaces (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            value TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS shared_network_macs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            value TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS shared_network_settings (
            id INTEGER PRIMARY KEY CHECK(id = 1),
            mac_filter INTEGER NOT NULL DEFAULT 0,
            mac_mode TEXT NOT NULL DEFAULT 'blacklist'
        );
        "#,
    )
    .map_err(|err| format!("create v15 schema tables failed: {err}"))
}

fn ensure_upgraded_columns(conn: &Connection) -> SchemaResult<()> {
    ensure_columns(
        conn,
        "runtime_profile",
        &[
            ("boot_auto_start", "INTEGER NOT NULL DEFAULT 0"),
            ("clean_vendor_firewall", "INTEGER NOT NULL DEFAULT 0"),
            ("ipv6_mode", "TEXT NOT NULL DEFAULT 'enable'"),
            ("taskset_cpu", "INTEGER NOT NULL DEFAULT 0"),
        ],
    )?;
    ensure_columns(
        conn,
        "cnip_settings",
        &[
            ("cnip_mode", "TEXT NOT NULL DEFAULT 'ipset'"),
            ("ipv4_source", "TEXT NOT NULL DEFAULT 'url'"),
            ("ipv6_source", "TEXT NOT NULL DEFAULT 'url'"),
            ("ipv4_additions", "TEXT NOT NULL DEFAULT ''"),
            ("ipv6_additions", "TEXT NOT NULL DEFAULT ''"),
        ],
    )?;
    ensure_columns(
        conn,
        "network_match_settings",
        &[
            ("network_enable_on_unmatched", "INTEGER NOT NULL DEFAULT 1"),
            ("network_enable_on_matched", "INTEGER NOT NULL DEFAULT 1"),
            ("matched_action", "TEXT NOT NULL DEFAULT 'enable'"),
            ("unmatched_action", "TEXT NOT NULL DEFAULT 'stop'"),
            ("switch_config", "INTEGER NOT NULL DEFAULT 0"),
            ("matched_config_name", "TEXT NOT NULL DEFAULT ''"),
            ("unmatched_config_name", "TEXT NOT NULL DEFAULT ''"),
            ("matched_script_name", "TEXT NOT NULL DEFAULT ''"),
            ("unmatched_script_name", "TEXT NOT NULL DEFAULT ''"),
        ],
    )?;
    Ok(())
}

fn migrate_legacy_wifi_tables(conn: &Connection) -> SchemaResult<()> {
    if table_exists(conn, "wifi_match_settings") {
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM network_match_settings", [], |r| r.get(0))
            .unwrap_or(0);
        if count == 0 {
            let _ = conn.execute(
                "INSERT OR IGNORE INTO network_match_settings (
                    id, network_control_enabled, use_on_wifi_disconnect, use_on_wifi_connect,
                    enable_ssid_matching, enable_network_control_log, list_mode
                 )
                 SELECT id, network_control_enabled, use_on_wifi_disconnect, use_on_wifi_connect,
                        enable_ssid_matching, enable_network_control_log, list_mode
                 FROM wifi_match_settings",
                [],
            );
        }
    }
    if table_exists(conn, "wifi_match_ssids") {
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM network_match_ssids", [], |r| r.get(0))
            .unwrap_or(0);
        if count == 0 {
            let _ = conn.execute(
                "INSERT INTO network_match_ssids (value) SELECT value FROM wifi_match_ssids",
                [],
            );
        }
    }
    if table_exists(conn, "wifi_match_bssids") {
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM network_match_bssids", [], |r| r.get(0))
            .unwrap_or(0);
        if count == 0 {
            let _ = conn.execute(
                "INSERT INTO network_match_bssids (value) SELECT value FROM wifi_match_bssids",
                [],
            );
        }
    }
    Ok(())
}

fn migrate_legacy_hotspot_tables(conn: &Connection) -> SchemaResult<()> {
    if table_exists(conn, "hotspot_settings") {
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM shared_network_settings", [], |r| r.get(0))
            .unwrap_or(0);
        if count == 0 {
            let _ = conn.execute(
                "INSERT OR IGNORE INTO shared_network_settings (id, mac_filter, mac_mode)
                 SELECT id, mac_filter, mac_mode FROM hotspot_settings",
                [],
            );
        }
    }
    if table_exists(conn, "hotspot_ap_interfaces") {
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM shared_network_interfaces", [], |r| r.get(0))
            .unwrap_or(0);
        if count == 0 {
            let _ = conn.execute(
                "INSERT INTO shared_network_interfaces (value) SELECT value FROM hotspot_ap_interfaces",
                [],
            );
        }
    }
    if table_exists(conn, "hotspot_macs") {
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM shared_network_macs", [], |r| r.get(0))
            .unwrap_or(0);
        if count == 0 {
            let _ = conn.execute(
                "INSERT INTO shared_network_macs (value) SELECT value FROM hotspot_macs",
                [],
            );
        }
    }
    Ok(())
}

pub fn table_exists(conn: &Connection, table: &str) -> bool {
    conn.query_row(
        "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ?1 LIMIT 1",
        [table],
        |_| Ok(()),
    )
    .is_ok()
}

pub fn table_columns(conn: &Connection, table: &str) -> SchemaResult<Vec<String>> {
    let pragma = format!("PRAGMA table_info({table})");
    let mut stmt = conn
        .prepare(&pragma)
        .map_err(|err| format!("read {table} schema failed: {err}"))?;
    let columns = stmt
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(|err| format!("read {table} schema failed: {err}"))?;

    columns
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|err| format!("read {table} schema failed: {err}"))
}

fn ensure_columns(conn: &Connection, table: &str, columns: &[(&str, &str)]) -> SchemaResult<()> {
    let existing = table_columns(conn, table)?;
    for (column, definition) in columns {
        if existing.iter().any(|c| c.eq_ignore_ascii_case(column)) {
            continue;
        }
        let sql = format!("ALTER TABLE {table} ADD COLUMN {column} {definition}");
        conn.execute(&sql, [])
            .map_err(|err| format!("update {table}.{column} failed: {err}"))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{
        match_first_rule_group, CoreKind, NetworkObservation, NetworkRuleAction, NetworkRuleGroup,
    };

    #[test]
    fn migrates_legacy_v1_and_v3_boxctl_db_to_v15_preserving_data() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "
            CREATE TABLE runtime_profile (id INTEGER PRIMARY KEY);
            CREATE TABLE cnip_settings (id INTEGER PRIMARY KEY);
            CREATE TABLE wifi_match_settings (
                id INTEGER PRIMARY KEY,
                network_control_enabled INTEGER NOT NULL,
                use_on_wifi_disconnect INTEGER NOT NULL,
                use_on_wifi_connect INTEGER NOT NULL,
                enable_ssid_matching INTEGER NOT NULL,
                enable_network_control_log INTEGER NOT NULL,
                list_mode TEXT NOT NULL
            );
            INSERT INTO wifi_match_settings VALUES (1, 1, 0, 1, 1, 1, 'whitelist');
            CREATE TABLE wifi_match_ssids (id INTEGER PRIMARY KEY AUTOINCREMENT, value TEXT NOT NULL);
            INSERT INTO wifi_match_ssids (value) VALUES ('OfficeWiFi');
            CREATE TABLE hotspot_settings (id INTEGER PRIMARY KEY, mac_filter INTEGER NOT NULL, mac_mode TEXT NOT NULL);
            INSERT INTO hotspot_settings VALUES (1, 1, 'whitelist');
            CREATE TABLE hotspot_ap_interfaces (id INTEGER PRIMARY KEY AUTOINCREMENT, value TEXT NOT NULL);
            INSERT INTO hotspot_ap_interfaces (value) VALUES ('ap0');
            CREATE TABLE hotspot_macs (id INTEGER PRIMARY KEY AUTOINCREMENT, value TEXT NOT NULL);
            INSERT INTO hotspot_macs (value) VALUES ('AA:BB:CC:DD:EE:FF');
            PRAGMA user_version = 1;
            ",
        )
        .unwrap();

        ensure_schema_v15(&conn).unwrap();

        let version: i64 = conn
            .query_row("PRAGMA user_version", [], |r| r.get(0))
            .unwrap();
        assert_eq!(version, 15);
        assert!(all_v15_tables_present(&conn));

        let ssid: String = conn
            .query_row("SELECT value FROM network_match_ssids LIMIT 1", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(ssid, "OfficeWiFi");

        let list_mode: String = conn
            .query_row("SELECT list_mode FROM network_match_settings WHERE id = 1", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(list_mode, "whitelist");

        let ap: String = conn
            .query_row(
                "SELECT value FROM shared_network_interfaces LIMIT 1",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(ap, "ap0");

        let mac: String = conn
            .query_row("SELECT value FROM shared_network_macs LIMIT 1", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(mac, "AA:BB:CC:DD:EE:FF");
    }

    #[test]
    fn seeds_defaults_idempotently() {
        let conn = Connection::open_in_memory().unwrap();
        seed_default_rows(&conn, Path::new("/data/user/0/io.honk.android/files/box")).unwrap();
        seed_default_rows(&conn, Path::new("/data/user/0/io.honk.android/files/box")).unwrap();

        let v4: i64 = conn
            .query_row("SELECT COUNT(*) FROM intranet_ipv4_cidrs", [], |r| r.get(0))
            .unwrap();
        let v6: i64 = conn
            .query_row("SELECT COUNT(*) FROM intranet_ipv6_cidrs", [], |r| r.get(0))
            .unwrap();
        assert_eq!(v4, 16);
        assert_eq!(v6, 12);
    }

    #[test]
    fn evaluates_network_rule_ast_by_priority() {
        let g1 = NetworkRuleGroup {
            id: 1,
            name: "Home WiFi".into(),
            enabled: true,
            priority: 10,
            expression: r#"{"kind":"op","op":"and","children":[{"kind":"leaf","type":"wifi","value":""},{"kind":"leaf","type":"wifi_ssid","value":"Home5G"}]}"#.into(),
            actions: vec![NetworkRuleAction {
                id: 1,
                group_id: 1,
                sort_order: 0,
                action_type: "stop".into(),
                target: String::new(),
                continue_on_error: false,
            }],
        };
        let g2 = NetworkRuleGroup {
            id: 2,
            name: "SIM2 Cellular".into(),
            enabled: true,
            priority: 5,
            expression: r#"{"kind":"op","op":"and","children":[{"kind":"leaf","type":"cellular","value":""},{"kind":"leaf","type":"sim2","value":"46001"}]}"#.into(),
            actions: vec![],
        };

        let obs_wifi = NetworkObservation {
            wifi_connected: true,
            wifi_ssid: "Home5G".into(),
            ..Default::default()
        };
        assert_eq!(
            match_first_rule_group(&[g1.clone(), g2.clone()], &obs_wifi).map(|g| g.id),
            Some(1)
        );

        let obs_cell = NetworkObservation {
            cellular_connected: true,
            active_sim_slot: Some(2),
            sim2_mcc_mnc: "46001".into(),
            ..Default::default()
        };
        assert_eq!(
            match_first_rule_group(&[g1, g2], &obs_cell).map(|g| g.id),
            Some(2)
        );
    }

    #[test]
    fn core_kind_supports_all_seven_cores_including_meow() {
        assert_eq!(CoreKind::ALL.len(), 7);
        assert_eq!(CoreKind::parse("meow").unwrap(), CoreKind::Meow);
        assert!(CoreKind::Meow.supports_ebpf());
        assert!(CoreKind::Honk.prefers_honk_native_api());
        assert!(!CoreKind::Mihomo.prefers_honk_native_api());
    }
}
