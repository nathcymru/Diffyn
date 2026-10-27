//! Conservative source spans. This is a bounded text aid, not a Markdown renderer.

use schemars::JsonSchema;
use serde::Serialize;

#[derive(Clone, Debug, Serialize, JsonSchema)]
pub struct ProtectedSpan {
    /// UTF-8 byte offsets into the unmodified source, with an exclusive end.
    pub start_byte: usize,
    pub end_byte: usize,
    pub kind: String,
}

/// Protect recognizable blocks and inline syntax while leaving surrounding prose usable.
/// Unclosed inline constructs conservatively protect the remaining source.
pub(super) fn protected_spans(text: &str) -> Vec<ProtectedSpan> {
    let mut ranges = Vec::new();
    let mut fence: Option<(char, usize)> = None;
    let mut offset = 0;
    for line in text.split_inclusive('\n') {
        let body = line.trim_end_matches(['\r', '\n']);
        let trimmed = body.trim_start();
        let indent = body.len() - trimmed.len();
        let marker = trimmed.chars().next();
        let run = marker.map_or(0, |ch| {
            trimmed.chars().take_while(|next| *next == ch).count()
        });
        let is_fence = indent <= 3 && matches!(marker, Some('`' | '~')) && run >= 3;
        let kind = if let Some((ch, minimum)) = fence {
            if is_fence && marker == Some(ch) && run >= minimum && trimmed[run..].trim().is_empty()
            {
                fence = None;
            }
            Some("code_block")
        } else if is_fence {
            fence = marker.map(|ch| (ch, run));
            Some("code_block")
        } else if indent >= 4 || body.starts_with('\t') {
            Some("indented_source")
        } else if trimmed.starts_with('>') {
            Some("quotation_block")
        } else if body.contains('|') {
            Some("table_or_pipe_syntax")
        } else if body.contains(['$', '=', '≤', '≥', '∑', '√']) {
            Some("mathematical_or_currency_line")
        } else if trimmed.starts_with('[') && trimmed.contains("]:") {
            Some("link_definition")
        } else {
            None
        };
        if let Some(kind) = kind {
            ranges.push(ProtectedSpan {
                start_byte: offset,
                end_byte: offset + line.len(),
                kind: kind.into(),
            });
        }
        offset += line.len();
    }

    let mut block_mask = vec![false; text.len()];
    for range in &ranges {
        block_mask[range.start_byte..range.end_byte].fill(true);
    }
    let mut position = 0;
    while position < text.len() {
        if block_mask[position] {
            position += 1;
            continue;
        }
        let tail = &text[position..];
        let Some(ch) = tail.chars().next() else { break };
        let previous = text[..position].chars().next_back();
        let mut protected = None;
        if tail.starts_with("<!--") {
            let end = tail
                .find("-->")
                .map_or(text.len(), |end| position + end + 3);
            protected = Some((end, "html_comment"));
        } else if tail.starts_with("\\(") || tail.starts_with("\\[") {
            let close = if tail.starts_with("\\(") {
                "\\)"
            } else {
                "\\]"
            };
            protected = Some((closing_end(text, position + 2, close), "mathematics"));
        } else if ch == '\\' {
            // Escapes are source syntax; do not interpret the escaped delimiter.
            let end = position + 1 + tail[1..].chars().next().map_or(0, char::len_utf8);
            protected = Some((end, "escape"));
        } else if ch == '`' {
            let count = tail.bytes().take_while(|byte| *byte == b'`').count();
            protected = Some((
                closing_end(text, position + count, &tail[..count]),
                "inline_code",
            ));
        } else if matches!(ch, '"' | '“' | '«')
            || (ch == '\'' && previous.is_none_or(char::is_whitespace) && tail[1..].contains('\''))
        {
            let close = match ch {
                '“' => "”",
                '«' => "»",
                '"' => "\"",
                _ => "'",
            };
            protected = Some((
                closing_end(text, position + ch.len_utf8(), close),
                "quotation",
            ));
        } else if ch == '[' {
            let mut end = balanced_end(text, position, '[', ']');
            if end < text.len() {
                if text[end..].starts_with('(') {
                    end = balanced_end(text, end, '(', ')');
                } else if text[end..].starts_with('[') {
                    end = balanced_end(text, end, '[', ']');
                }
            }
            protected = Some((end, "bracket_or_link"));
        } else if matches!(ch, '*' | '_')
            && !is_list_star(text, position)
            && !(ch == '_'
                && previous.is_some_and(char::is_alphanumeric)
                && tail[1..].chars().next().is_some_and(char::is_alphanumeric))
        {
            let count = tail.chars().take_while(|next| *next == ch).count();
            protected = Some((
                closing_end(text, position + count, &tail[..count]),
                "emphasis",
            ));
        } else if tail.starts_with("www.") || url_starts_here(tail) {
            let length = tail.find(char::is_whitespace).unwrap_or(tail.len());
            protected = Some((position + length, "url"));
        } else if ch == '<' {
            protected = Some((html_end(text, position), "angle_syntax"));
        }
        if let Some((end, kind)) = protected {
            ranges.push(ProtectedSpan {
                start_byte: position,
                end_byte: end,
                kind: kind.into(),
            });
            position = end;
        } else {
            position += ch.len_utf8();
        }
    }
    ranges.sort_by_key(|range| range.start_byte);
    let mut merged: Vec<ProtectedSpan> = Vec::new();
    for range in ranges {
        if let Some(previous) = merged.last_mut()
            && (range.start_byte < previous.end_byte
                || (range.start_byte == previous.end_byte && range.kind == previous.kind))
        {
            previous.end_byte = previous.end_byte.max(range.end_byte);
            if previous.kind != range.kind {
                previous.kind = "mixed_source_syntax".into();
            }
        } else {
            merged.push(range);
        }
    }
    merged
}

