#[derive(Debug, Clone)]
pub struct NetworkInterfaceStats {
    pub name: String,
    pub rx_bytes_per_sec: u64,
    pub tx_bytes_per_sec: u64,
    pub ip_local: Option<String>,
}

#[derive(Debug, Clone)]
pub struct NetworkStats {
    pub interfaces: Vec<NetworkInterfaceStats>,
    pub active_interface: Option<String>,
    pub ping_ms: Option<f32>,
    pub ping_host: String,
}

impl Default for NetworkStats {
    fn default() -> Self {
        Self {
            interfaces: vec![],
            active_interface: None,
            ping_ms: None,
            ping_host: "1.1.1.1".to_string(),
        }
    }
}

/// Requires a previous snapshot to compute differential throughput.
pub fn get_network_stats(previous: &NetworkStats) -> NetworkStats {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_ping_host() {
        assert_eq!(NetworkStats::default().ping_host, "1.1.1.1");
    }
}
