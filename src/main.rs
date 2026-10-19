#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
slint::include_modules!();

use std::cell::RefCell;
use std::env;
use std::path::PathBuf;
use std::rc::Rc;
use slint::{ComponentHandle, ModelRc, VecModel};

use mdpad::core::dialog::{open_file_dialog, save_file_dialog};
use mdpad::core::document::Document;
use mdpad::core::encoding::LineEnding;
use mdpad::core::search::{SearchEngine, SearchMode};
use mdpad::core::watcher::DiskWatcher;
use mdpad::markdown::{
    format_bold, format_bullet, format_checkbox, format_code_block, format_heading_level,
    format_inline_code, format_italic, format_link, format_numbered, format_quote,
    format_strikethrough, format_table, parse_live_items, parse_markdown_blocks,
    parse_source_lines, toggle_task_checkbox, update_line_in_text, BlockKind,
};

fn update_render_items(window: &AppWindow, content: &str) {
    let blocks = parse_markdown_blocks(content);
    let mut items = Vec::new();

    for block in blocks {
        let (kind, text, extra, checked) = match block.kind {
            BlockKind::Paragraph(p) => (0, p, String::new(), false),
            BlockKind::Heading(lvl, h) => {
                let kind = if lvl <= 3 { lvl as i32 } else { 6 + lvl as i32 };
                (kind, h, String::new(), false)
            }
            BlockKind::TaskItem(chk, txt, _line_idx) => (4, txt, String::new(), chk),
            BlockKind::ListItem(li) => (5, li, String::new(), false),
            BlockKind::Blockquote(bq) => (6, bq, String::new(), false),
            BlockKind::CodeBlock(lang, code) => (7, code, lang, false),
            BlockKind::ThematicBreak => (8, String::new(), String::new(), false),
            BlockKind::Table(headers, rows) => {
                let mut table_str = headers.join(" | ");
                table_str.push('\n');
                for row in rows {
                    table_str.push_str(&row.join(" | "));
                    table_str.push('\n');
                }
                (9, table_str, String::new(), false)
            }
        };

        items.push(RenderItem {
            kind,
            text: text.into(),
            extra: extra.into(),
            line_index: block.line_index as i32,
            checked,
        });
    }

    window.set_render_items(ModelRc::new(VecModel::from(items)));
}

fn update_source_lines(window: &AppWindow, content: &str) {
    let lines = parse_source_lines(content);
    let mut items = Vec::new();
    for l in lines {
        items.push(SourceLineItem {
            line_index: l.line_index as i32,
            raw_text: l.raw_text.into(),
            prefix: l.prefix.into(),
            content: l.content.into(),
            kind: l.kind,
            is_bold: l.is_bold,
        });
    }
    window.set_source_lines(ModelRc::new(VecModel::from(items)));
}

fn update_live_items(window: &AppWindow, content: &str) {
    let items_raw = parse_live_items(content);
    let mut items = Vec::new();
    for it in items_raw {
        items.push(LiveBlockItem {
            line_index: it.line_index as i32,
            raw_text: it.raw_text.into(),
            kind: it.kind,
            text: it.text.into(),
            checked: it.checked,
        });
    }
    window.set_live_items(ModelRc::new(VecModel::from(items)));
}

fn sync_all_views(window: &AppWindow, content: &str) {
    window.set_doc_text(content.into());
    update_render_items(window, content);
    update_source_lines(window, content);
    update_live_items(window, content);
}

fn sync_status_bar(window: &AppWindow, doc: &Document) {
    let stats = doc.stats();
    window.set_status_counts(format!("{} words, {} chars", stats.word_count, stats.char_count).into());
    window.set_status_line_col(format!("Ln 1, Col 1 ({} lines)", stats.line_count).into());

    let enc_str = if doc.has_bom() { "UTF-8 BOM" } else { "UTF-8" };
    window.set_status_encoding(enc_str.into());

    let ending_str = match doc.line_ending() {
        LineEnding::CRLF => "CRLF",
        LineEnding::LF => "LF",
    };
    window.set_status_line_ending(ending_str.into());
    window.set_status_zoom("100%".into());
}

