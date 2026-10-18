use std::fs;
use std::time::Duration;

use mdpad::core::document::{CloseDecision, Document};
use mdpad::core::encoding::{
    encode_to_bytes, load_file, load_from_bytes, save_file, LineEnding, UTF8_BOM,
};
use mdpad::core::search::{SearchEngine, SearchMode};
use mdpad::core::watcher::DiskWatcher;

#[test]
fn test_utf8_without_bom_roundtrip() {
    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join("mdpad_test_nobom.md");

    let original_text = "# Header\n\nRust text with Unicode: 🦀 UTF-8 text.";
    fs::write(&temp_file, original_text).expect("Failed to write temp file");

    let loaded = load_file(&temp_file).expect("Failed to load file");
    assert!(!loaded.has_bom, "Document should not have BOM");
    assert_eq!(loaded.content, original_text);

    save_file(&temp_file, &loaded.content, loaded.has_bom, loaded.line_ending)
        .expect("Failed to save file");

    let saved_bytes = fs::read(&temp_file).expect("Failed to read saved file");
    assert!(!saved_bytes.starts_with(UTF8_BOM));
    assert_eq!(String::from_utf8(saved_bytes).unwrap(), original_text);

    let _ = fs::remove_file(temp_file);
}

#[test]
fn test_utf8_with_bom_roundtrip() {
    let temp_dir = std::env::temp_dir();
    let temp_file = temp_dir.join("mdpad_test_withbom.md");

    let text_content = "File with BOM marker.\nSecond line.\n";
    let mut initial_bytes = Vec::new();
    initial_bytes.extend_from_slice(UTF8_BOM);
    initial_bytes.extend_from_slice(text_content.as_bytes());
    fs::write(&temp_file, &initial_bytes).expect("Failed to write temp file with BOM");

    let loaded = load_file(&temp_file).expect("Failed to load file with BOM");
    assert!(loaded.has_bom, "Document must detect BOM presence");
    assert_eq!(
        loaded.content, text_content,
        "Loaded content must have BOM stripped from string"
    );

    save_file(&temp_file, &loaded.content, loaded.has_bom, loaded.line_ending)
        .expect("Failed to save file with BOM");

    let saved_bytes = fs::read(&temp_file).expect("Failed to read saved file");
    assert!(
        saved_bytes.starts_with(UTF8_BOM),
        "Saved file must preserve BOM marker"
    );
    assert_eq!(saved_bytes, initial_bytes);

    let _ = fs::remove_file(temp_file);
}

#[test]
fn test_line_ending_preservation_lf_and_crlf() {
    let lf_content = "Line 1\nLine 2\nLine 3\n";
    let loaded_lf = load_from_bytes(lf_content.as_bytes()).unwrap();
    assert_eq!(loaded_lf.line_ending, LineEnding::LF);
    let encoded_lf = encode_to_bytes(&loaded_lf.content, false, loaded_lf.line_ending);
    assert_eq!(String::from_utf8(encoded_lf).unwrap(), lf_content);

    let crlf_content = "Line 1\r\nLine 2\r\nLine 3\r\n";
    let loaded_crlf = load_from_bytes(crlf_content.as_bytes()).unwrap();
    assert_eq!(loaded_crlf.line_ending, LineEnding::CRLF);
    let encoded_crlf = encode_to_bytes(&loaded_crlf.content, false, loaded_crlf.line_ending);
    assert_eq!(String::from_utf8(encoded_crlf).unwrap(), crlf_content);

    let mixed = "Line 1\r\nLine 2\nLine 3\rLine 4";
    let normalized_crlf = LineEnding::CRLF.normalize(mixed);
    assert_eq!(normalized_crlf, "Line 1\r\nLine 2\r\nLine 3\r\nLine 4");
    let normalized_lf = LineEnding::LF.normalize(mixed);
    assert_eq!(normalized_lf, "Line 1\nLine 2\nLine 3\nLine 4");
}

#[test]
fn test_document_state_and_titles() {
    let mut doc = Document::new();
    assert_eq!(doc.window_title(), "Untitled - MdPad");
    assert!(!doc.is_dirty());

    doc.set_content("Modified content".to_string());
    assert!(doc.is_dirty());
    assert_eq!(doc.window_title(), "*Untitled - MdPad");

    let temp_path = std::path::PathBuf::from("workspace/notes.md");
    doc.set_path(Some(temp_path));
    assert_eq!(doc.window_title(), "*notes.md - MdPad");

    doc.set_dirty(false);
    assert_eq!(doc.window_title(), "notes.md - MdPad");
}

#[test]
fn test_document_stats_and_cursor() {
    let empty_doc = Document::new();
    let stats = empty_doc.stats();
    assert_eq!(stats.line_count, 1);
    assert_eq!(stats.word_count, 0);
    assert_eq!(stats.char_count, 0);

    let text = "First line here\nSecond line with words\nThird";
    let doc = Document::with_content(text.to_string());
    let doc_stats = doc.stats();
    assert_eq!(doc_stats.line_count, 3);
    assert_eq!(doc_stats.word_count, 8);
    assert_eq!(doc_stats.char_count, text.chars().count());

    assert_eq!(doc.cursor_position(0), (1, 1));
    assert_eq!(doc.cursor_position(5), (1, 6));
    assert_eq!(doc.cursor_position(16), (2, 1));
}

