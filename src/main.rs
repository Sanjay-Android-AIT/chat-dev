mod app;
mod crypto;
mod disguise;
mod network;
mod nostr;
mod protocol;
mod stego;
mod ui;

use std::error::Error;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use clap::Parser;
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use tokio::sync::mpsc;

use crate::app::App;
use crate::disguise::{DisguiseEngine, PresetMode};
use crate::network::NetworkManager;
use crate::protocol::NetworkPacket;

const AFTER_HELP_TEXT: &str = "\
HOW TO USE THIS APPLICATION:
  1. Quick Chat on LAN (Default):
     dev-1$ tiktik lunch-crew
     dev-2$ tiktik lunch-crew
     Messages and live presence beacons are automatically broadcast encrypted over local LAN/Wi-Fi.

  2. Testing Two Instances on the Same Machine:
     term-1$ tiktik test-room
     term-2$ tiktik test-room
     (Automatic port fallback will bind term-2 to port 9945 and auto-connect to term-1!)

  3. Disguised Cover Stories:
     - React Native:  tiktik lunch-crew --preset react   (Metro Bundler 8081 & Hermes engine)
     - Kotlin/Gradle: tiktik lunch-crew --preset kotlin  (Gradle tasks & Android Logcat)
     Press F2 inside the app to toggle cover stories on the fly!

  4. Self-Destructing / Ephemeral Room (TTL):
     tiktik lunch-secret --ttl 60
     All chat messages older than 60 seconds automatically evaporate from RAM.

  5. Steganography Bridge (Chatting over Corporate Slack / Teams):
     Type in tiktik:   /encode Can we grab lunch in 10 mins?
     Copy the fake React Native or Kotlin error into your Slack/Teams channel.
     Recipient pastes: /decode <paste error trace>
     tiktik extracts and decrypts the hidden payload!

  6. Emergency Boss Key & Duress Burn:
     - Hit ESC at any moment to display fake compilation & unit test results.
     - Type your burn word ('nuke' or /burn) to instantly wipe RAM and run 'git status'.

HOTKEYS (INSIDE APP):
  Escape        Boss Key / Panic Toggle (instantly shows realistic unit test output)
  F2            Toggle Disguise Preset between React Native (Metro) and Kotlin (Gradle)
  Tab           Toggle Layout between Disguised Server Logs and Clean Chat
  Ctrl+L        Clear All (instantly wipes history from RAM and resets screen)
  Ctrl+U        Clear current input line
  Ctrl+C        Clean exit (wipes memory from RAM)

IN-APP SLASH COMMANDS:
  /help         Show in-app command reference and cheat sheet
  /clear        Instant wipe of all chat history and screen reset
  /preset <p>   Switch disguise: '/preset react' or '/preset kotlin'
  /encode <msg> Steganography: hide encrypted message inside a realistic stack trace
  /decode <str> Steganography: decrypt message from a pasted stack trace or token
  /ttl <sec>    Set auto-destruct timer (e.g. '/ttl 60' or '/ttl off')
  /burn         Duress burn: instantly zeroize RAM and drop into a real 'git status'
  /heartbeat    Toggle background realistic log streaming on/off
  /panic        Trigger panic screen";

