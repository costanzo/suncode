

use unicode_normalization::UnicodeNormalization;





pub(crate) fn clean_page(raw: &str) -> String {
    let mut normalized = String::with_capacity(raw.len());
    for character in raw.chars() {
        if is_compatibility_form(character) {
            normalized.extend(std::iter::once(character).nfkc());
        } else {
            normalized.push(character);
        }
    }
    let mut lines: Vec<&str> = Vec::new();
    for line in normalized.lines().map(str::trim_end) {
        if line.is_empty() && lines.last().is_none_or(|last: &&str| last.is_empty()) {
            continue;
        }
        lines.push(line);
    }
    while lines.last().is_some_and(|line| line.is_empty()) {
        lines.pop();
    }
    lines.join("\n")
}

fn is_compatibility_form(character: char) -> bool {
    matches!(
        character,
        '\u{2E80}'..='\u{2FDF}' | '\u{F900}'..='\u{FAFF}' | '\u{FB00}'..='\u{FB06}'
    )
}




pub(crate) struct Folded {
    pub(crate) text: String,
    origins: Vec<usize>,
}

impl Folded {
    pub(crate) fn new(source: &str) -> Self {
        let mut text = String::with_capacity(source.len());
        let mut origins = Vec::with_capacity(source.len());
        let mut in_space = false;
        for (index, character) in source.chars().enumerate() {
            if character.is_whitespace() {
                if !in_space {
                    text.push(' ');
                    origins.push(index);
                    in_space = true;
                }
                continue;
            }
            in_space = false;
            for lower in character.to_lowercase() {
                text.push(lower);
                origins.extend(std::iter::repeat_n(index, lower.len_utf8()));
            }
        }
        Self { text, origins }
    }

    pub(crate) fn source_range(&self, start: usize, end: usize) -> (usize, usize) {
        (self.origins[start], self.origins[end - 1] + 1)
    }
}


pub(crate) fn fold_query(query: &str) -> String {
    Folded::new(query.trim()).text
}


pub(crate) fn char_slice(text: &str, start: usize, end: usize) -> &str {
    let byte = |position: usize| {
        text.char_indices()
            .nth(position)
            .map_or(text.len(), |(index, _)| index)
    };
    let start_byte = byte(start);
    let end_byte = start_byte
        + text[start_byte..]
            .char_indices()
            .nth(end - start)
            .map_or(text.len() - start_byte, |(index, _)| index);
    &text[start_byte..end_byte]
}


pub(crate) fn collapse_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}
