use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use futures_util::{SinkExt, StreamExt};
use k256::schnorr::signature::Signer;
use k256::schnorr::SigningKey;
use sha2::{Digest, Sha256};
use tokio::sync::{mpsc, Mutex};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

use crate::crypto::{decrypt, encrypt};
use crate::protocol::NetworkPacket;

pub struct NostrRelayClient {
    room_key: [u8; 32],
    topic_hash: String,
    signing_key: SigningKey,
    pubkey_hex: String,
    outbound_tx: mpsc::UnboundedSender<Vec<u8>>,
}

impl NostrRelayClient {
    pub fn start(
        room_secret: &str,
        incoming_tx: mpsc::UnboundedSender<NetworkPacket>,
    ) -> Arc<Self> {
        let room_key = crate::crypto::derive_key(room_secret);

        // Derive deterministic 32-byte topic hash from room secret
        let mut hasher = Sha256::new();
        hasher.update(b"tiktik-nostr-topic-v1:");
        hasher.update(room_secret.as_bytes());
        let topic_hash: String = hasher.finalize().iter().map(|b| format!("{:02x}", b)).collect();

        // Generate ephemeral random Schnorr keypair for this session
        let mut rng = rand::thread_rng();
        let signing_key = SigningKey::random(&mut rng);
        let verifying_key = signing_key.verifying_key();
        let pubkey_bytes = verifying_key.to_bytes();
        let pubkey_hex: String = pubkey_bytes.iter().map(|b| format!("{:02x}", b)).collect();

        let (outbound_tx, outbound_rx) = mpsc::unbounded_channel::<Vec<u8>>();

        let client = Arc::new(Self {
            room_key,
            topic_hash: topic_hash.clone(),
            signing_key,
            pubkey_hex: pubkey_hex.clone(),
            outbound_tx,
        });

        // Spawn background worker connecting to public relays
        let worker_client = Arc::clone(&client);
        tokio::spawn(async move {
            worker_client.run_relay_loop(outbound_rx, incoming_tx).await;
        });

        client
    }

    /// Broadcasts an encrypted raw packet over the Nostr relay network.
    pub fn send_packet(&self, packet: &NetworkPacket) -> Result<(), String> {
        let json_bytes = packet.to_bytes()?;
        let encrypted_payload = encrypt(&self.room_key, &json_bytes)?;
        let _ = self.outbound_tx.send(encrypted_payload);
        Ok(())
    }

    async fn run_relay_loop(
        self: Arc<Self>,
        outbound_rx: mpsc::UnboundedReceiver<Vec<u8>>,
        incoming_tx: mpsc::UnboundedSender<NetworkPacket>,
    ) {
        // High-availability public relays
        let relays = [
            "wss://relay.damus.io",
            "wss://nos.lol",
            "wss://relay.primal.net",
        ];

        let rx_arc = Arc::new(Mutex::new(outbound_rx));

        for relay_url in relays {
            let rx_clone = Arc::clone(&rx_arc);
            let in_clone = incoming_tx.clone();
            let self_clone = Arc::clone(&self);
            let url = relay_url.to_string();

            tokio::spawn(async move {
                self_clone.connect_and_serve(&url, rx_clone, in_clone).await;
            });
        }
    }

