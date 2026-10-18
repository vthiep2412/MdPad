#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SearchMode {
    #[default]
    Normal,
    Regex,
    Fuzzy,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchResult {
    pub start_idx: usize,
    pub end_idx: usize,
    pub score: Option<i64>,
}

impl SearchResult {
    pub fn new(start_idx: usize, end_idx: usize) -> Self {
        Self {
            start_idx,
            end_idx,
            score: None,
        }
    }

    pub fn with_score(start_idx: usize, end_idx: usize, score: i64) -> Self {
        Self {
            start_idx,
            end_idx,
            score: Some(score),
        }
    }
}

pub fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

pub fn is_whole_word_boundary(text: &str, start_idx: usize, end_idx: usize) -> bool {
    let before_ok = if start_idx == 0 {
        true
    } else {
        text[..start_idx]
            .chars()
            .next_back()
            .map_or(true, |c| !is_word_char(c))
    };

    let after_ok = if end_idx >= text.len() {
        true
    } else {
        text[end_idx..]
            .chars()
            .next()
            .map_or(true, |c| !is_word_char(c))
    };

    before_ok && after_ok
}

#[derive(Debug, Clone, Default)]
pub struct SearchEngine {
    pub case_sensitive: bool,
    pub whole_word: bool,
    pub mode: SearchMode,
}

impl SearchEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn normal() -> Self {
        Self {
            case_sensitive: false,
            whole_word: false,
            mode: SearchMode::Normal,
        }
    }

    pub fn regex() -> Self {
        Self {
            case_sensitive: false,
            whole_word: false,
            mode: SearchMode::Regex,
        }
    }

    pub fn fuzzy() -> Self {
        Self {
            case_sensitive: false,
            whole_word: false,
            mode: SearchMode::Fuzzy,
        }
    }

    pub fn with_case_sensitive(mut self, case_sensitive: bool) -> Self {
        self.case_sensitive = case_sensitive;
        self
    }

    pub fn with_whole_word(mut self, whole_word: bool) -> Self {
        self.whole_word = whole_word;
        self
    }

    pub fn with_mode(mut self, mode: SearchMode) -> Self {
        self.mode = mode;
        self
    }

    pub fn validate_query(&self, query: &str) -> Result<(), String> {
        if self.mode == SearchMode::Regex {
            regex::RegexBuilder::new(query)
                .case_insensitive(!self.case_sensitive)
                .build()
                .map(|_| ())
                .map_err(|e| e.to_string())
        } else {
            Ok(())
        }
    }

    pub fn find_all(&self, text: &str, query: &str) -> Vec<SearchResult> {
        if query.is_empty() || text.is_empty() {
            return Vec::new();
        }

        match self.mode {
            SearchMode::Normal => {
                find_normal_in_text(text, query, self.case_sensitive, self.whole_word)
            }
            SearchMode::Regex => {
                find_regex_in_text(text, query, self.case_sensitive, self.whole_word)
            }
            SearchMode::Fuzzy => {
                find_fuzzy_in_text(text, query, self.case_sensitive, self.whole_word)
            }
        }
    }

    pub fn find_next(&self, text: &str, query: &str, cursor_idx: usize) -> Option<SearchResult> {
        let results = self.find_all(text, query);
        if results.is_empty() {
            return None;
        }

        results
            .iter()
            .find(|r| r.start_idx >= cursor_idx)
            .cloned()
            .or_else(|| results.first().cloned())
    }

    pub fn find_prev(&self, text: &str, query: &str, cursor_idx: usize) -> Option<SearchResult> {
        let results = self.find_all(text, query);
        if results.is_empty() {
            return None;
        }

        results
            .iter()
            .rev()
            .find(|r| r.start_idx < cursor_idx)
            .cloned()
            .or_else(|| results.last().cloned())
    }

    pub fn replace_match(&self, text: &str, result: &SearchResult, replacement: &str) -> String {
        if result.start_idx > text.len()
            || result.end_idx > text.len()
            || result.start_idx > result.end_idx
        {
            return text.to_string();
        }
        let mut new_text = String::with_capacity(text.len() + replacement.len());
        new_text.push_str(&text[..result.start_idx]);
        new_text.push_str(replacement);
        new_text.push_str(&text[result.end_idx..]);
        new_text
    }

    pub fn replace(&self, text: &str, query: &str, replacement: &str) -> (String, bool) {
        self.replace_next(text, query, replacement, 0)
    }

    pub fn replace_next(
        &self,
        text: &str,
        query: &str,
        replacement: &str,
        cursor_idx: usize,
    ) -> (String, bool) {
        let result = self.find_next(text, query, cursor_idx);
        if let Some(ref m) = result {
            let updated = self.replace_match(text, m, replacement);
            (updated, true)
        } else {
            (text.to_string(), false)
        }
    }

    pub fn replace_all(&self, text: &str, query: &str, replacement: &str) -> (String, usize) {
        let matches = self.find_all(text, query);
        if matches.is_empty() {
            return (text.to_string(), 0);
        }

        let count = matches.len();
        let mut result = text.to_string();

        for m in matches.iter().rev() {
            result = self.replace_match(&result, m, replacement);
        }

        (result, count)
    }
}

