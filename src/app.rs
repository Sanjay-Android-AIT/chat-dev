use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::disguise::{DisguiseEngine, PresetMode};
use crate::network::NetworkManager;
use crate::protocol::ChatMessage;
use crate::stego::StegoEngine;

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum ViewMode {
    ServerLogs, // Disguised as server stdout/stderr
    CleanChat,  // Standard clean terminal chat
}

#[derive(Clone)]
pub struct MessageEntry {
    pub text: String,
    pub created_at: Instant,
    pub is_chat: bool,
}

pub struct App {
    pub nick: String,
    pub room_name: String,
    pub input: String,
    pub cursor_position: usize,
    pub messages: VecDeque<MessageEntry>,
    pub max_messages: usize,
    pub network: Arc<NetworkManager>,
    pub is_panic: bool,
    pub view_mode: ViewMode,
    pub auto_mock_logs: bool,
    pub message_ttl: Option<Duration>,
    pub preset: PresetMode,
    pub last_activity: Instant,
    pub idle_timeout: Duration,
    pub is_ghost_mode: bool,
    pub has_unread: bool,
    pub burn_code: String,
    pub is_burned: bool,
    pub peers: HashMap<String, (String, Instant)>,
    pub seen_message_ids: HashSet<String>,
    pub nostr: Option<Arc<crate::nostr::NostrRelayClient>>,
}

impl App {
    pub fn new(
        nick: String,
        room_name: String,
        network: Arc<NetworkManager>,
        nostr: Option<Arc<crate::nostr::NostrRelayClient>>,
        ttl_seconds: Option<u64>,
        preset: PresetMode,
        burn_code: String,
        idle_timeout_secs: u64,
    ) -> Self {
        let mut app = Self {
            nick,
            room_name,
            input: String::new(),
            cursor_position: 0,
            messages: VecDeque::with_capacity(1000),
            max_messages: 1000,
            network,
            nostr,
            is_panic: false,
            view_mode: ViewMode::ServerLogs,
            auto_mock_logs: true,
            message_ttl: ttl_seconds.map(Duration::from_secs),
            preset,
            last_activity: Instant::now(),
            idle_timeout: Duration::from_secs(idle_timeout_secs),
            is_ghost_mode: false,
            has_unread: false,
            burn_code,
            is_burned: false,
            peers: HashMap::new(),
            seen_message_ids: HashSet::new(),
        };

        // Seed initial disguise logs so it immediately looks like work
        for _ in 0..6 {
            app.add_raw_log(DisguiseEngine::generate_mock_log(preset), false);
        }

        app
    }

    pub fn record_peer_presence(&mut self, node_id: String, nick: String) {
        let is_new = !self.peers.contains_key(&node_id);
        self.peers.insert(node_id.clone(), (nick.clone(), Instant::now()));

        if is_new {
            let note = match self.preset {
                PresetMode::ReactNative => {
                    format!("LOG  [Metro:Mesh] Attached client node @{} (id: {})", nick, &node_id[..6.min(node_id.len())])
                }
                PresetMode::Kotlin => {
                    format!("D/Daemon(18420): Peer node @{} attached (cluster_id: {})", nick, &node_id[..6.min(node_id.len())])
                }
            };
            self.add_raw_log(note, false);
        }
    }

    pub fn prune_offline_peers(&mut self) {
        let timeout = Duration::from_secs(8);
        self.peers.retain(|_, (_, last_seen)| last_seen.elapsed() < timeout);
    }

    pub fn active_node_count(&self) -> usize {
        self.peers.len() + 1 // Include self
    }

    pub fn record_user_activity(&mut self) {
        self.last_activity = Instant::now();
        self.is_ghost_mode = false;
        self.has_unread = false;
    }

    pub fn check_idle_timeout(&mut self) {
        if !self.is_panic && self.last_activity.elapsed() >= self.idle_timeout {
            if !self.is_ghost_mode {
                self.is_ghost_mode = true;
            }
        }
    }

    pub fn add_raw_log(&mut self, text: String, is_chat: bool) {
        if self.messages.len() >= self.max_messages {
            self.messages.pop_front();
        }
        self.messages.push_back(MessageEntry {
            text,
            created_at: Instant::now(),
            is_chat,
        });
    }

