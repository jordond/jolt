use ratatui::{
    layout::{Alignment, Rect},
    style::Style,
    text::{Line, Span},
    widgets::Paragraph,
    Frame,
};

use crate::app::App;
use crate::data::SystemInfo;
use crate::input::keys;
use crate::theme::ThemeColors;

pub fn render_title_bar(
    frame: &mut Frame,
    area: Rect,
    system_info: &SystemInfo,
    theme: &ThemeColors,
) {
    let version = super::VERSION;

    let left_spans = vec![
        Span::styled("⚡️jolt ", Style::default().fg(theme.accent)),
        Span::styled(format!("v{}", version), Style::default().fg(theme.muted)),
    ];

    let right_text = format!(
        "{} · macOS {} · {} ",
        system_info.chip,
        system_info.os_version,
        system_info.cores_display()
    );

    let left_width: usize = left_spans.iter().map(|s| s.width()).sum();
    let right_width = right_text.chars().count();
    let padding = (area.width as usize).saturating_sub(left_width + right_width);

    let mut spans = left_spans;
    spans.push(Span::raw(" ".repeat(padding)));
    spans.push(Span::styled(right_text, Style::default().fg(theme.muted)));

    let bar = Paragraph::new(Line::from(spans)).style(Style::default().bg(theme.bg));
    frame.render_widget(bar, area);
}

pub fn render_status_bar(frame: &mut Frame, area: Rect, app: &App, theme: &ThemeColors) {
    let theme_name = app.config.theme_name();
    let theme_display = if theme_name.len() > 12 {
        format!("{}...", &theme_name[..9])
    } else {
        theme_name.to_string()
    };

    let appearance = app.config.appearance_label().to_lowercase();

    let theme_hint = format!("theme ({})", theme_display);
    let appearance_hint = format!("appearance ({})", appearance);

    let left_hints: Vec<(&str, String)> = vec![
        (keys::HELP, "help".to_string()),
        (keys::THEME, theme_hint),
        (keys::APPEARANCE, appearance_hint),
        (keys::QUIT, "quit".to_string()),
    ];

    let mut left_spans: Vec<Span> = vec![Span::raw(" ")];
    for (i, (key, desc)) in left_hints.iter().enumerate() {
        if i > 0 {
            left_spans.push(Span::styled(" │ ", Style::default().fg(theme.border)));
        }
        left_spans.push(Span::styled(*key, Style::default().fg(theme.accent)));
        left_spans.push(Span::styled(
            format!(" {}", desc),
            Style::default().fg(theme.muted),
        ));
    }

    let background_recording = app.config.user_config.history.background_recording;

    let mut right_spans: Vec<Span> = Vec::new();

    if app.is_reconnecting() {
        right_spans.push(Span::styled(
            "⟳ reconnecting ",
            Style::default().fg(theme.warning),
        ));
        right_spans.push(Span::styled("│ ", Style::default().fg(theme.border)));
    } else if app.is_data_stale() {
        right_spans.push(Span::styled("⚠ stale ", Style::default().fg(theme.warning)));
        right_spans.push(Span::styled("│ ", Style::default().fg(theme.border)));
    }

    if background_recording {
        right_spans.extend(vec![
            Span::styled("background: ", Style::default().fg(theme.muted)),
            Span::styled("on", Style::default().fg(theme.success)),
            Span::styled(" │ ", Style::default().fg(theme.border)),
        ]);
    }

    right_spans.extend(vec![
        Span::styled(keys::HISTORY, Style::default().fg(theme.accent)),
        Span::styled(" history ", Style::default().fg(theme.muted)),
        Span::styled(keys::SETTINGS, Style::default().fg(theme.accent)),
        Span::styled(" settings ", Style::default().fg(theme.muted)),
    ]);

    let left_width: usize = left_spans.iter().map(|s| s.width()).sum();
    let right_width: usize = right_spans.iter().map(|s| s.width()).sum();
    let padding = (area.width as usize).saturating_sub(left_width + right_width);

    left_spans.push(Span::raw(" ".repeat(padding)));
    left_spans.extend(right_spans);

    let line = Line::from(left_spans);
    let bar = Paragraph::new(line)
        .style(Style::default().bg(theme.bg))
        .alignment(Alignment::Left);

    frame.render_widget(bar, area);
}
