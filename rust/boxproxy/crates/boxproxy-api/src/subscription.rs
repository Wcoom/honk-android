//! M14 local `Subscription-Userinfo` header parser.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubscriptionUserInfo {
    pub upload: u64,
    pub download: u64,
    pub total: u64,
    pub expire: u64,
}

impl SubscriptionUserInfo {
    pub fn used(&self) -> u64 {
        self.upload.saturating_add(self.download)
    }

    pub fn remaining(&self) -> u64 {
        self.total.saturating_sub(self.used())
    }
}

pub fn parse_subscription_userinfo(header: &str) -> SubscriptionUserInfo {
    let mut info = SubscriptionUserInfo::default();
    for part in header.split(';') {
        let Some((k, v)) = part.split_once('=') else {
            continue;
        };
        let Ok(num) = v.trim().parse::<u64>() else {
            continue;
        };
        match k.trim().to_ascii_lowercase().as_str() {
            "upload" => info.upload = num,
            "download" => info.download = num,
            "total" => info.total = num,
            "expire" => info.expire = num,
            _ => {}
        }
    }
    info
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_subscription_userinfo_header() {
        let info = parse_subscription_userinfo(
            "upload=1024; download=2048; total=10240; expire=1800000000",
        );
        assert_eq!(info.used(), 3072);
        assert_eq!(info.remaining(), 7168);
        assert_eq!(info.expire, 1_800_000_000);
    }
}
