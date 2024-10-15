use std::{
    fs,
    net::IpAddr,
    path::{Path, PathBuf},
    str::FromStr,
};

use crate::ip_list::IpList;

/// Unit struct that ensures the ip list file doesn't exist before a test and is cleaned up after a
/// test.
struct CleanupIpList {
    path: PathBuf,
}

impl CleanupIpList {
    /// Creates a new instance that deletes an existing ip_list file if it exists.
    pub fn new<P: AsRef<Path>>(path: P) -> Self {
        if path.as_ref().exists() {
            fs::remove_file(&path).expect("Failed to cleanup IpList file.");
        }

        assert!(!path.as_ref().exists());

        CleanupIpList {
            path: path.as_ref().to_path_buf(),
        }
    }
}

impl Drop for CleanupIpList {
    fn drop(&mut self) {
        if !self.path.exists() {
            return;
        }

        fs::remove_file(&self.path).expect("Failed to cleanup IpList file.");
    }
}

/// Tests that if the file doesn't exist, a new one is correctly created.
#[test]
pub fn create_new_list() {
    let path = Path::new("ip_list.test.create_new_list.toml");
    let _cleanup = CleanupIpList::new(path);

    let ip_list1 = IpList::load_or_create(path).expect("failed to load or create ip_list.");
    assert!(path.exists());

    let contents = fs::read_to_string(path).expect("failed to read file");
    let ip_list2: IpList = toml::from_str(&contents).expect("failed to parse contents");

    let expected_ips: &[IpAddr] = &[];
    assert_eq!(expected_ips, ip_list1.get_blocked_ips());
    assert_eq!(expected_ips, ip_list2.get_blocked_ips());
    assert_eq!(expected_ips, ip_list1.get_trusted_ips());
    assert_eq!(expected_ips, ip_list2.get_trusted_ips());
}

/// Tests that if the file exists, then it is loaded correctly.
#[test]
pub fn load_list() {
    let path = Path::new("ip_list.test.load_list.toml");
    let _cleanup = CleanupIpList::new(path);

    fs::write(
        path,
        r#"
	blocked_ips = ["255.255.255.255", "ffff:ffff:ffff:ffff:ffff:ffff:ffff:ffff"]
	trusted_ips = ["127.0.0.1", "0:0:0:0:0:0:0:1"]"#,
    )
    .expect("failed to write example file");
    assert!(path.exists());

    let ip_list = IpList::load_or_create(path).expect("failed to load or create ip_list.");

    let expected_blocked_ips: &[IpAddr] = &[
        IpAddr::from_str("255.255.255.255").unwrap(),
        IpAddr::from_str("ffff:ffff:ffff:ffff:ffff:ffff:ffff:ffff").unwrap(),
    ];
    let expected_trusted_ips: &[IpAddr] = &[
        IpAddr::from_str("127.0.0.1").unwrap(),
        IpAddr::from_str("0:0:0:0:0:0:0:1").unwrap(),
    ];
    assert_eq!(expected_blocked_ips, ip_list.get_blocked_ips());
    assert_eq!(expected_trusted_ips, ip_list.get_trusted_ips());
}

/// Tests that when an IP is blocked the file is correctly updated.
#[test]
pub fn save_on_block() {
    let path = Path::new("ip_list.test.save_on_block.toml");
    let _cleanup = CleanupIpList::new(path);

    let mut ip_list1 = IpList::load_or_create(path).expect("failed to load or create ip_list.");
    assert!(path.exists());

    let ip1 = IpAddr::from_str("255.255.255.255").expect("Failed to parse IP Addr");
    ip_list1.block_untrusted(ip1).expect("failed to block IP");
    assert!(ip_list1.is_blocked(&ip1));

    let contents = fs::read_to_string(path).expect("failed to read file");
    let ip_list2: IpList = toml::from_str(&contents).expect("failed to parse contents");

    let expected_ips: &[IpAddr] = &[ip1];
    assert_eq!(expected_ips, ip_list1.get_blocked_ips());
    assert_eq!(expected_ips, ip_list2.get_blocked_ips());
}

/// Tests that when an IP is trusted the file is correctly updated.
#[test]
pub fn save_on_trust() {
    let path = Path::new("ip_list.test.save_on_trust.toml");
    let _cleanup = CleanupIpList::new(path);

    let mut ip_list1 = IpList::load_or_create(path).expect("failed to load or create ip_list.");
    assert!(path.exists());

    let ip1 = IpAddr::from_str("255.255.255.255").expect("Failed to parse IP Addr");
    ip_list1.trust_unblocked(ip1).expect("failed to block IP");
    assert!(ip_list1.is_trusted(&ip1));

    let contents = fs::read_to_string(path).expect("failed to read file");
    let ip_list2: IpList = toml::from_str(&contents).expect("failed to parse contents");

    let expected_ips: &[IpAddr] = &[ip1];
    assert_eq!(expected_ips, ip_list1.get_trusted_ips());
    assert_eq!(expected_ips, ip_list2.get_trusted_ips());
}
