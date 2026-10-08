use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::mpsc;

use crate::crypto::{decrypt, encrypt};
use crate::protocol::{ChatMessage, NetworkPacket};

pub struct NetworkManager {
    socket: Arc<UdpSocket>,
    room_key: [u8; 32],
    peer_addr: Option<SocketAddr>,
    pub bound_port: u16,
    pub base_port: u16,
    my_nick: String,
    pub node_id: String,
}

impl NetworkManager {
    pub async fn bind(
        bind_port: u16,
        broadcast_port: u16,
        peer_addr: Option<SocketAddr>,
        room_secret: &str,
        my_nick: String,
    ) -> Result<Self, String> {
        let mut bound_socket = None;
        let mut actual_port = bind_port;

        // Try requested port, then cycle through next 10 ports automatically
        for port in bind_port..=(bind_port + 10) {
            let bind_addr = format!("0.0.0.0:{}", port);
            if let Ok(s) = UdpSocket::bind(&bind_addr).await {
                let _ = s.set_broadcast(true);
                bound_socket = Some(s);
                actual_port = port;
                break;
            }
        }

        let socket = bound_socket.ok_or_else(|| {
            format!("Failed to bind any UDP port in range {}..={}", bind_port, bind_port + 10)
        })?;

        let room_key = crate::crypto::derive_key(room_secret);
        let node_id = format!("{:08x}", rand::random::<u32>());

        Ok(Self {
            socket: Arc::new(socket),
            room_key,
            peer_addr,
            bound_port: actual_port,
            base_port: broadcast_port,
            my_nick,
            node_id,
        })
    }

    #[allow(dead_code)]
    pub fn local_addr(&self) -> SocketAddr {
        self.socket.local_addr().unwrap_or_else(|_| "0.0.0.0:0".parse().unwrap())
    }

    pub fn room_key(&self) -> &[u8; 32] {
        &self.room_key
    }

    /// Spawns the incoming packet listener loop.
    pub fn start_receiving(
        &self,
        incoming_tx: mpsc::UnboundedSender<NetworkPacket>,
    ) {
        let socket = Arc::clone(&self.socket);
        let room_key = self.room_key;
        let my_node_id = self.node_id.clone();
        let my_nick = self.my_nick.clone();

        tokio::spawn(async move {
            let mut buf = vec![0u8; 65535];
            loop {
                match socket.recv_from(&mut buf).await {
                    Ok((len, _from)) => {
                        let payload = &buf[..len];
                        // Decrypt using AEAD room key
                        if let Ok(decrypted_bytes) = decrypt(&room_key, payload) {
                            if let Ok(packet) = NetworkPacket::from_bytes(&decrypted_bytes) {
                                match &packet {
                                    NetworkPacket::Chat(msg) => {
                                        if msg.sender != my_nick {
                                            let _ = incoming_tx.send(packet);
                                        }
                                    }
                                    NetworkPacket::Presence { node_id, .. } => {
                                        // Ignore our own presence ping
                                        if node_id != &my_node_id {
                                            let _ = incoming_tx.send(packet);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(_) => {
                        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    }
                }
            }
        });
    }

    /// Broadcasts an encrypted raw packet to the LAN and localhost port range.
    async fn broadcast_encrypted(&self, encrypted_payload: &[u8]) -> Result<(), String> {
        if let Some(target) = self.peer_addr {
            let _ = self.socket.send_to(encrypted_payload, target).await;
            return Ok(());
        }

        // 1. Broadcast to LAN across sibling ports (e.g. 9944..=9948)
        for p in self.base_port..=(self.base_port + 4) {
            let bcast_target = format!("255.255.255.255:{}", p);
            let _ = self.socket.send_to(encrypted_payload, &bcast_target).await;
        }

        // 2. Loopback to localhost sibling ports (handles multiple instances on the same machine)
        for p in self.base_port..=(self.base_port + 4) {
            if p != self.bound_port {
                let loopback_target = format!("127.0.0.1:{}", p);
                let _ = self.socket.send_to(encrypted_payload, &loopback_target).await;
            }
        }

        Ok(())
    }

    /// Encrypts and transmits a chat message.
    pub async fn send_message(&self, msg: &ChatMessage) -> Result<(), String> {
        let packet = NetworkPacket::Chat(msg.clone());
        let json_bytes = packet.to_bytes()?;
        let encrypted_payload = encrypt(&self.room_key, &json_bytes)?;
        self.broadcast_encrypted(&encrypted_payload).await
    }

    /// Broadcasts an encrypted presence heartbeat ping.
    pub async fn send_presence(&self) -> Result<(), String> {
        let packet = NetworkPacket::Presence {
            node_id: self.node_id.clone(),
            nick: self.my_nick.clone(),
        };
        let json_bytes = packet.to_bytes()?;
        let encrypted_payload = encrypt(&self.room_key, &json_bytes)?;
        self.broadcast_encrypted(&encrypted_payload).await
    }
}