fn find_normal_in_text(
    text: &str,
    query: &str,
    case_sensitive: bool,
    whole_word: bool,
) -> Vec<SearchResult> {
    let mut results = Vec::new();

    if case_sensitive {
        for (start, matched) in text.match_indices(query) {
            let end = start + matched.len();
            if !whole_word || is_whole_word_boundary(text, start, end) {
                results.push(SearchResult::new(start, end));
            }
        }
    } else {
        let escaped = regex::escape(query);
        if let Ok(re) = regex::RegexBuilder::new(&escaped)
            .case_insensitive(true)
            .build()
        {
            for m in re.find_iter(text) {
                let start = m.start();
                let end = m.end();
                if !whole_word || is_whole_word_boundary(text, start, end) {
                    results.push(SearchResult::new(start, end));
                }
            }
        }
    }

    results
}

fn find_regex_in_text(
    text: &str,
    query: &str,
    case_sensitive: bool,
    whole_word: bool,
) -> Vec<SearchResult> {
    let re = match regex::RegexBuilder::new(query)
        .case_insensitive(!case_sensitive)
        .build()
    {
        Ok(re) => re,
        Err(_) => return Vec::new(),
    };

    let mut results = Vec::new();
    for m in re.find_iter(text) {
        let start = m.start();
        let end = m.end();
        if start == end {
            continue;
        }
        if !whole_word || is_whole_word_boundary(text, start, end) {
            results.push(SearchResult::new(start, end));
        }
    }

    results
}

fn find_fuzzy_in_text(
    text: &str,
    query: &str,
    case_sensitive: bool,
    whole_word: bool,
) -> Vec<SearchResult> {
    let query_chars: Vec<char> = query.chars().collect();
    let mut results = Vec::new();
    let mut line_start_byte = 0;

    for line in text.split_inclusive('\n') {
        let line_len = line.len();
        let trimmed_line = line
            .strip_suffix("\r\n")
            .unwrap_or_else(|| line.strip_suffix('\n').unwrap_or(line));

        let line_matches = find_fuzzy_in_line(
            trimmed_line,
            &query_chars,
            case_sensitive,
            whole_word,
            line_start_byte,
        );
        results.extend(line_matches);

        line_start_byte += line_len;
    }

    results
}

fn find_fuzzy_in_line(
    line: &str,
    query_chars: &[char],
    case_sensitive: bool,
    whole_word: bool,
    line_offset: usize,
) -> Vec<SearchResult> {
    let mut matches = Vec::new();
    let line_char_indices: Vec<(usize, char)> = line.char_indices().collect();
    if line_char_indices.len() < query_chars.len() {
        return matches;
    }

    let mut start_search_from = 0;

    while start_search_from < line_char_indices.len() {
        let mut query_idx = 0;
        let mut forward_indices = Vec::with_capacity(query_chars.len());

        for i in start_search_from..line_char_indices.len() {
            let (_, c) = line_char_indices[i];
            let target = query_chars[query_idx];
            let is_match = if case_sensitive {
                c == target
            } else {
                c.to_lowercase().eq(target.to_lowercase())
            };

            if is_match {
                forward_indices.push(i);
                query_idx += 1;
                if query_idx == query_chars.len() {
                    break;
                }
            }
        }

        if query_idx < query_chars.len() {
            break;
        }

        let last_idx = *forward_indices.last().unwrap();
        let mut backward_indices = Vec::with_capacity(query_chars.len());
        let mut rev_query_idx = query_chars.len();

        for i in (0..=last_idx).rev() {
            let (_, c) = line_char_indices[i];
            let target = query_chars[rev_query_idx - 1];
            let is_match = if case_sensitive {
                c == target
            } else {
                c.to_lowercase().eq(target.to_lowercase())
            };

            if is_match {
                backward_indices.push(i);
                rev_query_idx -= 1;
                if rev_query_idx == 0 {
                    break;
                }
            }
        }
        backward_indices.reverse();

        let first_char_idx = backward_indices[0];
        let last_char_idx = *backward_indices.last().unwrap();

        let start_byte = line_char_indices[first_char_idx].0;
        let (last_byte, last_char) = line_char_indices[last_char_idx];
        let end_byte = last_byte + last_char.len_utf8();

        let doc_start = line_offset + start_byte;
        let doc_end = line_offset + end_byte;

        let whole_word_ok = if whole_word {
            is_whole_word_boundary(line, start_byte, end_byte)
        } else {
            true
        };

        if whole_word_ok {
            let score = calculate_fuzzy_score(
                line,
                &line_char_indices,
                &backward_indices,
                query_chars,
                case_sensitive,
            );
            matches.push(SearchResult::with_score(doc_start, doc_end, score));
        }

        start_search_from = last_char_idx + 1;
    }

    matches
}

fn calculate_fuzzy_score(
    line: &str,
    char_indices: &[(usize, char)],
    matched_char_indices: &[usize],
    query_chars: &[char],
    case_sensitive: bool,
) -> i64 {
    let mut score: i64 = 100;
    let query_len = query_chars.len();
    let first_idx = matched_char_indices[0];
    let last_idx = *matched_char_indices.last().unwrap();
    let span_len = last_idx - first_idx + 1;

    score -= (span_len as i64) * 2;

    if span_len == query_len {
        score += 50;
    }

    for (q_idx, &char_pos) in matched_char_indices.iter().enumerate() {
        let (byte_pos, c) = char_indices[char_pos];
        let target_q = query_chars[q_idx];

        if q_idx > 0 && char_pos == matched_char_indices[q_idx - 1] + 1 {
            score += 10;
        }

        if byte_pos == 0 {
            score += 15;
        } else {
            let prev_char = line[..byte_pos].chars().next_back();
            if let Some(pc) = prev_char {
                if !is_word_char(pc) {
                    score += 15;
                }
            }
        }

        if !case_sensitive && c == target_q {
            score += 2;
        }
    }

    score
}