    async fn connect_and_serve(
        &self,
        relay_url: &str,
        outbound_rx: Arc<Mutex<mpsc::UnboundedReceiver<Vec<u8>>>>,
        incoming_tx: mpsc::UnboundedSender<NetworkPacket>,
    ) {
        loop {
            match connect_async(relay_url).await {
                Ok((ws_stream, _)) => {
                    let (mut write, mut read) = ws_stream.split();

                    // Send Nostr REQ to subscribe to ephemeral events matching our topic hash
                    let req_msg = format!(
                        "[\"REQ\",\"tiktik-sub\",{{\"kinds\":[20000],\"#t\":[\"{}\"]}}]",
                        self.topic_hash
                    );
                    if write.send(Message::Text(req_msg)).await.is_err() {
                        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                        continue;
                    }

                    // Listen and forward
                    let write_arc = Arc::new(Mutex::new(write));

                    loop {
                        tokio::select! {
                            // Incoming message from relay
                            msg = read.next() => {
                                match msg {
                                    Some(Ok(Message::Text(text))) => {
                                        self.handle_relay_message(&text, &incoming_tx);
                                    }
                                    Some(Ok(Message::Ping(p))) => {
                                        let mut w = write_arc.lock().await;
                                        let _ = w.send(Message::Pong(p)).await;
                                    }
                                    Some(Ok(Message::Close(_))) | None => {
                                        break; // Reconnect
                                    }
                                    _ => {}
                                }
                            }

                            // Outbound packet to send to relay
                            Some(payload) = async {
                                let mut rx = outbound_rx.lock().await;
                                rx.recv().await
                            } => {
                                if let Ok(event_json) = self.build_nostr_event(&payload) {
                                    let mut w = write_arc.lock().await;
                                    let _ = w.send(Message::Text(event_json)).await;
                                }
                            }
                        }
                    }
                }
                Err(_) => {
                    // Silent retry after 5 seconds
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                }
            }
        }
    }

    fn handle_relay_message(
        &self,
        raw_json: &str,
        incoming_tx: &mpsc::UnboundedSender<NetworkPacket>,
    ) {
        // Nostr format: ["EVENT", "sub_id", { event_object }]
        if !raw_json.starts_with("[\"EVENT\"") {
            return;
        }

        if let Ok(v) = serde_json::from_str::<serde_json::Value>(raw_json) {
            if let Some(event) = v.get(2) {
                if let Some(content_hex) = event.get("content").and_then(|c| c.as_str()) {
                    // Decode hex to encrypted payload
                    if content_hex.len() % 2 == 0 {
                        let mut payload = Vec::with_capacity(content_hex.len() / 2);
                        for i in (0..content_hex.len()).step_by(2) {
                            if let Ok(b) = u8::from_str_radix(&content_hex[i..i + 2], 16) {
                                payload.push(b);
                            }
                        }

                        // Decrypt AEAD payload using room key
                        if let Ok(decrypted_bytes) = decrypt(&self.room_key, &payload) {
                            if let Ok(packet) = NetworkPacket::from_bytes(&decrypted_bytes) {
                                let _ = incoming_tx.send(packet);
                            }
                        }
                    }
                }
            }
        }
    }

    fn build_nostr_event(&self, encrypted_payload: &[u8]) -> Result<String, String> {
        let content_hex: String = encrypted_payload.iter().map(|b| format!("{:02x}", b)).collect();
        let now_sec = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        // Serialized array for BIP-340 id: [0, pubkey, created_at, kind, tags, content]
        let serialized_for_id = serde_json::json!([
            0,
            self.pubkey_hex,
            now_sec,
            20000,
            [["t", self.topic_hash]],
            content_hex
        ])
        .to_string();

        let mut hasher = Sha256::new();
        hasher.update(serialized_for_id.as_bytes());
        let id_bytes = hasher.finalize();
        let id_hex: String = id_bytes.iter().map(|b| format!("{:02x}", b)).collect();

        // Sign using BIP-340 Schnorr signature
        let sig = self.signing_key.sign(&id_bytes);
        let sig_bytes = sig.to_bytes();
        let sig_hex: String = sig_bytes.iter().map(|b| format!("{:02x}", b)).collect();

        let event = serde_json::json!([
            "EVENT",
            {
                "id": id_hex,
                "pubkey": self.pubkey_hex,
                "created_at": now_sec,
                "kind": 20000,
                "tags": [["t", self.topic_hash]],
                "content": content_hex,
                "sig": sig_hex
            }
        ]);

        Ok(event.to_string())
    }
}