#[derive(Parser, Debug)]
#[command(
    name = "tiktik",
    author,
    version,
    about = "tiktik: Stealth encrypted terminal chat disguised as React Native and Kotlin dev logs",
    long_about = "tiktik is a peer-to-peer, end-to-end encrypted terminal chat designed to look\n100% like real development work (React Native Metro bundler or Kotlin/Android Gradle).\nAll messages are encrypted with ChaCha20-Poly1305 and held ephemerally in RAM.",
    after_help = AFTER_HELP_TEXT
)]
struct Cli {
    /// Room code / namespace (e.g. `tiktik lunch-crew`)
    #[arg(
        value_name = "ROOM",
        help = "Room name or code (e.g. 'tiktik lunch-crew')"
    )]
    room_arg: Option<String>,

    /// Room / Namespace identifier (alternative to positional argument)
    #[arg(
        short,
        long,
        help = "Room name or code (alternative to positional argument)"
    )]
    room: Option<String>,

    /// Your handle / worker alias (disguised as service or thread name in logs)
    #[arg(
        short,
        long,
        default_value = "worker-01",
        help = "Your alias/nickname in the chat (disguised as thread/service name in logs)"
    )]
    nick: String,

    /// Shared room encryption secret (defaults to room name if omitted)
    #[arg(
        short,
        long,
        help = "Custom passphrase for ChaCha20-Poly1305 key derivation (defaults to room name)"
    )]
    secret: Option<String>,

    /// Local UDP port to listen on
    #[arg(
        short,
        long,
        default_value_t = 9944,
        help = "Local UDP port to bind for incoming encrypted packets (default: 9944, with auto-fallback)"
    )]
    port: u16,

    /// Direct peer address (e.g. 127.0.0.1:9945) for point-to-point or local testing
    #[arg(
        long,
        help = "Direct IP:port of the peer (used when testing on the same machine or unicast)"
    )]
    peer: Option<SocketAddr>,

    /// Target broadcast port on LAN
    #[arg(
        long,
        default_value_t = 9944,
        help = "Port used for local network UDP broadcast discovery (default: 9944)"
    )]
    broadcast_port: u16,

    /// Ephemeral message auto-destruct TTL in seconds (e.g. 60 or 300)
    #[arg(
        long,
        help = "Message self-destruct timer in seconds (messages older than TTL are wiped from RAM)"
    )]
    ttl: Option<u64>,

    /// Initial disguise preset: 'react' (Metro bundler) or 'kotlin' (Gradle / Logcat)
    #[arg(
        long,
        default_value = "react",
        help = "Initial cover story preset: 'react' (Metro Bundler) or 'kotlin' (Gradle/Logcat)"
    )]
    preset: String,

    /// Duress / Burn trigger word that immediately wipes memory and executes git status
    #[arg(
        long,
        default_value = "nuke",
        help = "Secret word that, when typed into the prompt, wipes all RAM and runs 'git status'"
    )]
    burn_code: String,

    /// Idle inactivity timeout in seconds before auto-entering Ghost Mode
    #[arg(
        long,
        default_value_t = 20,
        help = "Seconds of inactivity before entering Ghost Mode (continuous fake build logs)"
    )]
    idle_timeout: u64,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Install rustls ring crypto provider for TLS/WSS connections
    let _ = rustls::crypto::ring::default_provider().install_default();

    let args = Cli::parse();
    let room_name = args.room_arg.or(args.room).unwrap_or_else(|| "auth-flow".to_string());
    let secret = args.secret.unwrap_or_else(|| room_name.clone());

    let preset_mode = match args.preset.to_lowercase().as_str() {
        "kotlin" | "android" | "gradle" => PresetMode::Kotlin,
        _ => PresetMode::ReactNative,
    };

    // Initialize Network Manager with auto port binding
    let network = match NetworkManager::bind(
        args.port,
        args.broadcast_port,
        args.peer,
        &secret,
        args.nick.clone(),
    )
    .await
    {
        Ok(net) => Arc::new(net),
        Err(e) => {
            eprintln!("Failed to bind network: {}", e);
            std::process::exit(1);
        }
    };

    // Setup channels for incoming packets (shared between LAN UDP and Nostr Relays)
    let (incoming_tx, mut incoming_rx) = mpsc::unbounded_channel::<NetworkPacket>();
    network.start_receiving(incoming_tx.clone());

    // Initialize Nostr Relay Client for cross-network / remote Wi-Fi connectivity
    let nostr = crate::nostr::NostrRelayClient::start(&secret, incoming_tx);

    // Initial presence beacon over LAN and Nostr
    let _ = network.send_presence().await;
    let _ = nostr.send_packet(&NetworkPacket::Presence {
        node_id: network.node_id.clone(),
        nick: args.nick.clone(),
    });

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(
        stdout,
        EnterAlternateScreen,
        crossterm::event::EnableBracketedPaste
    )?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(
        args.nick,
        room_name,
        network.clone(),
        Some(nostr.clone()),
        args.ttl,
        preset_mode,
        args.burn_code,
        args.idle_timeout,
    );

    // Periodic tickers
    let mut heartbeat_interval = tokio::time::interval(Duration::from_secs(4));
    let mut presence_interval = tokio::time::interval(Duration::from_secs(3));
    let mut prune_interval = tokio::time::interval(Duration::from_secs(1));

    // Main Event Loop
    let mut should_exit = false;
    while !should_exit && !app.is_burned {
        terminal.draw(|f| ui::render(f, &app))?;

        tokio::select! {
            // 1. Incoming network packets (Chat & Presence from LAN or Nostr)
            Some(packet) = incoming_rx.recv() => {
                match packet {
                    NetworkPacket::Chat(msg) => {
                        app.handle_incoming_message(msg);
                    }
                    NetworkPacket::Presence { node_id, nick } => {
                        app.record_peer_presence(node_id, nick);
                    }
                }
            }

            // 2. Periodic presence beacon broadcast & offline peer pruning (LAN + Nostr)
            _ = presence_interval.tick() => {
                let _ = network.send_presence().await;
                let _ = nostr.send_packet(&NetworkPacket::Presence {
                    node_id: network.node_id.clone(),
                    nick: app.nick.clone(),
                });
                app.prune_offline_peers();
            }

            // 3. Periodic background mock log generator (keeps terminal looking busy)
            _ = heartbeat_interval.tick() => {
                app.check_idle_timeout();
                if (app.auto_mock_logs || app.is_ghost_mode) && !app.is_panic {
                    app.add_raw_log(DisguiseEngine::generate_mock_log(app.preset), false);
                }
            }

            // 4. Ephemeral TTL message pruning
            _ = prune_interval.tick() => {
                app.prune_expired_messages();
            }

            // 5. User keyboard input
            _ = tokio::time::sleep(Duration::from_millis(50)) => {
                while event::poll(Duration::from_millis(0))? {
                    match event::read()? {
                        Event::Paste(ref text) => {
                            if !app.is_panic {
                                app.insert_str(text);
                            }
                        }
                        Event::Key(key) => {
                            app.record_user_activity();

                            match key.code {
                                KeyCode::Esc => {
                                    // Boss key / Panic toggle
                                    app.toggle_panic();
                                }
                                KeyCode::Tab => {
                                    app.toggle_mode();
                                }
                                KeyCode::F(2) => {
                                    // Toggle between React Native and Kotlin presets
                                    app.toggle_preset();
                                }
                                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                    should_exit = true;
                                    break;
                                }
                                KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                    app.input.clear();
                                    app.cursor_position = 0;
                                }
                                KeyCode::Char('l') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                    // Instant clear all history
                                    app.clear_all_history();
                                }
                                KeyCode::Enter => {
                                    app.submit_input().await;
                                }
                                KeyCode::Char(c) => {
                                    if !app.is_panic {
                                        app.insert_char(c);
                                    }
                                }
                                KeyCode::Backspace => {
                                    if !app.is_panic {
                                        app.delete_char();
                                    }
                                }
                                KeyCode::Left => {
                                    app.move_cursor_left();
                                }
                                KeyCode::Right => {
                                    app.move_cursor_right();
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    // Clean up and restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        crossterm::event::DisableBracketedPaste,
        LeaveAlternateScreen
    )?;
    terminal.show_cursor()?;

    if app.is_burned {
        // Duress mode triggered: execute real git status and exit with zero trace
        println!("$ git status");
        let _ = std::process::Command::new("git").arg("status").status();
    } else {
        println!("Session terminated. In-memory data zeroed.");
    }

    Ok(())
}
