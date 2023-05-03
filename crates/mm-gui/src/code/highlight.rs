use egui::text::LayoutJob;
use mm_eval::span::Span;

use super::CodeTheme;

pub fn highlight(theme: &CodeTheme, hover: Option<Span>, mut text: &str) -> LayoutJob {
    let mut job = LayoutJob::default();
    let mut start = 0;

    while !text.is_empty() {
        let (mut format, end) = if text.starts_with("--") {
            let end = text.find('\n').unwrap_or(text.len());
            (theme.comment.clone(), end)
        } else if text.starts_with(|c: char| c.is_ascii_digit()) {
            let end = text
                .find(|c: char| !(c.is_ascii_digit() || c == '_'))
                .unwrap_or(text.len());
            (theme.number.clone(), end)
        } else if text.starts_with(|c: char| c.is_alphabetic()) {
            let end = text
                .find(|c: char| !(c.is_alphanumeric() || c == '_' || c == '\''))
                .unwrap_or(text.len());
            (theme.plain.clone(), end)
        } else {
            let mut it = text.char_indices();
            it.next();
            let end = it.next().map_or(text.len(), |(idx, _)| idx);
            (theme.punctuation.clone(), end)
        };

        if let Some(hover) = hover {
            if hover.start == start && hover.end == start + end {
                format.background = theme.hover;
            }
        }

        job.append(&text[..end], 0.0, format);
        text = &text[end..];
        start += end;
    }

    job
}
