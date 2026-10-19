
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
            i = start_line;
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
    let mut out = lines.join("\n");
    if content.ends_with('\n') {
        out.push('\n');
    }
    out
}

#[derive(Debug, Clone, PartialEq)]
pub struct SourceLine {
    pub line_index: usize,
    pub raw_text: String,
    pub prefix: String,
    pub content: String,
    pub kind: i32,
    pub is_bold: bool,
}

pub fn parse_source_lines(content: &str) -> Vec<SourceLine> {
    let mut result = Vec::new();
    let lines: Vec<&str> = if content.is_empty() {
        vec![""]
    } else {
        content.split('\n').collect()
    };

    for (i, &raw_line) in lines.iter().enumerate() {
        let line = raw_line.strip_suffix('\r').unwrap_or(raw_line);
        let trimmed = line.trim_start();
        let indent = &line[..(line.len() - trimmed.len())];

        if trimmed.starts_with("# ") {
            result.push(SourceLine {
                line_index: i,
                raw_text: line.to_string(),
                prefix: format!("{}# ", indent),
                content: trimmed[2..].to_string(),
                kind: 1,
                is_bold: true,
            });
        } else if trimmed.starts_with("## ") {
            result.push(SourceLine {
                line_index: i,
                raw_text: line.to_string(),
                prefix: format!("{}## ", indent),
                content: trimmed[3..].to_string(),
                kind: 2,
                is_bold: true,
            });
        } else if trimmed.starts_with("### ") {
            result.push(SourceLine {
                line_index: i,
                raw_text: line.to_string(),
                prefix: format!("{}### ", indent),
                content: trimmed[4..].to_string(),
                kind: 3,
                is_bold: true,
            });
        } else if trimmed.starts_with("#### ") {
            result.push(SourceLine {
                line_index: i,
                raw_text: line.to_string(),
                prefix: format!("{}#### ", indent),
                content: trimmed[5..].to_string(),
                kind: 4,
                is_bold: true,
            });
        } else if trimmed.starts_with("##### ") {
            result.push(SourceLine {
                line_index: i,
                raw_text: line.to_string(),
                prefix: format!("{}##### ", indent),
                content: trimmed[6..].to_string(),
                kind: 5,
                is_bold: true,
            });
        } else if trimmed.starts_with("###### ") {
            result.push(SourceLine {
                line_index: i,
                raw_text: line.to_string(),
                prefix: format!("{}###### ", indent),
                content: trimmed[7..].to_string(),
                kind: 6,
                is_bold: true,
            });
        } else if trimmed.starts_with("> ") {
            result.push(SourceLine {
                line_index: i,
                raw_text: line.to_string(),
                prefix: format!("{}> ", indent),
                content: trimmed[2..].to_string(),
                kind: 7,
                is_bold: false,
            });
        } else if trimmed.starts_with("- [ ] ") || trimmed.starts_with("- [x] ") || trimmed.starts_with("- [X] ") {
            result.push(SourceLine {
                line_index: i,
                raw_text: line.to_string(),
                prefix: format!("{}{}", indent, &trimmed[..6]),
                content: trimmed[6..].to_string(),
                kind: 9,
                is_bold: false,
            });
        } else if trimmed.starts_with("- ") || trimmed.starts_with("* ") || trimmed.starts_with("+ ") {
            result.push(SourceLine {
                line_index: i,
                raw_text: line.to_string(),
                prefix: format!("{}{}", indent, &trimmed[..2]),
                content: trimmed[2..].to_string(),
                kind: 8,
                is_bold: false,
            });
        } else if trimmed.starts_with("```") {
            result.push(SourceLine {
                line_index: i,
                raw_text: line.to_string(),
                prefix: format!("{}```", indent),
                content: trimmed[3..].to_string(),
                kind: 10,
                is_bold: false,
            });
        } else if trimmed == "---" || trimmed == "***" || trimmed == "___" {
            result.push(SourceLine {
                line_index: i,
                raw_text: line.to_string(),
                prefix: line.to_string(),
                content: String::new(),
                kind: 11,
                is_bold: false,
            });
        } else {
            let is_bold = trimmed.contains("**") || trimmed.contains("__");
            result.push(SourceLine {
                line_index: i,
                raw_text: line.to_string(),
                prefix: String::new(),
                content: line.to_string(),
                kind: 0,
                is_bold,
            });
        }
    }

    result
}