    pub fn prune_expired_messages(&mut self) {
        if let Some(ttl) = self.message_ttl {
            let mut removed_count = 0;
            let initial_len = self.messages.len();
            self.messages.retain(|entry| {
                if entry.is_chat && entry.created_at.elapsed() > ttl {
                    removed_count += 1;
                    false
                } else {
                    true
                }
            });

            if removed_count > 0 {
                let note = match self.preset {
                    PresetMode::ReactNative => format!(
                        "LOG  [Metro:Cache] Evicted {} expired bundle delta artifact(s)",
                        removed_count
                    ),
                    PresetMode::Kotlin => format!(
                        "> Task :app:cleanExpiredCache UP-TO-DATE (evicted {} ephemeral objects)",
                        removed_count
                    ),
                };
                self.add_raw_log(note, false);
            }

            if self.messages.is_empty() && initial_len > 0 {
                self.add_raw_log(DisguiseEngine::generate_mock_log(self.preset), false);
            }
        }
    }

    pub fn clear_all_history(&mut self) {
        self.messages.clear();
        for _ in 0..4 {
            self.add_raw_log(DisguiseEngine::generate_mock_log(self.preset), false);
        }
        let restore_note = match self.preset {
            PresetMode::ReactNative => "LOG  [Metro] Bundler ready. Cache flushed.".into(),
            PresetMode::Kotlin => "BUILD SUCCESSFUL in 2s (all tasks cached)".into(),
        };
        self.add_raw_log(restore_note, false);
    }

    pub fn handle_incoming_message(&mut self, msg: ChatMessage) {
        // Deduplicate messages received over multiple ports or channels
        if !self.seen_message_ids.insert(msg.id.clone()) {
            return;
        }

        // Trigger silent visual notification if user is not actively typing
        if self.is_ghost_mode || self.input.is_empty() {
            self.has_unread = true;
        }

        let formatted = match self.view_mode {
            ViewMode::ServerLogs => {
                DisguiseEngine::format_disguised_chat(&msg.sender, &msg.content, self.preset)
            }
            ViewMode::CleanChat => {
                format!("[{}] <{}> {}", msg.timestamp, msg.sender, msg.content)
            }
        };
        self.add_raw_log(formatted, true);
    }