#[test]
fn test_close_decision_representation() {
    let decision = CloseDecision::Save;
    assert_eq!(decision, CloseDecision::Save);
    assert_ne!(decision, CloseDecision::DontSave);
    assert_ne!(decision, CloseDecision::Cancel);
}

#[test]
fn test_disk_watcher_external_change() {
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("mdpad_watcher_test.txt");

    fs::write(&test_file, "Initial data").unwrap();
    let mut watcher = DiskWatcher::for_path(&test_file);

    assert!(
        !watcher.check_for_external_change(),
        "No change right after initialization"
    );

    std::thread::sleep(Duration::from_millis(50));
    fs::write(&test_file, "Modified externally").unwrap();

    assert!(
        watcher.check_for_external_change(),
        "Must detect external change after file update"
    );
    assert!(
        !watcher.check_for_external_change(),
        "Subsequent check without modification must return false"
    );

    let _ = fs::remove_file(test_file);
}

#[test]
fn test_search_normal_case_and_word_boundaries() {
    let text = "The quick brown Fox jumps over the lazy dog. A fox is crafty.";

    let case_sens = SearchEngine::normal().with_case_sensitive(true);
    let results = case_sens.find_all(text, "Fox");
    assert_eq!(results.len(), 1);
    assert_eq!(&text[results[0].start_idx..results[0].end_idx], "Fox");

    let case_insens = SearchEngine::normal().with_case_sensitive(false);
    let results_insens = case_insens.find_all(text, "fox");
    assert_eq!(results_insens.len(), 2);

    let whole_word_text = "cat category bobcat cat";
    let non_whole = SearchEngine::normal().with_whole_word(false);
    assert_eq!(non_whole.find_all(whole_word_text, "cat").len(), 4);

    let whole = SearchEngine::normal().with_whole_word(true);
    let whole_results = whole.find_all(whole_word_text, "cat");
    assert_eq!(whole_results.len(), 2);
    assert_eq!(whole_results[0].start_idx, 0);
    assert_eq!(whole_results[0].end_idx, 3);
    assert_eq!(whole_results[1].start_idx, 20);
    assert_eq!(whole_results[1].end_idx, 23);
}

#[test]
fn test_search_regex_patterns_and_errors() {
    let text = "Order: #1234, SKU: #5678, Ref: 9999";
    let regex_engine = SearchEngine::regex();

    let matches = regex_engine.find_all(text, r"#\d{4}");
    assert_eq!(matches.len(), 2);
    assert_eq!(&text[matches[0].start_idx..matches[0].end_idx], "#1234");
    assert_eq!(&text[matches[1].start_idx..matches[1].end_idx], "#5678");

    let invalid_matches = regex_engine.find_all(text, "[unclosed-class");
    assert!(
        invalid_matches.is_empty(),
        "Invalid regex must return empty results without panicking"
    );
    assert!(regex_engine.validate_query("[unclosed-class").is_err());
    assert!(regex_engine.validate_query(r"#\d{4}").is_ok());
}

#[test]
fn test_search_fuzzy_matching_and_scoring() {
    let text = "DocumentController\nDataCenter\nDirectConnection";
    let fuzzy = SearchEngine::new().with_mode(SearchMode::Fuzzy);

    let matches = fuzzy.find_all(text, "DC");
    assert_eq!(matches.len(), 3);

    let tight_text = "mdpad";
    let loose_text = "m_extra_d_extra_p_a_d";
    let tight_score = fuzzy.find_all(tight_text, "mdpad")[0].score.unwrap();
    let loose_score = fuzzy.find_all(loose_text, "mdpad")[0].score.unwrap();
    assert!(
        tight_score > loose_score,
        "Contiguous match ({tight_score}) must outscore scattered match ({loose_score})"
    );
}

#[test]
fn test_search_navigation_find_next_and_prev() {
    let text = "rust is fast. rust is safe. rust is productive.";
    let engine = SearchEngine::normal();

    let first = engine.find_next(text, "rust", 0).unwrap();
    assert_eq!(first.start_idx, 0);

    let second = engine.find_next(text, "rust", 5).unwrap();
    assert_eq!(second.start_idx, 14);

    let third = engine.find_next(text, "rust", 20).unwrap();
    assert_eq!(third.start_idx, 28);

    let wrap_around = engine.find_next(text, "rust", 35).unwrap();
    assert_eq!(
        wrap_around.start_idx, 0,
        "Find next beyond end must wrap to first match"
    );

    let prev = engine.find_prev(text, "rust", 25).unwrap();
    assert_eq!(prev.start_idx, 14);

    let prev_wrap = engine.find_prev(text, "rust", 0).unwrap();
    assert_eq!(
        prev_wrap.start_idx, 28,
        "Find prev before first must wrap to last match"
    );
}

#[test]
fn test_search_replace_and_replace_all() {
    let text = "apple orange apple banana apple";
    let engine = SearchEngine::normal();

    let (replaced_single, success) = engine.replace(text, "apple", "peach");
    assert!(success);
    assert_eq!(replaced_single, "peach orange apple banana apple");

    let (replaced_all, count) = engine.replace_all(text, "apple", "grape");
    assert_eq!(count, 3);
    assert_eq!(replaced_all, "grape orange grape banana grape");

    let (empty_rep, _) = engine.replace_all(text, "apple", "");
    assert_eq!(empty_rep, " orange  banana ");

    let (next_rep, replaced) = engine.replace_next(text, "apple", "pear", 10);
    assert_eq!(next_rep, "apple orange pear banana apple");
    assert!(replaced);
}
