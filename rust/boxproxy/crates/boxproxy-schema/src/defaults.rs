//! Canonical default seed values matching `com.boxproxy.honk` v214.

pub const DEFAULT_CNIP_IPV4_URL: &str =
    "https://github.com/Hackl0us/GeoIP2-CN/raw/release/CN-ip-cidr.txt";
pub const DEFAULT_CNIP_IPV6_URL: &str = "https://ispip.clang.cn/all_cn_ipv6.txt";

/// Exact 16 default IPv4 intranet bypass CIDRs from `com.boxproxy.honk` v214.
pub const DEFAULT_INTRANET_IPV4_CIDRS: &[&str] = &[
    "0.0.0.0/8",
    "10.0.0.0/8",
    "100.64.0.0/10",
    "127.0.0.0/8",
    "169.254.0.0/16",
    "172.16.0.0/12",
    "192.0.0.0/24",
    "192.0.2.0/24",
    "192.88.99.0/24",
    "192.168.0.0/16",
    "198.18.0.0/15",
    "198.51.100.0/24",
    "203.0.113.0/24",
    "224.0.0.0/4",
    "240.0.0.0/4",
    "255.255.255.255/32",
];

/// Exact 12 default IPv6 intranet bypass CIDRs from `com.boxproxy.honk` v214.
pub const DEFAULT_INTRANET_IPV6_CIDRS: &[&str] = &[
    "::/128",
    "::1/128",
    "::ffff:0:0/96",
    "64:ff9b::/96",
    "100::/64",
    "2001::/32",
    "2001:10::/28",
    "2001:20::/28",
    "2001:db8::/32",
    "2002::/16",
    "fc00::/7",
    "fe80::/10",
];

/// Default shared/tethering AP interfaces (empty by default, matching live `com.boxproxy.honk` v15 `box.db`).
pub const DEFAULT_SHARED_NETWORK_INTERFACES: &[&str] = &[];

/// All 30 `config_override.<core>.<option>` keys stored in `app_settings`.
pub const CONFIG_OVERRIDE_OPTIONS: &[&str] = &[
    // Base & Mode (13)
    "override_enabled",
    "override_dns",
    "override_tun",
    "override_tproxy",
    "override_redir",
    "override_external_controller",
    "override_secret",
    "override_external_ui",
    "override_ipv6",
    "override_unified_delay",
    "override_tcp_concurrent",
    "override_find_process_mode",
    "override_log_level",
    // Ports (4)
    "tproxy_port",
    "redir_port",
    "mixed_port",
    "dns_listen",
    // TUN (6)
    "tun_stack",
    "tun_device",
    "tun_auto_route",
    "tun_strict_route",
    "tun_auto_detect_interface",
    "tun_mtu",
    // DNS / FakeIP (4)
    "dns_enhanced_mode",
    "fake_ip_range",
    "fake_ip6_range",
    "dns_prefer_h3",
    // Sniffer / Geo / Scripts (3)
    "sniffer_enable",
    "geodata_mode",
    "scripts_enabled",
];

pub fn config_override_key(core: &str, option: &str) -> String {
    format!("config_override.{core}.{option}")
}