    pub async fn submit_input(&mut self) {
        self.record_user_activity();
        let text = self.input.trim().to_string();
        if text.is_empty() {
            return;
        }

        // Check Duress / Burn Code
        if text == self.burn_code || text == "/burn" {
            self.trigger_burn();
            return;
        }

        // Handle slash commands
        if text.starts_with('/') {
            let parts: Vec<&str> = text.split_whitespace().collect();
            let cmd = parts.get(0).copied().unwrap_or("");

            match cmd {
                "/" | "/help" => {
                    self.add_raw_log("=== TikTik Command Guide & Quick Reference ===".into(), false);
                    self.add_raw_log("  /clear        Instant wipe of all chat history and screen reset".into(), false);
                    self.add_raw_log("  /preset <p>   Switch disguise: '/preset react' (Metro) or '/preset kotlin' (Gradle)".into(), false);
                    self.add_raw_log("  /encode <msg> Steganography: hide encrypted message inside a realistic stack trace".into(), false);
                    self.add_raw_log("  /decode <str> Steganography: decrypt message from a pasted stack trace or token".into(), false);
                    self.add_raw_log("  /ttl <sec>    Set auto-destruct timer (e.g. '/ttl 60' or '/ttl off')".into(), false);
                    self.add_raw_log("  /burn         Duress burn: instantly zeroize RAM and drop into a real 'git status'".into(), false);
                    self.add_raw_log("  /heartbeat    Toggle background realistic log streaming on/off".into(), false);
                    self.add_raw_log("  /panic        Trigger panic screen (or press ESC)".into(), false);
                    self.add_raw_log("  Hotkeys: ESC (Boss Key) | F2 (Toggle Preset) | Tab (Toggle Layout) | Ctrl+L (Clear)".into(), false);
                    self.add_raw_log("==============================================".into(), false);
                }
                "/clear" | "/c" | "/cls" | "/wipe" => {
                    self.clear_all_history();
                }
                "/panic" => self.toggle_panic(),
                "/mode" => self.toggle_mode(),
                "/preset" => {
                    if let Some(target) = parts.get(1) {
                        match *target {
                            "react" | "rn" | "metro" => {
                                self.preset = PresetMode::ReactNative;
                                self.add_raw_log("LOG  [Metro] Switched active profile to React Native (Metro + Hermes)".into(), false);
                            }
                            "kotlin" | "android" | "gradle" => {
                                self.preset = PresetMode::Kotlin;
                                self.add_raw_log("> Task :app:switchProfileKotlin SUCCESS".into(), false);
                            }
                            _ => {
                                self.add_raw_log("[SYSTEM] Usage: /preset react | /preset kotlin".into(), false);
                            }
                        }
                    } else {
                        self.toggle_preset();
                    }
                }
                "/encode" => {
                    let msg_content = text.trim_start_matches("/encode").trim();
                    if msg_content.is_empty() {
                        self.add_raw_log("[STEGO] Usage: /encode <message to hide in stack trace>".into(), false);
                    } else {
                        match StegoEngine::encode_to_stack_trace(self.network.room_key(), &self.nick, msg_content, self.preset) {
                            Ok(trace) => {
                                self.add_raw_log("--- [STEGO PAYLOAD: Copy and paste into Slack / Teams] ---".into(), false);
                                for line in trace.lines() {
                                    self.add_raw_log(line.to_string(), false);
                                }
                                self.add_raw_log("--- [END STEGO PAYLOAD] ---".into(), false);
                            }
                            Err(e) => {
                                self.add_raw_log(format!("[STEGO ERROR] Failed to encode: {}", e), false);
                            }
                        }
                    }
                }
                "/decode" => {
                    let raw_trace = text.trim_start_matches("/decode").trim();
                    if raw_trace.is_empty() {
                        self.add_raw_log("[STEGO] Usage: /decode <paste stack trace containing BuildToken or CrashToken>".into(), false);
                    } else {
                        match StegoEngine::decode_from_stack_trace(self.network.room_key(), raw_trace) {
                            Ok(msg) => {
                                self.handle_incoming_message(msg);
                            }
                            Err(e) => {
                                self.add_raw_log(format!("[STEGO DECODE ERROR] {}", e), false);
                            }
                        }
                    }
                }
                "/heartbeat" => {
                    self.auto_mock_logs = !self.auto_mock_logs;
                    self.add_raw_log(
                        format!("[SYSTEM] Mock telemetry heartbeat: {}", self.auto_mock_logs),
                        false,
                    );
                }
                "/ttl" => {
                    if let Some(sec_str) = parts.get(1) {
                        if *sec_str == "off" || *sec_str == "0" {
                            self.message_ttl = None;
                            self.add_raw_log("[SYSTEM] Message TTL disabled".into(), false);
                        } else if let Ok(secs) = sec_str.parse::<u64>() {
                            self.message_ttl = Some(Duration::from_secs(secs));
                            self.add_raw_log(
                                format!("[SYSTEM] Message TTL set to {} seconds", secs),
                                false,
                            );
                        } else {
                            self.add_raw_log("[SYSTEM] Usage: /ttl <seconds> (e.g. /ttl 60 or /ttl off)".into(), false);
                        }
                    } else {
                        let cur = self.message_ttl.map(|d| format!("{}s", d.as_secs())).unwrap_or_else(|| "off".into());
                        self.add_raw_log(format!("[SYSTEM] Current message TTL: {}", cur), false);
                    }
                }
                _ => {
                    self.add_raw_log(
                        format!("[COMMAND] Unknown: {} (type / or /help)", text),
                        false,
                    );
                }
            }
            self.input.clear();
            self.cursor_position = 0;
            return;
        }

        let msg = ChatMessage::new(&self.nick, &text);
        self.seen_message_ids.insert(msg.id.clone());

        let local_formatted = match self.view_mode {
            ViewMode::ServerLogs => {
                DisguiseEngine::format_disguised_chat(&self.nick, &text, self.preset)
            }
            ViewMode::CleanChat => {
                format!("[{}] <{}> {}", msg.timestamp, self.nick, text)
            }
        };
        self.add_raw_log(local_formatted, true);

        let _ = self.network.send_message(&msg).await;
        if let Some(ref nostr) = self.nostr {
            let _ = nostr.send_packet(&crate::protocol::NetworkPacket::Chat(msg.clone()));
        }

        self.input.clear();
        self.cursor_position = 0;
    }

