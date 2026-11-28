use super::*;

pub(super) fn ensure_schema(conn: &Connection) -> Result<()> {
    boxproxy_schema::ensure_schema_v15(conn)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrates_version_one_schema_once_and_updates_user_version() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "
            CREATE TABLE runtime_profile (id INTEGER PRIMARY KEY);
            CREATE TABLE cnip_settings (id INTEGER PRIMARY KEY);
            PRAGMA user_version = 1;
            ",
        )
        .unwrap();

        ensure_schema(&conn).unwrap();

        let version: i64 = conn
            .query_row("PRAGMA user_version", [], |row| row.get(0))
            .unwrap();
        assert_eq!(version, boxproxy_schema::SCHEMA_VERSION);
        let profile_columns = boxproxy_schema::table_columns(&conn, "runtime_profile").unwrap();
        assert!(profile_columns.iter().any(|column| column == "taskset_cpu"));
        let cnip_columns = boxproxy_schema::table_columns(&conn, "cnip_settings").unwrap();
        assert!(cnip_columns.iter().any(|column| column == "cnip_mode"));
        assert!(boxproxy_schema::table_exists(&conn, "network_rule_groups"));
        assert!(boxproxy_schema::table_exists(&conn, "shared_network_settings"));
    }
}
