use crate::crypto::{decrypt, encrypt};
use crate::disguise::PresetMode;
use crate::protocol::ChatMessage;

pub struct StegoEngine;

impl StegoEngine {
    /// Encrypts a message and hides it inside a realistic React Native or Kotlin error stack trace.
    pub fn encode_to_stack_trace(
        key: &[u8; 32],
        sender: &str,
        text: &str,
        preset: PresetMode,
    ) -> Result<String, String> {
        let msg = ChatMessage::new(sender, text);
        let bytes = msg.to_bytes()?;
        let encrypted = encrypt(key, &bytes)?;
        let hex_payload: String = encrypted.iter().map(|b| format!("{:02x}", b)).collect();

        match preset {
            PresetMode::ReactNative => Ok(format!(
                "Error: [Metro] Invariant Violation / Module checksum mismatch:\n  BuildToken[0x{}]\n  at MetroBundler.processSource (node_modules/metro/src/Bundler.js:142:19)\n  at ModuleGraph.resolve (node_modules/metro/src/node-haste/index.js:84:11)\n  at async Server._processRequest (node_modules/metro/src/Server.js:419:22)",
                hex_payload
            )),
            PresetMode::Kotlin => Ok(format!(
                "FATAL EXCEPTION: main\n  Process: com.example.mobile, PID: 18420\n  CrashToken[0x{}]\n  at com.example.mobile.MainActivity.onCreate(MainActivity.kt:48)\n  at android.app.ActivityThread.performLaunchActivity(ActivityThread.java:3642)\n  at com.android.internal.os.ZygoteInit.main(ZygoteInit.java:1024)",
                hex_payload
            )),
        }
    }

    /// Extracts and decrypts a hidden message from a pasted stack trace or token.
    pub fn decode_from_stack_trace(
        key: &[u8; 32],
        pasted_text: &str,
    ) -> Result<ChatMessage, String> {
        // Extract hex string: look for BuildToken[0x...], CrashToken[0x...], or any 0x... hex sequence
        let hex_str = if let Some(start) = pasted_text.find("0x") {
            let after_0x = &pasted_text[start + 2..];
            let end = after_0x
                .find(|c: char| !c.is_ascii_hexdigit())
                .unwrap_or(after_0x.len());
            &after_0x[..end]
        } else {
            // Alternatively find continuous hex string of at least 24 chars (nonce + payload)
            let mut best_slice = "";
            let words = pasted_text.split_whitespace();
            for w in words {
                let cleaned = w.trim_matches(|c: char| !c.is_ascii_hexdigit());
                if cleaned.len() >= 24 && cleaned.len() > best_slice.len() {
                    best_slice = cleaned;
                }
            }
            if best_slice.is_empty() {
                return Err("No encrypted BuildToken/CrashToken found in pasted text".into());
            }
            best_slice
        };

        if hex_str.len() % 2 != 0 || hex_str.len() < 24 {
            return Err("Invalid or incomplete encrypted token format".into());
        }

        // Convert hex to bytes
        let mut payload = Vec::with_capacity(hex_str.len() / 2);
        for i in (0..hex_str.len()).step_by(2) {
            let byte = u8::from_str_radix(&hex_str[i..i + 2], 16)
                .map_err(|e| format!("Hex decoding failed: {}", e))?;
            payload.push(byte);
        }

        // Decrypt using AEAD key
        let decrypted_bytes = decrypt(key, &payload)?;
        let msg = ChatMessage::from_bytes(&decrypted_bytes)?;
        Ok(msg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::derive_key;

    #[test]
    fn test_stego_roundtrip_react_native() {
        let key = derive_key("rn-secret-room");
        let trace = StegoEngine::encode_to_stack_trace(&key, "alice", "Can you review my PR?", PresetMode::ReactNative).unwrap();
        assert!(trace.contains("BuildToken[0x"));

        let decoded = StegoEngine::decode_from_stack_trace(&key, &trace).unwrap();
        assert_eq!(decoded.sender, "alice");
        assert_eq!(decoded.content, "Can you review my PR?");
    }

    #[test]
    fn test_stego_roundtrip_kotlin() {
        let key = derive_key("kotlin-secret-room");
        let trace = StegoEngine::encode_to_stack_trace(&key, "bob", "Testing Kotlin log stego", PresetMode::Kotlin).unwrap();
        assert!(trace.contains("CrashToken[0x"));

        let decoded = StegoEngine::decode_from_stack_trace(&key, &trace).unwrap();
        assert_eq!(decoded.sender, "bob");
        assert_eq!(decoded.content, "Testing Kotlin log stego");
    }
}
