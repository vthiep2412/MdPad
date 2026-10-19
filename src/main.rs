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
    format_bold, format_bullet, format_checkbox, format_heading, format_italic, format_link,
    format_quote, format_table, parse_markdown_blocks, toggle_task_checkbox, BlockKind,
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

    if !opened_file {
        window.set_active_mode(1);
    }

    {
        let doc_ref = doc.borrow();
        window.set_window_title(doc_ref.window_title().into());
        window.set_doc_text(doc_ref.content().into());
        update_render_items(&window, doc_ref.content());
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
        if win.get_active_mode() == 0 || win.get_active_mode() == 1 {
            update_render_items(&win, d.content());
        }
    });

    let doc_clone = Rc::clone(&doc);
    let window_weak = window.as_weak();
    window.on_mode_selected(move |mode| {
        let Some(win) = window_weak.upgrade() else { return; };
        let d = doc_clone.borrow();
        if mode == 0 || mode == 1 {
            update_render_items(&win, d.content());
        }
    });

    let doc_clone = Rc::clone(&doc);
    let window_weak = window.as_weak();
    window.on_toggle_task_clicked(move |line_idx| {
        let Some(win) = window_weak.upgrade() else { return; };
        let mut d = doc_clone.borrow_mut();
        let new_content = toggle_task_checkbox(d.content(), line_idx as usize);
        d.set_content(new_content.clone());
        win.set_doc_text(new_content.into());
        win.set_window_title(d.window_title().into());
        sync_status_bar(&win, &d);
        update_render_items(&win, d.content());
    });

    let doc_clone = Rc::clone(&doc);
    let window_weak = window.as_weak();
    window.on_toolbar_click(move |action| {
        let Some(win) = window_weak.upgrade() else { return; };
        let mut d = doc_clone.borrow_mut();
        let mut content = d.content().to_string();

        let token = match action.as_str() {
            "heading" => format_heading("Heading"),
            "bold" => format_bold(""),
            "italic" => format_italic(""),
            "bullet" => format_bullet("List item"),
            "checkbox" => format_checkbox("Task item"),
            "quote" => format_quote("Quote text"),
            "link" => format_link(""),
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
            win.set_doc_text(content.into());
            win.set_window_title(d.window_title().into());
            sync_status_bar(&win, &d);
            update_render_items(&win, d.content());
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
        win.set_doc_text("".into());
        win.set_window_title(d.window_title().into());
        win.set_active_mode(1);
        sync_status_bar(&win, &d);
        update_render_items(&win, "");
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
                win.set_doc_text(d.content().into());
                win.set_window_title(d.window_title().into());
                win.set_active_mode(0);
                sync_status_bar(&win, &d);
                update_render_items(&win, d.content());
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

        let matches = engine.find_all(d.content(), query.as_str());
        if matches.is_empty() {
            win.set_match_status("No matches".into());
        } else {
            win.set_match_status(format!("{} found", matches.len()).into());
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

        let matches = engine.find_all(d.content(), query.as_str());
        if matches.is_empty() {
            win.set_match_status("No matches".into());
        } else {
            win.set_match_status(format!("{} found", matches.len()).into());
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

        let (new_text, replaced) = engine.replace_next(d.content(), query.as_str(), replacement.as_str(), 0);
        if replaced {
            d.set_content(new_text.clone());
            win.set_doc_text(new_text.into());
            win.set_window_title(d.window_title().into());
            sync_status_bar(&win, &d);
            update_render_items(&win, d.content());
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
            win.set_doc_text(new_text.into());
            win.set_window_title(d.window_title().into());
            sync_status_bar(&win, &d);
            update_render_items(&win, d.content());
            win.set_match_status(format!("Replaced {} matches", count).into());
        } else {
            win.set_match_status("No matches".into());
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