#[derive(Debug, Clone, PartialEq)]
pub struct LiveItem {
    pub line_index: usize,
    pub raw_text: String,
    pub kind: i32,
    pub text: String,
    pub checked: bool,
}

pub fn parse_live_items(content: &str) -> Vec<LiveItem> {
    let mut result = Vec::new();
    let lines: Vec<&str> = if content.is_empty() {
        vec![""]
    } else {
        content.split('\n').collect()
    };

    for (i, &raw_line) in lines.iter().enumerate() {
        let line = raw_line.strip_suffix('\r').unwrap_or(raw_line);
        let trimmed = line.trim();

        if trimmed.starts_with("# ") {
            result.push(LiveItem {
                line_index: i,
                raw_text: line.to_string(),
                kind: 1,
                text: trimmed[2..].trim().to_string(),
                checked: false,
            });
        } else if trimmed.starts_with("## ") {
            result.push(LiveItem {
                line_index: i,
                raw_text: line.to_string(),
                kind: 2,
                text: trimmed[3..].trim().to_string(),
                checked: false,
            });
        } else if trimmed.starts_with("### ") {
            result.push(LiveItem {
                line_index: i,
                raw_text: line.to_string(),
                kind: 3,
                text: trimmed[4..].trim().to_string(),
                checked: false,
            });
        } else if trimmed.starts_with("#### ") {
            result.push(LiveItem {
                line_index: i,
                raw_text: line.to_string(),
                kind: 4,
                text: trimmed[5..].trim().to_string(),
                checked: false,
            });
        } else if trimmed.starts_with("##### ") {
            result.push(LiveItem {
                line_index: i,
                raw_text: line.to_string(),
                kind: 5,
                text: trimmed[6..].trim().to_string(),
                checked: false,
            });
        } else if trimmed.starts_with("###### ") {
            result.push(LiveItem {
                line_index: i,
                raw_text: line.to_string(),
                kind: 6,
                text: trimmed[7..].trim().to_string(),
                checked: false,
            });
        } else if trimmed.starts_with("- [ ] ") {
            result.push(LiveItem {
                line_index: i,
                raw_text: line.to_string(),
                kind: 7,
                text: trimmed[6..].trim().to_string(),
                checked: false,
            });
        } else if trimmed.starts_with("- [x] ") || trimmed.starts_with("- [X] ") {
            result.push(LiveItem {
                line_index: i,
                raw_text: line.to_string(),
                kind: 7,
                text: trimmed[6..].trim().to_string(),
                checked: true,
            });
        } else if trimmed.starts_with("- ") || trimmed.starts_with("* ") || trimmed.starts_with("+ ") {
            result.push(LiveItem {
                line_index: i,
                raw_text: line.to_string(),
                kind: 8,
                text: trimmed[2..].trim().to_string(),
                checked: false,
            });
        } else if trimmed.starts_with("> ") {
            result.push(LiveItem {
                line_index: i,
                raw_text: line.to_string(),
                kind: 9,
                text: trimmed[2..].trim().to_string(),
                checked: false,
            });
        } else if trimmed.starts_with("```") {
            result.push(LiveItem {
                line_index: i,
                raw_text: line.to_string(),
                kind: 10,
                text: trimmed.to_string(),
                checked: false,
            });
        } else if trimmed == "---" || trimmed == "***" || trimmed == "___" {
            result.push(LiveItem {
                line_index: i,
                raw_text: line.to_string(),
                kind: 11,
                text: String::new(),
                checked: false,
            });
        } else {
            result.push(LiveItem {
                line_index: i,
                raw_text: line.to_string(),
                kind: 0,
                text: line.to_string(),
                checked: false,
            });
        }
    }

    result
}

pub fn update_line_in_text(content: &str, line_idx: usize, new_line: &str) -> String {
    let lines: Vec<&str> = content.split('\n').collect();
    let ending_newline = content.ends_with('\n');
    if line_idx < lines.len() {
        let mut new_lines = Vec::new();
        for (i, &l) in lines.iter().enumerate() {
            if i == line_idx {
                new_lines.push(new_line.to_string());
            } else {
                new_lines.push(l.strip_suffix('\r').unwrap_or(l).to_string());
            }
        }
        let mut out = new_lines.join("\n");
        if ending_newline && !out.ends_with('\n') {
            out.push('\n');
        }
        out
    } else {
        content.to_string()
    }
}
