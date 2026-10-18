use mdpad::markdown::{
    format_bold, format_bullet, format_checkbox, format_heading, format_italic, format_link,
    format_quote, format_table, parse_markdown_blocks, strip_markdown_markers,
    toggle_task_checkbox, BlockKind,
};

#[test]
fn test_heading_parsing() {
    let md = "# Title\n## Subtitle\n### Section\nRegular paragraph.";
    let blocks = parse_markdown_blocks(md);
    assert_eq!(blocks.len(), 4);

    assert_eq!(blocks[0].kind, BlockKind::Heading(1, "Title".to_string()));
    assert_eq!(blocks[1].kind, BlockKind::Heading(2, "Subtitle".to_string()));
    assert_eq!(blocks[2].kind, BlockKind::Heading(3, "Section".to_string()));
    assert_eq!(blocks[3].kind, BlockKind::Paragraph("Regular paragraph.".to_string()));
}

#[test]
fn test_task_checkbox_parsing_and_toggle() {
    let md = "- [ ] Unfinished task\n- [x] Finished task";
    let blocks = parse_markdown_blocks(md);
    assert_eq!(blocks.len(), 2);

    assert_eq!(blocks[0].kind, BlockKind::TaskItem(false, "Unfinished task".to_string(), 0));
    assert_eq!(blocks[1].kind, BlockKind::TaskItem(true, "Finished task".to_string(), 1));

    let toggled_0 = toggle_task_checkbox(md, 0);
    assert!(toggled_0.starts_with("- [x] Unfinished task"));

    let toggled_1 = toggle_task_checkbox(&toggled_0, 1);
    assert!(toggled_1.contains("- [ ] Finished task"));
}

#[test]
fn test_code_block_parsing() {
    let md = "```rust\nfn main() {\n    println!(\"Hello\");\n}\n```";
    let blocks = parse_markdown_blocks(md);
    assert_eq!(blocks.len(), 1);

    if let BlockKind::CodeBlock(lang, code) = &blocks[0].kind {
        assert_eq!(lang, "rust");
        assert!(code.contains("println!"));
    } else {
        panic!("Expected code block");
    }
}

#[test]
fn test_table_parsing() {
    let md = "| Header 1 | Header 2 |\n| --- | --- |\n| Cell 1 | Cell 2 |";
    let blocks = parse_markdown_blocks(md);
    assert_eq!(blocks.len(), 1);

    if let BlockKind::Table(headers, rows) = &blocks[0].kind {
        assert_eq!(headers, &vec!["Header 1".to_string(), "Header 2".to_string()]);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0], vec!["Cell 1".to_string(), "Cell 2".to_string()]);
    } else {
        panic!("Expected table block");
    }
}

#[test]
fn test_toolbar_formatting_helpers() {
    assert_eq!(format_heading("My Header"), "# My Header");
    assert_eq!(format_bold("selected"), "**selected**");
    assert_eq!(format_bold(""), "**bold text**");
    assert_eq!(format_italic("selected"), "*selected*");
    assert_eq!(format_italic(""), "*italic text*");
    assert_eq!(format_bullet("item"), "- item");
    assert_eq!(format_checkbox("task"), "- [ ] task");
    assert_eq!(format_quote("quote"), "> quote");
    assert_eq!(format_link("click"), "[click](https://example.com)");
    assert!(format_table().contains("| Header 1 |"));
}

#[test]
fn test_strip_markdown_markers() {
    assert_eq!(strip_markdown_markers("## Heading 2"), "Heading 2");
    assert_eq!(strip_markdown_markers("- [ ] Task"), "Task");
    assert_eq!(strip_markdown_markers("- Bullet item"), "Bullet item");
    assert_eq!(strip_markdown_markers("> Quoted item"), "Quoted item");
}
