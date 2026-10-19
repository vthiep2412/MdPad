pub mod parser;

pub use parser::{
    parse_live_items, parse_markdown_blocks, parse_source_lines, strip_markdown_markers,
    toggle_task_checkbox, update_line_in_text, BlockKind, LiveItem, MarkdownBlock, SourceLine,
};

pub fn format_heading(text: &str) -> String {
    format!("# {}", text)
}

pub fn format_heading_level(level: usize, text: &str) -> String {
    let hashes = "#".repeat(level.clamp(1, 6));
    format!("{} {}", hashes, text)
}

pub fn format_bold(selection: &str) -> String {
    if selection.is_empty() {
        "**bold text**".to_string()
    } else {
        format!("**{}**", selection)
    }
}

pub fn format_italic(selection: &str) -> String {
    if selection.is_empty() {
        "*italic text*".to_string()
    } else {
        format!("*{}*", selection)
    }
}

pub fn format_strikethrough(selection: &str) -> String {
    if selection.is_empty() {
        "~~strikethrough text~~".to_string()
    } else {
        format!("~~{}~~", selection)
    }
}

pub fn format_bullet(text: &str) -> String {
    format!("- {}", text)
}

pub fn format_numbered(num: usize, text: &str) -> String {
    format!("{}. {}", num, text)
}

pub fn format_checkbox(text: &str) -> String {
    format!("- [ ] {}", text)
}

pub fn format_quote(text: &str) -> String {
    format!("> {}", text)
}

pub fn format_inline_code(selection: &str) -> String {
    if selection.is_empty() {
        "`code`".to_string()
    } else {
        format!("`{}`", selection)
    }
}

pub fn format_code_block() -> &'static str {
    "```\ncode\n```\n"
}

pub fn format_link(selection: &str) -> String {
    if selection.is_empty() {
        "[link title](https://example.com)".to_string()
    } else {
        format!("[{}](https://example.com)", selection)
    }
}

pub fn format_table() -> &'static str {
    "\n| Header 1 | Header 2 |\n| --- | --- |\n| Cell 1 | Cell 2 |\n"
}
