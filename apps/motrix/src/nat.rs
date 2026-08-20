use std::{net::SocketAddrV4, num::NonZeroU16, sync::Mutex};

use portmapper::{Client, Config, Protocol};
use serde::Serialize;

use crate::config::{AppConfig, NatProtocol};

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NatStatus {
    pub enabled: bool,
    pub state: NatState,
    pub tcp_external_address: Option<String>,
    pub udp_external_address: Option<String>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum NatState {
    #[default]
    Disabled,
    Starting,
    Mapping,
    Mapped,
    Error,
}

impl Default for NatStatus {
    fn default() -> Self {
        Self {
            enabled: false,
            state: NatState::Disabled,
            tcp_external_address: None,
            udp_external_address: None,
        }
    }
}

struct Clients {
    protocol: NatProtocol,
    tcp: Client,
    udp: Client,
    listen_port: u16,
}

#[derive(Default)]
pub struct NatManager {
    clients: Mutex<Option<Clients>>,
}

impl NatManager {
    /// Synchronize the long-running mapping clients with persisted settings.
    /// `portmapper` owns renewal and network-change handling internally.
    pub fn sync(&self, config: &AppConfig) {
        let Ok(mut slot) = self.clients.lock() else {
            return;
        };
        if !config.nat_enabled {
            if let Some(clients) = slot.take() {
                clients.tcp.deactivate();
                clients.udp.deactivate();
            }
            return;
        }
        let recreate = slot
            .as_ref()
            .is_none_or(|clients| clients.protocol != config.nat_protocol);
        if recreate {
            let mapping_config = |protocol| protocol_config(config.nat_protocol, protocol);
            *slot = Some(Clients {
                protocol: config.nat_protocol,
                tcp: Client::new(mapping_config(Protocol::Tcp)),
                udp: Client::new(mapping_config(Protocol::Udp)),
                listen_port: 0,
            });
        }
        let Some(clients) = slot.as_mut() else { return };
        if clients.listen_port != config.listen_port {
            let port = NonZeroU16::new(config.listen_port).expect("validated listen port");
            clients.tcp.update_local_port(port);
            clients.udp.update_local_port(port);
            clients.listen_port = config.listen_port;
        }
    }

    pub fn status(&self, configured_enabled: bool) -> NatStatus {
        if !configured_enabled {
            return NatStatus {
                enabled: false,
                state: NatState::Disabled,
                ..NatStatus::default()
            };
        }
        let Ok(slot) = self.clients.lock() else {
            return NatStatus {
                enabled: true,
                state: NatState::Error,
                ..NatStatus::default()
            };
        };
        let Some(clients) = slot.as_ref() else {
            return NatStatus {
                enabled: true,
                state: NatState::Starting,
                ..NatStatus::default()
            };
        };
        let tcp = external_address(&clients.tcp);
        let udp = external_address(&clients.udp);
        NatStatus {
            enabled: true,
            state: if tcp.is_some() || udp.is_some() {
                NatState::Mapped
            } else {
                NatState::Mapping
            },
            tcp_external_address: tcp,
            udp_external_address: udp,
        }
    }
}

fn protocol_config(preferred: NatProtocol, protocol: Protocol) -> Config {
    Config {
        enable_upnp: matches!(preferred, NatProtocol::Auto | NatProtocol::Upnp),
        enable_pcp: matches!(preferred, NatProtocol::Auto | NatProtocol::Pcp),
        enable_nat_pmp: matches!(preferred, NatProtocol::Auto | NatProtocol::NatPmp),
        protocol,
    }
}

fn external_address(client: &Client) -> Option<String> {
    client
        .watch_external_address()
        .borrow()
        .as_ref()
        .map(SocketAddrV4::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preferred_protocol_enables_only_the_requested_mapper() {
        let config = protocol_config(NatProtocol::Pcp, Protocol::Tcp);
        assert!(config.enable_pcp);
        assert!(!config.enable_nat_pmp);
        assert!(!config.enable_upnp);

        let config = protocol_config(NatProtocol::Auto, Protocol::Udp);
        assert!(config.enable_pcp && config.enable_nat_pmp && config.enable_upnp);
    }
}
