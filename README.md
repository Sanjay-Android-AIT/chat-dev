# TikTik (`tiktik`)

> **Personal, private, end-to-end encrypted terminal chat disguised as React Native and Kotlin developer logs.**
> Built in Rust for macOS and Windows using [Ratatui](https://github.com/ratatui/ratatui), [Tokio](https://tokio.rs/), and [ChaCha20-Poly1305](https://en.wikipedia.org/wiki/ChaCha20-Poly1305).

---

## 📦 Installation

### macOS (Homebrew)
```bash
brew install Sanjay-Android-AIT/tap/tiktik
```

### Windows (PowerShell 1-Liner — No Package Manager Needed)
```powershell
irm https://raw.githubusercontent.com/Sanjay-Android-AIT/chat-dev/main/install.ps1 | iex
```

### Windows (Scoop)
```powershell
scoop bucket add mytools https://github.com/Sanjay-Android-AIT/scoop-bucket
scoop install tiktik
```

### Windows (Winget)
```powershell
winget install tiktik
```

### From Source (Any OS with Rust)
```bash
cargo install --git https://github.com/Sanjay-Android-AIT/chat-dev.git
```

---

## 🌐 Seamless Dual-Channel Networking

You just enter the room name: `tiktik <room_code>` — the app handles everything automatically:

1. **Same Wi-Fi / Local Office LAN:**
   * Uses low-latency local UDP broadcast with automatic sibling port cycling.
   * Two devs on the same Wi-Fi connect instantly without internet access.
   * Real-time presence beacon updates the live header: `[NODES: 2 (ONLINE ●)]`.

2. **Different Wi-Fi / Remote Internet (Nostr Relay Engine):**
   * If peers are on different networks (e.g. home vs office), `tiktik` automatically connects to high-availability public **Nostr Relays** (`wss://relay.damus.io`, `wss://nos.lol`) over secure Port 443 (WSS).
   * **Ephemeral by Design (NIP-16 Kind 20000):** Events are held in RAM by relays, delivered to subscribers, and never stored on disk.
   * **Zero-Knowledge:** Relays only see a hashed topic tag (`#t`). Payloads are authenticated and encrypted with `ChaCha20-Poly1305`.

---

## 🎭 The Disguise: Exclusively React Native & Kotlin

Switch between your cover stories on the fly using **`F2`** or `/preset react` / `/preset kotlin`.

### 1. React Native Preset (`--preset react`)
* **Header:** `[METRO: 8081] [HERMES: ACTIVE] [ROOM: auth-flow]`
* **Logs:** Simulates Metro bundle updates, Hermes runtime initialization, TurboModule bridge dispatches, and Flipper handshakes.
* **Disguised Messages:** Appear as `LOG [AppRegistry] <@alice>: Hey, did you push the fix?`
* **Input Disguise:** `$ npx react-native log --filter "..."`
* **Panic Screen:** Shows realistic Metro bundler output and Jest test suite passes (`✓ renders correctly`).

### 2. Kotlin / Android Preset (`--preset kotlin`)
* **Header:** `[GRADLE: 8.7] [JDK: 21] [TASK: :auth-flow]`
* **Logs:** Simulates Gradle tasks (`> Task :core:network:kspDebugKotlin`), Kotlin compiler warnings, Android Logcat traces, and OkHttp requests.
* **Disguised Messages:** Appear as `D/AndroidRuntime(18420): <@alice> Hey, did you push the fix?`
* **Input Disguise:** `$ ./gradlew -Pmsg="..."`
* **Panic Screen:** Shows realistic Gradle task execution and JUnit test results (`com.example.app.AuthTest > PASSED`).

---

## 👻 Ghost / Auto-Idle Camouflage

* If you are inactive for **20 seconds** (configurable with `--idle-timeout <secs>`):
  * The terminal automatically enters **Ghost Mode**.
  * The command prompt fades out to `$ ./gradlew --continuous` or `$ npx react-native start`.
  * Continuous realistic build logs stream automatically onto the screen so your terminal looks like a background watcher.
* **Wakeup:** Pressing **any key** instantly wakes the app back up to active chat.

---

## ⚡ Subtle Silent Notifications (No Audio)

* **Zero audio bells or alerts.**
* When an incoming message arrives while you are idle or looking away:
  * Normal state: `[● SYNCED]`
  * Unread state: `[⚡ 1 UPDATE PENDING]` in highlighted yellow/cyan.
* Resets automatically as soon as you type or hit any key.

---

## 🕵️ Steganography Mode (Slack / Microsoft Teams Bridge)

Need to message a teammate over corporate Slack or Teams without IT noticing?

1. **Encode:**
   ```text
   /encode Can we grab lunch in 10 minutes?
   ```
   The app encrypts the payload and formats it as a realistic React Native or Kotlin error:
   ```text
   Error: [Metro] Invariant Violation / Module checksum mismatch:
     BuildToken[0x7a8f09bc12...]
     at MetroBundler.processSource (node_modules/metro/src/Bundler.js:142:19)
   ```
2. **Send:** Copy and paste the fake error in a Slack/Teams channel. It looks like an innocent question asking for help with a build error.
3. **Decode:** The recipient pastes the snippet into their terminal:
   ```text
   /decode <paste the error snippet>
   ```
   `tiktik` automatically extracts the token, decrypts it, and displays the hidden message!

---

## 🚨 Duress / Burn Code

* Configure a custom burn code via `--burn-code <word>` (default: `nuke` or `/burn`).
* If someone walks up to your desk or demands to see your terminal, simply type your burn word and press Enter.
* **Action:**
  1. Instantly zeroes and wipes all keys, messages, and buffers from RAM.
  2. Drops out of alternate screen immediately.
  3. Executes a real `git status` command in your terminal so it looks completely normal:
     ```text
     $ git status
     On branch main
     nothing to commit, working tree clean
     ```

---

## ⏱️ Ephemeral Chat / Auto-Destruct (TTL)

* Launch with self-destruct timer:
  ```bash
  cargo run -- --ttl 60
  ```
* Or set dynamically inside chat: `/ttl 120` or `/ttl off`.
* Messages older than the TTL are automatically scrubbed from memory every second.

---

## ⌨️ Command & Shortcut Reference

| Hotkey / Command | Description |
| :--- | :--- |
| **`Escape`** | **Panic Button (Boss Key)** — Instant fake test/build output |
| **`F2`** | Toggle disguise preset (**React Native** ↔ **Kotlin**) |
| **`Tab`** | Toggle layout style (Disguised Logs vs. Clean Chat) |
| **`Ctrl+L`** or `/clear` | **Clear All** — Instantly wipes history and resets buffer |
| **`/encode <msg>`** | Hide encrypted message inside a realistic stack trace |
| **`/decode <trace>`** | Decrypt a message from a pasted stack trace |
| **`/ttl <seconds>`** | Set or change ephemeral message auto-destruct duration |
| **`/burn`** or `nuke` | **Burn & Duress Code** — Wipe RAM and execute `git status` |
| **`Ctrl+C`** | Clean exit |
