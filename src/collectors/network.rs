use crate::collectors::{async_trait, Collector, Snapshot};
use crate::platform::{api::RawNetStat, current};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct NetworkMetrics {
    pub timestamp: i64,
    pub interface: String,
    pub topology_hint: String,
    pub family_hint: String,
    pub medium_hint: String,
    pub rx_bytes_sec: u64,
    pub tx_bytes_sec: u64,
    pub rx_packets_sec: u64,
    pub tx_packets_sec: u64,
    pub rx_errors: u64,
    pub tx_errors: u64,
    pub rx_dropped: u64,
    pub tx_dropped: u64,
    pub connections_total: u32,
    pub connections_established: u32,
    pub tcp_syn_sent: u32,
    pub tcp_syn_recv: u32,
    pub tcp_fin_wait1: u32,
    pub tcp_fin_wait2: u32,
    pub tcp_time_wait: u32,
    pub tcp_close: u32,
    pub tcp_close_wait: u32,
    pub tcp_last_ack: u32,
    pub tcp_listen: u32,
    pub tcp_closing: u32,
    pub tcp_other: u32,
    pub udp_total: u32,
    pub udp_established: u32,
    pub udp_close: u32,
    pub udp_other: u32,
    pub retrans_segs: u64,
    pub tcp_state_breakdown_supported: bool,
    pub udp_breakdown_supported: bool,
    pub retrans_supported: bool,
}

pub struct NetworkCollector {
    prev: std::collections::HashMap<String, RawNetStat>,
    prev_at: Option<Instant>,
}

impl NetworkCollector {
    pub fn new() -> Self {
        Self {
            prev: std::collections::HashMap::new(),
            prev_at: None,
        }
    }
}

impl Default for NetworkCollector {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Collector for NetworkCollector {
    fn name(&self) -> &'static str {
        "network"
    }

    async fn collect(&mut self, snapshot: &mut Snapshot) -> Result<()> {
        snapshot.networks = collect_network(self)?;
        Ok(())
    }
}

fn collect_network(c: &mut NetworkCollector) -> Result<Vec<NetworkMetrics>> {
    let interfaces = current::read_network()?;
    let connections = current::read_net_connections();
    let now = chrono::Utc::now().timestamp();
    let collected_at = Instant::now();
    let elapsed_secs = c
        .prev_at
        .map(|prev| collected_at.saturating_duration_since(prev).as_secs_f64())
        .filter(|secs| *secs > 0.0);
    let mut results = Vec::new();

    for iface in interfaces {
        let identity = classify_network_identity(&iface.interface);
        let (rx_bytes_sec, tx_bytes_sec, rx_packets_sec, tx_packets_sec) = if let (
            Some(prev),
            Some(elapsed_secs),
        ) =
            (c.prev.get(&iface.interface), elapsed_secs)
        {
            (
                per_second_u64(iface.rx_bytes.saturating_sub(prev.rx_bytes), elapsed_secs),
                per_second_u64(iface.tx_bytes.saturating_sub(prev.tx_bytes), elapsed_secs),
                per_second_u64(
                    iface.rx_packets.saturating_sub(prev.rx_packets),
                    elapsed_secs,
                ),
                per_second_u64(
                    iface.tx_packets.saturating_sub(prev.tx_packets),
                    elapsed_secs,
                ),
            )
        } else {
            (0, 0, 0, 0)
        };

        results.push(NetworkMetrics {
            timestamp: now,
            interface: iface.interface.clone(),
            topology_hint: identity.topology,
            family_hint: identity.family,
            medium_hint: identity.medium,
            rx_bytes_sec,
            tx_bytes_sec,
            rx_packets_sec,
            tx_packets_sec,
            rx_errors: iface.rx_errors,
            tx_errors: iface.tx_errors,
            rx_dropped: iface.rx_dropped,
            tx_dropped: iface.tx_dropped,
            connections_total: connections.total,
            connections_established: connections.established,
            tcp_syn_sent: connections.tcp_syn_sent,
            tcp_syn_recv: connections.tcp_syn_recv,
            tcp_fin_wait1: connections.tcp_fin_wait1,
            tcp_fin_wait2: connections.tcp_fin_wait2,
            tcp_time_wait: connections.tcp_time_wait,
            tcp_close: connections.tcp_close,
            tcp_close_wait: connections.tcp_close_wait,
            tcp_last_ack: connections.tcp_last_ack,
            tcp_listen: connections.tcp_listen,
            tcp_closing: connections.tcp_closing,
            tcp_other: connections.tcp_other,
            udp_total: connections.udp_total,
            udp_established: connections.udp_established,
            udp_close: connections.udp_close,
            udp_other: connections.udp_other,
            retrans_segs: connections.retrans_segs,
            tcp_state_breakdown_supported: connections.tcp_state_breakdown_supported,
            udp_breakdown_supported: connections.udp_breakdown_supported,
            retrans_supported: connections.retrans_supported,
        });

        c.prev.insert(iface.interface.clone(), iface);
    }

    c.prev_at = Some(collected_at);
    Ok(results)
}