fn closing_end(text: &str, from: usize, delimiter: &str) -> usize {
    text[from..]
        .match_indices(delimiter)
        .find(|(offset, _)| {
            let start = from + offset;
            text[..start]
                .bytes()
                .rev()
                .take_while(|byte| *byte == b'\\')
                .count()
                % 2
                == 0
        })
        .map_or(text.len(), |(offset, _)| from + offset + delimiter.len())
}

fn html_end(text: &str, start: usize) -> usize {
    let tag_end = closing_end(text, start + 1, ">");
    let tag = &text[start + 1..tag_end];
    let name: String = tag
        .chars()
        .take_while(|ch| ch.is_ascii_alphanumeric())
        .collect();
    if name.is_empty()
        || tag.ends_with("/>")
        || [
            "br", "hr", "img", "input", "meta", "link", "wbr", "source", "area", "base", "embed",
            "param", "track",
        ]
        .contains(&name.to_ascii_lowercase().as_str())
    {
        return tag_end;
    }
    let close = format!("</{name}>");
    text[tag_end..]
        .find(&close)
        .map_or(text.len(), |offset| tag_end + offset + close.len())
}

fn balanced_end(text: &str, start: usize, open: char, close: char) -> usize {
    let mut depth = 0usize;
    let mut escaped = false;
    for (offset, ch) in text[start..].char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            continue;
        }
        if ch == open {
            depth += 1;
        }
        if ch == close {
            depth = depth.saturating_sub(1);
            if depth == 0 {
                return start + offset + ch.len_utf8();
            }
        }
    }
    text.len()
}

fn is_list_star(text: &str, position: usize) -> bool {
    text[position..].starts_with("* ")
        && text[..position]
            .rsplit('\n')
            .next()
            .is_some_and(|prefix| prefix.trim().is_empty())
}

fn url_starts_here(text: &str) -> bool {
    text.as_bytes()[..text.len().min(19)]
        .windows(3)
        .position(|part| part == b"://")
        .is_some_and(|end| {
            end > 0
                && end <= 16
                && text[..end]
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'+' | b'-' | b'.'))
                && text.as_bytes()[0].is_ascii_alphabetic()
        })
}

pub(super) fn overlaps(spans: &[ProtectedSpan], start: usize, end: usize) -> bool {
    spans
        .iter()
        .any(|span| span.start_byte < end && span.end_byte > start)
}

pub(super) fn unprotected_ranges(text: &str, spans: &[ProtectedSpan]) -> Vec<(usize, usize)> {
    let mut ranges = Vec::new();
    let mut start = 0;
    for span in spans {
        if start < span.start_byte {
            ranges.push((start, span.start_byte));
        }
        start = span.end_byte;
    }
    if start < text.len() {
        ranges.push((start, text.len()));
    }
    ranges
}
