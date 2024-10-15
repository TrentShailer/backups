use std::{net::IpAddr, str::FromStr};

use crate::ip_list::IpList;

#[test]
/// Tests if blocking an untrusted, unblocked IP, adds them to the blocklist.
pub fn block_untrusted_unblocked() {
    let mut ip_list = IpList::new_unbacked();
    let ip1 = IpAddr::from_str("255.255.255.255").expect("Failed to parse IP Addr");
    let ip2 = IpAddr::from_str("125.125.125.125").expect("Failed to parse IP Addr");

    assert!(!ip_list.is_blocked(&ip1));
    assert!(!ip_list.is_blocked(&ip2));

    ip_list.block_untrusted(ip1).expect("Failed to block IP");

    assert!(ip_list.is_blocked(&ip1));
    assert!(!ip_list.is_blocked(&ip2));

    ip_list.block_untrusted(ip2).expect("Failed to block IP");

    assert!(ip_list.is_blocked(&ip1));
    assert!(ip_list.is_blocked(&ip2));
}

#[test]
/// Tests if blocking a blocked IP does not add them to the blocklist.
pub fn block_untrusted_blocked() {
    let mut ip_list = IpList::new_unbacked();
    let ip1 = IpAddr::from_str("255.255.255.255").expect("Failed to parse IP Addr");

    assert!(!ip_list.is_blocked(&ip1));
    ip_list.block_untrusted(ip1).expect("Failed to block IP");
    assert!(ip_list.is_blocked(&ip1));

    ip_list.block_untrusted(ip1).expect("Failed to block IP");
    assert_eq!(1, ip_list.get_blocked_ips().len());
}

#[test]
/// Tests if blocking a trusted IP does not add them to the blocklist.
pub fn block_trusted_unblocked() {
    let mut ip_list = IpList::new_unbacked();
    let ip1 = IpAddr::from_str("255.255.255.255").expect("Failed to parse IP Addr");

    ip_list.trust_unblocked(ip1).expect("Failed to trust IP");
    assert!(!ip_list.is_blocked(&ip1));
    assert!(ip_list.is_trusted(&ip1));

    ip_list.block_untrusted(ip1).expect("Failed to block IP");
    assert!(!ip_list.is_blocked(&ip1));
    assert!(ip_list.is_trusted(&ip1));
}

#[test]
/// Tests if trusting an untrusted, unblocked IP, adds them to the trustlist.
pub fn trust_untrusted_unblocked() {
    let mut ip_list = IpList::new_unbacked();
    let ip1 = IpAddr::from_str("255.255.255.255").expect("Failed to parse IP Addr");
    let ip2 = IpAddr::from_str("125.125.125.125").expect("Failed to parse IP Addr");

    assert!(!ip_list.is_trusted(&ip1));
    assert!(!ip_list.is_trusted(&ip2));

    ip_list.trust_unblocked(ip1).expect("Failed to trust IP");

    assert!(ip_list.is_trusted(&ip1));
    assert!(!ip_list.is_trusted(&ip2));

    ip_list.trust_unblocked(ip2).expect("Failed to trust IP");

    assert!(ip_list.is_trusted(&ip1));
    assert!(ip_list.is_trusted(&ip2));
}

#[test]
/// Tests if trusting a trusted IP does not add them to the trustlist.
pub fn trust_trusted_unblocked() {
    let mut ip_list = IpList::new_unbacked();
    let ip1 = IpAddr::from_str("255.255.255.255").expect("Failed to parse IP Addr");

    assert!(!ip_list.is_trusted(&ip1));
    ip_list.trust_unblocked(ip1).expect("Failed to trust IP");
    assert!(ip_list.is_trusted(&ip1));

    ip_list.trust_unblocked(ip1).expect("Failed to trust IP");
    assert_eq!(1, ip_list.get_trusted_ips().len());
}

#[test]
/// Tests if trusted a blocked IP does not add them to the trustlist.
pub fn trust_untrusted_blocked() {
    let mut ip_list = IpList::new_unbacked();
    let ip1 = IpAddr::from_str("255.255.255.255").expect("Failed to parse IP Addr");

    ip_list.block_untrusted(ip1).expect("Failed to block IP");
    assert!(ip_list.is_blocked(&ip1));
    assert!(!ip_list.is_trusted(&ip1));

    ip_list.trust_unblocked(ip1).expect("Failed to trust IP");
    assert!(ip_list.is_blocked(&ip1));
    assert!(!ip_list.is_trusted(&ip1));
}