#[derive(Debug)]
struct NetworkIdentity {
    topology: String,
    family: String,
    medium: String,
}

fn classify_network_identity(interface: &str) -> NetworkIdentity {
    let lowered = interface.to_ascii_lowercase();

    let topology = if lowered == "lo" || lowered.starts_with("lo") {
        "loopback"
    } else if lowered.starts_with("docker")
        || lowered.starts_with("br-")
        || lowered.starts_with("veth")
        || lowered.starts_with("virbr")
        || lowered.starts_with("vmnet")
        || lowered.starts_with("vboxnet")
    {
        "bridge-virtual"
    } else if lowered.starts_with("tun")
        || lowered.starts_with("tap")
        || lowered.starts_with("wg")
        || lowered.starts_with("utun")
    {
        "tunnel"
    } else if lowered.starts_with("bond") || lowered.starts_with("team") {
        "aggregate"
    } else if lowered.contains("vpn") {
        "vpn"
    } else {
        "host-interface"
    };

    let family = if lowered == "lo" || lowered.starts_with("lo") {
        "local"
    } else if lowered.starts_with("wl")
        || lowered.starts_with("wlan")
        || lowered.starts_with("wi-fi")
        || lowered.starts_with("wifi")
        || lowered.starts_with("en0")
    {
        "wireless"
    } else if lowered.starts_with("tun")
        || lowered.starts_with("tap")
        || lowered.starts_with("wg")
        || lowered.starts_with("utun")
    {
        "overlay"
    } else if lowered.starts_with("docker")
        || lowered.starts_with("br-")
        || lowered.starts_with("veth")
        || lowered.starts_with("virbr")
    {
        "container"
    } else {
        "ethernet"
    };

    let medium = if family == "wireless" {
        "wifi"
    } else if topology == "loopback" {
        "software"
    } else if topology == "tunnel" || family == "overlay" || topology == "vpn" {
        "overlay"
    } else if topology == "bridge-virtual" || family == "container" {
        "virtual"
    } else {
        "wired"
    };

    NetworkIdentity {
        topology: topology.to_string(),
        family: family.to_string(),
        medium: medium.to_string(),
    }
}

fn per_second_u64(delta: u64, elapsed_secs: f64) -> u64 {
    if elapsed_secs <= 0.0 {
        0
    } else {
        (delta as f64 / elapsed_secs).round() as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn per_second_scaling_respects_elapsed_time() {
        let per_second = per_second_u64(600, 2.0);
        assert_eq!(per_second, 300);
    }

    #[tokio::test]
    async fn network_collector_runs_and_produces_sane_values_on_linux() {
        let mut collector = NetworkCollector::new();
        let mut snapshot = Snapshot::default();

        collector.collect(&mut snapshot).await.unwrap();

        for net in snapshot.networks {
            assert!(!net.interface.is_empty());
            assert!(net.connections_established <= net.connections_total);
        }
    }
}
