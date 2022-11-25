use ipdisserver::Signature;
use std::{net::Ipv4Addr, path::PathBuf};

pub const SCANNER_PORT_DEFAULT: u16 = 1902;
pub const BROADCAST_ADDR_DEFAULT: Ipv4Addr = Ipv4Addr::BROADCAST; // 255.255.255.255
pub const EXTRA_SIGNATURE_DEFAULT: &str = "pang-supremacy-maritime-revoke-afterglow"; // compatibility with original ipdiscan
pub const SCAN_PERIOD_DEFAULT: f64 = 1.0;

#[derive(Clone, Debug, PartialEq)]
pub struct ScannerConfig {
    pub port: u16,
    pub scan_period: f64,
    pub broadcast_addr: Ipv4Addr,
    pub target_port: u16,
    pub signatures: Vec<Signature>,
    pub log_file: Option<PathBuf>,
}