    pub fn trigger_burn(&mut self) {
        self.is_burned = true;
        self.messages.clear();
        self.input.clear();
    }

    pub fn toggle_panic(&mut self) {
        self.is_panic = !self.is_panic;
    }

    pub fn toggle_mode(&mut self) {
        self.view_mode = match self.view_mode {
            ViewMode::ServerLogs => ViewMode::CleanChat,
            ViewMode::CleanChat => ViewMode::ServerLogs,
        };
    }

    pub fn toggle_preset(&mut self) {
        self.preset = match self.preset {
            PresetMode::ReactNative => PresetMode::Kotlin,
            PresetMode::Kotlin => PresetMode::ReactNative,
        };
        let note = match self.preset {
            PresetMode::ReactNative => "LOG  [Metro] Activated React Native (Metro Bundler) profile",
            PresetMode::Kotlin => "> Task :app:assembleDebug (Activated Kotlin/Android profile)",
        };
        self.add_raw_log(note.into(), false);
    }

    pub fn insert_char(&mut self, c: char) {
        if c == '\n' || c == '\r' {
            return;
        }
        self.record_user_activity();
        let mut chars: Vec<char> = self.input.chars().collect();
        if self.cursor_position > chars.len() {
            self.cursor_position = chars.len();
        }
        chars.insert(self.cursor_position, c);
        self.cursor_position += 1;
        self.input = chars.into_iter().collect();
    }

    pub fn insert_str(&mut self, s: &str) {
        self.record_user_activity();
        // Flatten multi-line pastes into single line so pasted stack traces remain on command line
        let flattened: String = s
            .chars()
            .map(|c| if c == '\n' || c == '\r' { ' ' } else { c })
            .collect();
        let mut chars: Vec<char> = self.input.chars().collect();
        if self.cursor_position > chars.len() {
            self.cursor_position = chars.len();
        }
        let count = flattened.chars().count();
        chars.splice(self.cursor_position..self.cursor_position, flattened.chars());
        self.cursor_position += count;
        self.input = chars.into_iter().collect();
    }

    pub fn delete_char(&mut self) {
        self.record_user_activity();
        let mut chars: Vec<char> = self.input.chars().collect();
        if self.cursor_position > 0 && self.cursor_position <= chars.len() {
            self.cursor_position -= 1;
            chars.remove(self.cursor_position);
            self.input = chars.into_iter().collect();
        }
    }

    pub fn move_cursor_left(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
        }
    }

    pub fn move_cursor_right(&mut self) {
        let count = self.input.chars().count();
        if self.cursor_position < count {
            self.cursor_position += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_unicode_and_multiline_paste() {
        let net = NetworkManager::bind(0, 0, None, "test", "alice".into()).await.unwrap();

        let mut app = App::new(
            "alice".into(),
            "test".into(),
            Arc::new(net),
            None,
            None,
            PresetMode::ReactNative,
            "nuke".into(),
            20,
        );

        // Test inserting multi-byte unicode characters (emojis, curly quotes)
        app.insert_char('🚀');
        app.insert_char('“');
        app.insert_char('a');
        assert_eq!(app.input, "🚀“a");
        assert_eq!(app.cursor_position, 3);

        // Test deleting multi-byte character
        app.delete_char();
        assert_eq!(app.input, "🚀“");
        assert_eq!(app.cursor_position, 2);

        // Test pasting multi-line stack trace
        let multi_line_paste = "Error: [Metro]\n  BuildToken[0x1234]\n  at Bundler.js:42";
        app.insert_str(multi_line_paste);
        assert!(!app.input.contains('\n'));
        assert!(app.input.contains("BuildToken[0x1234]"));
    }
}