fn main() -> Result<(), slint::PlatformError> {
    let window = AppWindow::new()?;

    let doc = Rc::new(RefCell::new(Document::new()));
    let watcher = Rc::new(RefCell::new(DiskWatcher::default()));
    let search_engine = Rc::new(RefCell::new(SearchEngine::default()));

    let args: Vec<String> = env::args().collect();
    let mut opened_file = false;
    if args.len() > 1 {
        let file_path = PathBuf::from(&args[1]);
        if file_path.exists() {
            if let Ok(loaded_doc) = Document::open(&file_path) {
                *doc.borrow_mut() = loaded_doc;
                watcher.borrow_mut().set_path(Some(file_path));
                opened_file = true;
            }
        } else {
            let mut d = doc.borrow_mut();
            d.set_path(Some(file_path.clone()));
            watcher.borrow_mut().set_path(Some(file_path));
        }
    }

    let initial_mode = if opened_file { 0 } else { 2 };
    window.set_active_mode(initial_mode);

    {
        let doc_ref = doc.borrow();
        window.set_window_title(doc_ref.window_title().into());
        sync_all_views(&window, doc_ref.content());
        sync_status_bar(&window, &doc_ref);
    }

    let doc_clone = Rc::clone(&doc);
    let window_weak = window.as_weak();
    window.on_text_edited(move |new_text| {
        let Some(win) = window_weak.upgrade() else { return; };
        let mut d = doc_clone.borrow_mut();
        d.set_content(new_text.to_string());
        win.set_window_title(d.window_title().into());
        sync_status_bar(&win, &d);
        update_render_items(&win, d.content());
        update_source_lines(&win, d.content());
        update_live_items(&win, d.content());
    });

    let doc_clone = Rc::clone(&doc);
    let window_weak = window.as_weak();
    window.on_source_line_edited(move |line_idx, new_line| {
        let Some(win) = window_weak.upgrade() else { return; };
        let mut d = doc_clone.borrow_mut();
        let updated = update_line_in_text(d.content(), line_idx as usize, new_line.as_str());
        d.set_content(updated);
        win.set_window_title(d.window_title().into());
        sync_status_bar(&win, &d);
        win.set_doc_text(d.content().into());
        update_render_items(&win, d.content());
        update_live_items(&win, d.content());
    });

    let doc_clone = Rc::clone(&doc);
    let window_weak = window.as_weak();
    window.on_source_line_committed(move |_line_idx, _new_line| {
        let Some(win) = window_weak.upgrade() else { return; };
        let d = doc_clone.borrow();
        win.set_active_source_line(-1);
        update_source_lines(&win, d.content());
    });

    let doc_clone = Rc::clone(&doc);
    let window_weak = window.as_weak();
    window.on_live_line_edited(move |line_idx, new_line| {
        let Some(win) = window_weak.upgrade() else { return; };
        let mut d = doc_clone.borrow_mut();
        let updated = update_line_in_text(d.content(), line_idx as usize, new_line.as_str());
        d.set_content(updated);
        win.set_window_title(d.window_title().into());
        sync_status_bar(&win, &d);
        win.set_doc_text(d.content().into());
        update_render_items(&win, d.content());
        update_source_lines(&win, d.content());
    });

    let doc_clone = Rc::clone(&doc);
    let window_weak = window.as_weak();
    window.on_live_line_committed(move |_line_idx, _new_line| {
        let Some(win) = window_weak.upgrade() else { return; };
        let d = doc_clone.borrow();
        win.set_active_live_line(-1);
        update_live_items(&win, d.content());
    });

    let doc_clone = Rc::clone(&doc);
    let window_weak = window.as_weak();
    window.on_mode_selected(move |_mode| {
        let Some(win) = window_weak.upgrade() else { return; };
        let d = doc_clone.borrow();
        win.set_active_source_line(-1);
        win.set_active_live_line(-1);
        sync_all_views(&win, d.content());
    });

    let doc_clone = Rc::clone(&doc);
    let window_weak = window.as_weak();
    window.on_toggle_task_clicked(move |line_idx| {
        let Some(win) = window_weak.upgrade() else { return; };
        let mut d = doc_clone.borrow_mut();
        let new_content = toggle_task_checkbox(d.content(), line_idx as usize);
        d.set_content(new_content.clone());
        win.set_window_title(d.window_title().into());
        sync_status_bar(&win, &d);
        sync_all_views(&win, &new_content);
    });

    let doc_clone = Rc::clone(&doc);
    let window_weak = window.as_weak();
    window.on_toolbar_click(move |action| {
        let Some(win) = window_weak.upgrade() else { return; };
        let mut d = doc_clone.borrow_mut();
        let mut content = d.content().to_string();

        let token = match action.as_str() {
            "h1" => format_heading_level(1, "Heading 1"),
            "h2" => format_heading_level(2, "Heading 2"),
            "h3" => format_heading_level(3, "Heading 3"),
            "bold" => format_bold("bold text"),
            "italic" => format_italic("italic text"),
            "strike" => format_strikethrough("strikethrough text"),
            "bullet" => format_bullet("List item"),
            "numbered" => format_numbered(1, "Numbered item"),
            "checkbox" => format_checkbox("Task item"),
            "quote" => format_quote("Quote text"),
            "inline_code" => format_inline_code("code"),
            "code_block" => format_code_block().to_string(),
            "link" => format_link("link text"),
            "table" => format_table().to_string(),
            _ => String::new(),
        };

        if !token.is_empty() {
            if !content.is_empty() && !content.ends_with('\n') {
                content.push('\n');
            }
            content.push_str(&token);
            content.push('\n');

            d.set_content(content.clone());
            win.set_window_title(d.window_title().into());
            sync_status_bar(&win, &d);
            sync_all_views(&win, &content);
        }
    });

    let doc_clone = Rc::clone(&doc);
    let watcher_clone = Rc::clone(&watcher);
    let window_weak = window.as_weak();
    window.on_save_doc(move || {
        let Some(win) = window_weak.upgrade() else { return; };
        let mut d = doc_clone.borrow_mut();
        if d.path().is_none() {
            if let Some(target_path) = save_file_dialog("Untitled.md") {
                let _ = d.save_as(&target_path);
                watcher_clone.borrow_mut().set_path(Some(target_path));
            }
        } else {
            let _ = d.save();
        }
        win.set_window_title(d.window_title().into());
        sync_status_bar(&win, &d);
    });

    let doc_clone = Rc::clone(&doc);
    let watcher_clone = Rc::clone(&watcher);
    let window_weak = window.as_weak();
    window.on_save_as_doc(move || {
        let Some(win) = window_weak.upgrade() else { return; };
        let mut d = doc_clone.borrow_mut();
        let default_name = d
            .path()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("Untitled.md");
        if let Some(target_path) = save_file_dialog(default_name) {
            let _ = d.save_as(&target_path);
            watcher_clone.borrow_mut().set_path(Some(target_path));
            win.set_window_title(d.window_title().into());
            sync_status_bar(&win, &d);
        }
    });

    let doc_clone = Rc::clone(&doc);
    let watcher_clone = Rc::clone(&watcher);
    let window_weak = window.as_weak();
    window.on_new_doc(move || {
        let Some(win) = window_weak.upgrade() else { return; };
        let mut d = doc_clone.borrow_mut();
        if d.is_dirty() {
            win.set_dialog_text("Save changes before creating a new document?".into());
            win.set_unsaved_dialog_open(true);
            return;
        }
        *d = Document::new();
        watcher_clone.borrow_mut().set_path::<PathBuf>(None);
        win.set_window_title(d.window_title().into());
        win.set_active_mode(2);
        sync_status_bar(&win, &d);
        sync_all_views(&win, "");
    });

    let doc_clone = Rc::clone(&doc);
    let watcher_clone = Rc::clone(&watcher);
    let window_weak = window.as_weak();
    window.on_open_doc(move || {
        let Some(win) = window_weak.upgrade() else { return; };
        let mut d = doc_clone.borrow_mut();
        if d.is_dirty() {
            win.set_dialog_text("Save changes before opening another document?".into());
            win.set_unsaved_dialog_open(true);
            return;
        }
        if let Some(picked_path) = open_file_dialog() {
            if let Ok(loaded) = Document::open(&picked_path) {
                *d = loaded;
                watcher_clone.borrow_mut().set_path(Some(picked_path));
                win.set_window_title(d.window_title().into());
                win.set_active_mode(0);
                sync_status_bar(&win, &d);
                sync_all_views(&win, d.content());
            }
        }
    });

    let window_weak = window.as_weak();
    window.on_trigger_find(move || {
        if let Some(win) = window_weak.upgrade() {
            win.set_find_visible(true);
        }
    });

    let window_weak = window.as_weak();
    window.on_trigger_replace(move || {
        if let Some(win) = window_weak.upgrade() {
            win.set_find_visible(true);
            win.set_replace_visible(true);
        }
    });

    let window_weak = window.as_weak();
    window.on_close_find_bar(move || {
        if let Some(win) = window_weak.upgrade() {
            win.set_find_visible(false);
            win.set_replace_visible(false);
        }
    });

    let doc_clone = Rc::clone(&doc);
    let engine_clone = Rc::clone(&search_engine);
    let window_weak = window.as_weak();
    window.on_find_next_clicked(move || {
        let Some(win) = window_weak.upgrade() else { return; };
        let d = doc_clone.borrow();
        let query = win.get_find_query();

        let mut engine = engine_clone.borrow_mut();
        engine.case_sensitive = win.get_search_case_sensitive();
        engine.whole_word = win.get_search_whole_word();
        engine.mode = match win.get_search_engine_mode() {
            1 => SearchMode::Fuzzy,
            2 => SearchMode::Regex,
            _ => SearchMode::Normal,
        };

        let results = engine.find_all(d.content(), query.as_str());
        if results.is_empty() {
            win.set_match_status("0 of 0".into());
        } else {
            win.set_match_status(format!("1 of {}", results.len()).into());
        }
    });

    let doc_clone = Rc::clone(&doc);
    let engine_clone = Rc::clone(&search_engine);
    let window_weak = window.as_weak();
    window.on_find_prev_clicked(move || {
        let Some(win) = window_weak.upgrade() else { return; };
        let d = doc_clone.borrow();
        let query = win.get_find_query();

        let mut engine = engine_clone.borrow_mut();
        engine.case_sensitive = win.get_search_case_sensitive();
        engine.whole_word = win.get_search_whole_word();
        engine.mode = match win.get_search_engine_mode() {
            1 => SearchMode::Fuzzy,
            2 => SearchMode::Regex,
            _ => SearchMode::Normal,
        };

        let results = engine.find_all(d.content(), query.as_str());
        if results.is_empty() {
            win.set_match_status("0 of 0".into());
        } else {
            win.set_match_status(format!("{} of {}", results.len(), results.len()).into());
        }
    });

    let doc_clone = Rc::clone(&doc);
    let engine_clone = Rc::clone(&search_engine);
    let window_weak = window.as_weak();
    window.on_replace_clicked(move || {
        let Some(win) = window_weak.upgrade() else { return; };
        let mut d = doc_clone.borrow_mut();
        let query = win.get_find_query();
        let replacement = win.get_replace_query();

        let mut engine = engine_clone.borrow_mut();
        engine.case_sensitive = win.get_search_case_sensitive();
        engine.whole_word = win.get_search_whole_word();
        engine.mode = match win.get_search_engine_mode() {
            1 => SearchMode::Fuzzy,
            2 => SearchMode::Regex,
            _ => SearchMode::Normal,
        };

        let (new_text, replaced) = engine.replace(d.content(), query.as_str(), replacement.as_str());
        if replaced {
            d.set_content(new_text.clone());
            win.set_window_title(d.window_title().into());
            sync_status_bar(&win, &d);
            sync_all_views(&win, &new_text);
            win.set_match_status("Replaced".into());
        } else {
            win.set_match_status("No match".into());
        }
    });

    let doc_clone = Rc::clone(&doc);
    let engine_clone = Rc::clone(&search_engine);
    let window_weak = window.as_weak();
    window.on_replace_all_clicked(move || {
        let Some(win) = window_weak.upgrade() else { return; };
        let mut d = doc_clone.borrow_mut();
        let query = win.get_find_query();
        let replacement = win.get_replace_query();

        let mut engine = engine_clone.borrow_mut();
        engine.case_sensitive = win.get_search_case_sensitive();
        engine.whole_word = win.get_search_whole_word();
        engine.mode = match win.get_search_engine_mode() {
            1 => SearchMode::Fuzzy,
            2 => SearchMode::Regex,
            _ => SearchMode::Normal,
        };

        let (new_text, count) = engine.replace_all(d.content(), query.as_str(), replacement.as_str());
        if count > 0 {
            d.set_content(new_text.clone());
            win.set_window_title(d.window_title().into());
            sync_status_bar(&win, &d);
            sync_all_views(&win, &new_text);
            win.set_match_status(format!("Replaced {} matches", count).into());
        } else {
            win.set_match_status("No matches".into());
        }
    });

    let window_weak = window.as_weak();
    window.on_menu_action(move |action| {
        let Some(win) = window_weak.upgrade() else { return; };
        match action.as_str() {
            "exit" => {
                let _ = win.hide();
            }
            "about" => {
                win.set_dialog_text("MdPad v0.1.0\nLightweight native Markdown reader and editor".into());
                win.set_unsaved_dialog_open(true);
            }
            _ => {}
        }
    });

    let doc_clone = Rc::clone(&doc);
    let watcher_clone = Rc::clone(&watcher);
    let window_weak = window.as_weak();
    window.on_dialog_action(move |choice| {
        let Some(win) = window_weak.upgrade() else { return; };
        let mut d = doc_clone.borrow_mut();
        match choice {
            0 => {
                if d.path().is_none() {
                    if let Some(target_path) = save_file_dialog("Untitled.md") {
                        let _ = d.save_as(&target_path);
                        watcher_clone.borrow_mut().set_path(Some(target_path));
                    }
                } else {
                    let _ = d.save();
                }
                win.set_window_title(d.window_title().into());
                sync_status_bar(&win, &d);
                win.set_unsaved_dialog_open(false);
            }
            1 => {
                d.set_dirty(false);
                win.set_window_title(d.window_title().into());
                sync_status_bar(&win, &d);
                win.set_unsaved_dialog_open(false);
            }
            _ => {
                win.set_unsaved_dialog_open(false);
            }
        }
    });

    window.run()
}
