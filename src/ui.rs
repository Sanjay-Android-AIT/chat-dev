use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame,
};

use crate::app::{App, ViewMode};
use crate::disguise::{DisguiseEngine, PresetMode};

pub fn render(f: &mut Frame, app: &App) {
    if app.is_panic {
        render_panic_screen(f, app.preset);
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Top header / build telemetry
            Constraint::Min(5),    // Main logs / chat body
            Constraint::Length(3), // Disguised command input
        ])
        .split(f.area());

    // 1. Header (Camouflaged as Metro Bundler or Gradle Daemon)
    let header_text = build_header_spans(app);

    let header_title = match app.preset {
        PresetMode::ReactNative => " Metro Bundler v0.76.1 (Hermes) ",
        PresetMode::Kotlin => " Gradle Daemon 8.7 (Android SDK 35) ",
    };

    let header = Paragraph::new(Line::from(header_text)).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
            .title(header_title),
    );
    f.render_widget(header, chunks[0]);

    // 2. Logs / Chat Body
    let items: Vec<ListItem> = app
        .messages
        .iter()
        .map(|msg| {
            let styled_line = colorize_preset_log(&msg.text, app.view_mode, app.preset);
            ListItem::new(styled_line)
        })
        .collect();

    let logs_title = match (app.view_mode, app.preset) {
        (ViewMode::CleanChat, _) => " Channel Feed ",
        (ViewMode::ServerLogs, PresetMode::ReactNative) => " React Native Metro Console (tail -f) ",
        (ViewMode::ServerLogs, PresetMode::Kotlin) => " Android Logcat & Gradle Output ",
    };

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
            .title(logs_title),
    );
    f.render_widget(list, chunks[1]);

    // 3. Bottom Disguised Input Bar
    let (prompt_prefix, prompt_suffix) = if app.is_ghost_mode {
        match app.preset {
            PresetMode::ReactNative => ("$ npx react-native start (watching filesystem...)", ""),
            PresetMode::Kotlin => ("$ ./gradlew --continuous (waiting for changes...)", ""),
        }
    } else {
        match (app.view_mode, app.preset) {
            (ViewMode::CleanChat, _) => ("> ", ""),
            (ViewMode::ServerLogs, PresetMode::ReactNative) => ("$ npx react-native log --filter \"", "\""),
            (ViewMode::ServerLogs, PresetMode::Kotlin) => ("$ ./gradlew -Pmsg=\"", "\""),
        }
    };

    let input_line = if app.is_ghost_mode {
        Line::from(vec![
            Span::styled(prompt_prefix, Style::default().fg(Color::DarkGray)),
        ])
    } else {
        Line::from(vec![
            Span::styled(prompt_prefix, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            Span::styled(&app.input, Style::default().fg(Color::White)),
            Span::styled(prompt_suffix, Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ])
    };

    let input_title = if app.is_ghost_mode {
        " Daemon Watcher (Idle) "
    } else {
        " Ingress Command "
    };

    let input_widget = Paragraph::new(input_line).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::DarkGray))
            .title(input_title),
    );
    f.render_widget(input_widget, chunks[2]);

    if !app.is_ghost_mode {
        let cursor_x = chunks[2].x + prompt_prefix.len() as u16 + app.cursor_position as u16 + 1;
        let cursor_y = chunks[2].y + 1;
        f.set_cursor_position((cursor_x, cursor_y));
    }
}

