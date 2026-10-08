use chrono::Local;
use rand::seq::SliceRandom;
use rand::Rng;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum PresetMode {
    ReactNative, // Metro Bundler, Hermes engine, React Native Bridge
    Kotlin,      // Gradle Daemon, Android Logcat, Kotlin coroutines
}

impl PresetMode {
    pub fn name(&self) -> &'static str {
        match self {
            PresetMode::ReactNative => "REACT_NATIVE",
            PresetMode::Kotlin => "KOTLIN_ANDROID",
        }
    }
}

pub struct DisguiseEngine;

impl DisguiseEngine {
    /// Generates a realistic mock log line depending on the active preset.
    pub fn generate_mock_log(preset: PresetMode) -> String {
        let mut rng = rand::thread_rng();

        match preset {
            PresetMode::ReactNative => {
                let time = Local::now().format("%H:%M:%S").to_string();
                let logs = [
                    format!("BUNDLE ./index.js ░░░░░░░░░░░░░░░░ 100.0% ({}/{}), done.", rng.gen_range(1400..1520), rng.gen_range(1520..1550)),
                    format!("{} LOG  [Hermes] Initializing JSI runtime executor (engine: v0.12.0)", time),
                    format!("{} LOG  [Bridge] NativeModules.UIManager.dispatchViewManagerCommand(tag: {}, cmd: 1)", time, rng.gen_range(10..99)),
                    format!("{} LOG  [FastRefresh] Connected to Metro WebSocket packager on ws://localhost:8081", time),
                    format!("{} DEBUG [Flipper] Client handshake acknowledged: iPhone 16 Pro (iOS 18.2)", time),
                    format!("{} LOG  [Reanimated] Initialized worklets runtime (version 3.16.1)", time),
                    format!("{} LOG  [AsyncStorage] Cache initialized with {} keys (latency: {}ms)", time, rng.gen_range(12..40), rng.gen_range(1..5)),
                    format!("{} LOG  [RCTEventEmitter] Event 'onLayoutChange' dispatched to root ReactTag #{}", time, rng.gen_range(1..10)),
                ];
                logs.choose(&mut rng).cloned().unwrap_or_else(|| "BUNDLE ./index.js done.".into())
            }
            PresetMode::Kotlin => {
                let time = Local::now().format("%H:%M:%S%.3f").to_string();
                let pid = rng.gen_range(14200..18900);
                let logs = [
                    format!("> Task :core:network:kspDebugKotlin UP-TO-DATE"),
                    format!("> Task :app:compileDebugKotlin"),
                    format!("w: /app/src/main/kotlin/AuthRepository.kt: ({}, {}): Variable 'authToken' is never used", rng.gen_range(12..90), rng.gen_range(5..25)),
                    format!("{} I/System.out({}): CoroutineExceptionHandler invoked for job JobImpl{{Active}}@{:x}", time, pid, rng.gen_range(0x1000..0xffff)),
                    format!("{} D/AndroidRuntime({}): Calling main entry com.example.app.MainActivity", time, pid),
                    format!("{} I/NavController({}): Navigated to destination AuthGraph -> DashboardScreen", time, pid),
                    format!("{} D/OkHttp({}): <-- 200 OK https://api.internal/v1/user/profile ({}ms)", time, pid, rng.gen_range(18..140)),
                    format!("> Task :app:mergeDebugNativeLibs UP-TO-DATE"),
                    format!("BUILD SUCCESSFUL in {}s (24 actionable tasks: 3 executed, 21 up-to-date)", rng.gen_range(4..14)),
                ];
                logs.choose(&mut rng).cloned().unwrap_or_else(|| "> Task :app:assembleDebug".into())
            }
        }
    }

    /// Formats a chat message disguised as an authentic React Native or Kotlin log.
    pub fn format_disguised_chat(sender: &str, text: &str, preset: PresetMode) -> String {
        let time = Local::now().format("%H:%M:%S").to_string();

        match preset {
            PresetMode::ReactNative => {
                let modules = ["AppRegistry", "ReactContext", "TurboModule:State", "EventEmitter"];
                let mut rng = rand::thread_rng();
                let module = modules.choose(&mut rng).unwrap_or(&"AppRegistry");
                format!("{} LOG  [{}] <@{}> {}", time, module, sender, text)
            }
            PresetMode::Kotlin => {
                let time_full = Local::now().format("%H:%M:%S%.3f").to_string();
                let tags = ["D/AndroidRuntime(18420)", "I/KotlinLogger(18420)", "D/StateFlow(18420)", "D/Coroutines(18420)"];
                let mut rng = rand::thread_rng();
                let tag = tags.choose(&mut rng).unwrap_or(&"D/KotlinLogger");
                format!("{} {}: <@{}> {}", time_full, tag, sender, text)
            }
        }
    }

    /// Returns realistic panic screen lines matching the active developer preset.
    pub fn panic_screen_lines(preset: PresetMode) -> Vec<String> {
        match preset {
            PresetMode::ReactNative => vec![
                " Welcome to React Native (v0.76.1)".into(),
                "                        ".into(),
                " BUNDLE  ./index.js ░░░░░░░░░░░░░░░░ 100.0% (1512/1512), done.".into(),
                "".into(),
                " LOG  Running \"MobileApp\" with {\"rootTag\":1}".into(),
                " LOG  [Hermes] bytecode version: 92".into(),
                " LOG  [Reanimated] Initialized worklets runtime".into(),
                " LOG  [FastRefresh] Connected to Metro websocket.".into(),
                "".into(),
                " PASS  __tests__/App.test.tsx".into(),
                "  ✓ renders correctly (84 ms)".into(),
                "  ✓ navigates to dashboard on login success (142 ms)".into(),
                "".into(),
                "Test Suites: 1 passed, 1 total".into(),
                "Tests:       2 passed, 2 total".into(),
                "Snapshots:   0 total".into(),
                "Time:        1.428 s".into(),
                "$ _".into(),
            ],
            PresetMode::Kotlin => vec![
                "> Task :app:preBuild UP-TO-DATE".into(),
                "> Task :core:network:compileDebugKotlin".into(),
                "> Task :app:compileDebugKotlin".into(),
                "> Task :app:compileDebugUnitTestKotlin".into(),
                "> Task :app:testDebugUnitTest".into(),
                "".into(),
                "com.example.app.AuthRepositoryTest > testLoginSuccess() PASSED".into(),
                "com.example.app.AuthRepositoryTest > testTokenRefresh() PASSED".into(),
                "com.example.app.NavigationTest > testRouteProtection() PASSED".into(),
                "".into(),
                "BUILD SUCCESSFUL in 9s".into(),
                "34 actionable tasks: 4 executed, 30 up-to-date".into(),
                "".into(),
                "$ _".into(),
            ],
        }
    }
}
