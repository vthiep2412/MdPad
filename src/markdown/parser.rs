
#[derive(Debug, Clone, PartialEq)]
pub enum BlockKind {
    Heading(u8, String),
    Paragraph(String),
    CodeBlock(String, String),
    Blockquote(String),
    ListItem(String),
    TaskItem(bool, String, usize),
    Table(Vec<String>, Vec<Vec<String>>),
    ThematicBreak,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MarkdownBlock {
    pub kind: BlockKind,
    pub raw_text: String,
    pub line_index: usize,
}

pub fn parse_markdown_blocks(content: &str) -> Vec<MarkdownBlock> {
    let mut blocks = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i];
        let trimmed = line.trim();

        if trimmed.is_empty() {
            i += 1;
            continue;
        }

        if trimmed.starts_with("#") {
            let mut level = 0u8;
            for ch in trimmed.chars() {
                if ch == '#' {
                    level += 1;
                } else {
                    break;
                }
            }
            if level <= 6 {
                let text = trimmed[level as usize..].trim().to_string();
                blocks.push(MarkdownBlock {
                    kind: BlockKind::Heading(level, text),
                    raw_text: line.to_string(),
                    line_index: i,
                });
                i += 1;
                continue;
            }
        }

        if trimmed.starts_with("- [ ] ") || trimmed.starts_with("- [x] ") || trimmed.starts_with("- [X] ") {
            let checked = trimmed.starts_with("- [x] ") || trimmed.starts_with("- [X] ");
            let text = trimmed[6..].trim().to_string();
            blocks.push(MarkdownBlock {
                kind: BlockKind::TaskItem(checked, text, i),
                raw_text: line.to_string(),
                line_index: i,
            });
            i += 1;
            continue;
        }

        if trimmed.starts_with("- ") || trimmed.starts_with("* ") || trimmed.starts_with("+ ") {
            let text = trimmed[2..].trim().to_string();
            blocks.push(MarkdownBlock {
                kind: BlockKind::ListItem(text),
                raw_text: line.to_string(),
                line_index: i,
            });
            i += 1;
            continue;
        }

        if trimmed.starts_with("> ") {
            let text = trimmed[2..].trim().to_string();
            blocks.push(MarkdownBlock {
                kind: BlockKind::Blockquote(text),
                raw_text: line.to_string(),
                line_index: i,
            });
            i += 1;
            continue;
        }

        if trimmed == "---" || trimmed == "***" || trimmed == "___" {
            blocks.push(MarkdownBlock {
                kind: BlockKind::ThematicBreak,
                raw_text: line.to_string(),
                line_index: i,
            });
            i += 1;
            continue;
        }

        if trimmed.starts_with("```") {
            let lang = trimmed[3..].trim().to_string();
            let mut code_lines = Vec::new();
            let start_line = i;
            i += 1;
            while i < lines.len() {
                if lines[i].trim().starts_with("```") {
                    i += 1;
                    break;
                }
                code_lines.push(lines[i]);
                i += 1;
            }
            let code = code_lines.join("\n");
            let raw_text = lines[start_line..i.min(lines.len())].join("\n");
            blocks.push(MarkdownBlock {
                kind: BlockKind::CodeBlock(lang, code),
                raw_text,
                line_index: start_line,
            });
            continue;
        }

        if trimmed.starts_with("|") && trimmed.ends_with("|") {
            let mut table_lines = Vec::new();
            let start_line = i;
            while i < lines.len() && lines[i].trim().starts_with("|") {
                table_lines.push(lines[i].trim());
                i += 1;
            }
            if table_lines.len() >= 2 {
                let parse_row = |row: &str| -> Vec<String> {
                    row.split('|')
                        .map(|c| c.trim().to_string())
                        .filter(|c| !c.is_empty())
                        .collect()
                };
                let headers = parse_row(table_lines[0]);
                let rows: Vec<Vec<String>> = table_lines.iter().skip(2).map(|r| parse_row(r)).collect();
                let raw_text = lines[start_line..i].join("\n");
                blocks.push(MarkdownBlock {
                    kind: BlockKind::Table(headers, rows),
                    raw_text,
                    line_index: start_line,
                });
                continue;
            }
        }

        blocks.push(MarkdownBlock {
            kind: BlockKind::Paragraph(trimmed.to_string()),
            raw_text: line.to_string(),
            line_index: i,
        });
        i += 1;
    }

    blocks
}

pub fn strip_markdown_markers(line: &str) -> String {
    let trimmed = line.trim();
    if trimmed.starts_with("#") {
        let no_hash = trimmed.trim_start_matches('#');
        return no_hash.trim().to_string();
    }
    if trimmed.starts_with("- [ ] ") || trimmed.starts_with("- [x] ") || trimmed.starts_with("- [X] ") {
        return trimmed[6..].trim().to_string();
    }
    if trimmed.starts_with("- ") || trimmed.starts_with("* ") || trimmed.starts_with("+ ") {
        return trimmed[2..].trim().to_string();
    }
    if trimmed.starts_with("> ") {
        return trimmed[2..].trim().to_string();
    }
    trimmed.to_string()
}

pub fn toggle_task_checkbox(content: &str, target_line_index: usize) -> String {
    let mut lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
    if target_line_index < lines.len() {
        let line = &lines[target_line_index];
        let trimmed = line.trim_start();
        let indent = &line[..(line.len() - trimmed.len())];
        if trimmed.starts_with("- [ ] ") {
            lines[target_line_index] = format!("{}- [x] {}", indent, &trimmed[6..]);
        } else if trimmed.starts_with("- [x] ") || trimmed.starts_with("- [X] ") {
            lines[target_line_index] = format!("{}- [ ] {}", indent, &trimmed[6..]);
        }
    }
    lines.join("\n")
}