fn build_header_spans(app: &App) -> Vec<Span<'static>> {
    let mut spans = Vec::new();

    match app.preset {
        PresetMode::ReactNative => {
            spans.push(Span::styled(" [METRO: ", Style::default().fg(Color::DarkGray)));
            spans.push(Span::styled("8081", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)));
            spans.push(Span::styled("] [HERMES: ", Style::default().fg(Color::DarkGray)));
            spans.push(Span::styled("ACTIVE", Style::default().fg(Color::Green)));
            spans.push(Span::styled("] [ROOM: ", Style::default().fg(Color::DarkGray)));
            spans.push(Span::styled(app.room_name.clone(), Style::default().fg(Color::LightBlue)));
        }
        PresetMode::Kotlin => {
            spans.push(Span::styled(" [GRADLE: ", Style::default().fg(Color::DarkGray)));
            spans.push(Span::styled("8.7", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)));
            spans.push(Span::styled("] [JDK: ", Style::default().fg(Color::DarkGray)));
            spans.push(Span::styled("21", Style::default().fg(Color::Green)));
            spans.push(Span::styled("] [TASK: ", Style::default().fg(Color::DarkGray)));
            spans.push(Span::styled(format!(":{}", app.room_name), Style::default().fg(Color::LightBlue)));
        }
    }

    // Subtle silent notification indicator (no audio, visual icon only)
    if app.has_unread {
        spans.push(Span::styled("] [⚡ ", Style::default().fg(Color::DarkGray)));
        spans.push(Span::styled("1 UPDATE PENDING", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)));
    } else {
        spans.push(Span::styled("] [● ", Style::default().fg(Color::DarkGray)));
        spans.push(Span::styled("SYNCED", Style::default().fg(Color::DarkGray)));
    }

    // Live Mesh Nodes counter
    let node_count = app.active_node_count();
    let (node_label, node_style) = if node_count > 1 {
        (
            format!("{} (ONLINE ●)", node_count),
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD),
        )
    } else {
        (
            "1 (SEARCHING...)".to_string(),
            Style::default().fg(Color::Yellow),
        )
    };
    spans.push(Span::styled("] [NODES: ", Style::default().fg(Color::DarkGray)));
    spans.push(Span::styled(node_label, node_style));

    if let Some(ttl) = app.message_ttl {
        spans.push(Span::styled("] [TTL: ", Style::default().fg(Color::DarkGray)));
        spans.push(Span::styled(format!("{}s", ttl.as_secs()), Style::default().fg(Color::Magenta)));
    }

    spans.push(Span::styled("] [F2: ", Style::default().fg(Color::DarkGray)));
    spans.push(Span::styled(app.preset.name(), Style::default().fg(Color::Yellow)));
    spans.push(Span::styled("] [ESC: ", Style::default().fg(Color::DarkGray)));
    spans.push(Span::styled("PANIC", Style::default().fg(Color::Red)));
    spans.push(Span::styled("] ", Style::default().fg(Color::DarkGray)));

    spans
}

fn render_panic_screen(f: &mut Frame, preset: PresetMode) {
    let lines = DisguiseEngine::panic_screen_lines(preset);
    let spans: Vec<Line> = lines
        .into_iter()
        .map(|l| {
            if l.contains("PASS") || l.contains("SUCCESSFUL") || l.contains("PASSED") || l.contains("✓") {
                Line::from(vec![
                    Span::styled(l, Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
                ])
            } else if l.contains("> Task") || l.contains("BUNDLE") {
                Line::from(vec![
                    Span::styled(l, Style::default().fg(Color::Cyan)),
                ])
            } else {
                Line::from(l)
            }
        })
        .collect();

    let paragraph = Paragraph::new(spans).block(
        Block::default()
            .borders(Borders::NONE)
            .style(Style::default().bg(Color::Reset)),
    );
    f.render_widget(paragraph, f.area());
}

fn colorize_preset_log<'a>(line: &'a str, mode: ViewMode, _preset: PresetMode) -> Line<'a> {
    if mode == ViewMode::CleanChat {
        return Line::from(line.to_string());
    }

    if line.contains("BUNDLE") {
        Line::from(vec![
            Span::styled("BUNDLE ", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::raw(line.trim_start_matches("BUNDLE ")),
        ])
    } else if line.contains("> Task") {
        Line::from(vec![
            Span::styled(line, Style::default().fg(Color::Cyan)),
        ])
    } else if line.contains("LOG") {
        let parts: Vec<&str> = line.splitn(2, "LOG").collect();
        Line::from(vec![
            Span::styled(parts[0], Style::default().fg(Color::DarkGray)),
            Span::styled("LOG", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
            Span::styled(parts.get(1).copied().unwrap_or(""), Style::default().fg(Color::White)),
        ])
    } else if line.contains("DEBUG") || line.contains("D/") {
        Line::from(vec![
            Span::styled(line, Style::default().fg(Color::Yellow)),
        ])
    } else if line.contains("BUILD SUCCESSFUL") {
        Line::from(vec![
            Span::styled(line, Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)),
        ])
    } else if line.contains("--- [STEGO PAYLOAD") || line.contains("BuildToken[0x") || line.contains("CrashToken[0x") {
        Line::from(vec![
            Span::styled(line, Style::default().fg(Color::Magenta)),
        ])
    } else {
        Line::from(line.to_string())
    }
}
