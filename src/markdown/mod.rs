pub mod parser;

pub use parser::{parse_markdown_blocks, strip_markdown_markers, toggle_task_checkbox, BlockKind, MarkdownBlock};

pub fn format_heading(text: &str) -> String {
    format!("# {}", text)
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

pub fn format_bullet(text: &str) -> String {
    format!("- {}", text)
}

pub fn format_checkbox(text: &str) -> String {
    format!("- [ ] {}", text)
}

pub fn format_quote(text: &str) -> String {
    format!("> {}", text)
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
