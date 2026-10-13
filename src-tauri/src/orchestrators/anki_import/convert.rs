//! How the notes of the owner's Anki collection become verbvm cards. Pure functions over the HTML
//! of the fields, so the rules can be tested without a collection.
//!
//! The collection has one note type (Front, Back). A *Pronúncia* note has the sentence in the front
//! and `<strong>word</strong>: /ipa/<br>` lines in the back. A *Vocabulário* note has the sentence
//! (with the studied word in `<strong>`) in the front and `<strong>word:</strong> definition` in
//! the back, possibly one `<strong>word</strong>: gloss` line per word, and sometimes an image.

/// One piece of HTML: text, or a tag with its name and attributes.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Token {
    Text(String),
    Tag { name: String, closing: bool, attributes: String },
}

fn decode(text: &str) -> String {
    let mut decoded = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(start) = rest.find('&') {
        decoded.push_str(&rest[..start]);
        rest = &rest[start..];
        let Some(end) = rest.find(';').filter(|end| *end <= 8) else {
            decoded.push('&');
            rest = &rest[1..];
            continue;
        };
        let entity = &rest[1..end];
        let replacement = match entity {
            "nbsp" => Some(' '),
            "amp" => Some('&'),
            "lt" => Some('<'),
            "gt" => Some('>'),
            "quot" => Some('"'),
            "apos" => Some('\''),
            _ => entity
                .strip_prefix('#')
                .and_then(|n| match n.strip_prefix(['x', 'X']) {
                    Some(hex) => u32::from_str_radix(hex, 16).ok(),
                    None => n.parse().ok(),
                })
                .and_then(char::from_u32),
        };
        match replacement {
            Some(c) => {
                decoded.push(c);
                rest = &rest[end + 1..];
            }
            None => {
                decoded.push('&');
                rest = &rest[1..];
            }
        }
    }
    decoded.push_str(rest);
    decoded
}

fn tokenize(html: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut rest = html;
    while !rest.is_empty() {
        if let Some(stripped) = rest.strip_prefix('<') {
            if let Some(end) = stripped.find('>') {
                let inside = &stripped[..end];
                let closing = inside.starts_with('/');
                let inside = inside.trim_start_matches('/').trim_end_matches('/');
                let (name, attributes) = inside.split_once(char::is_whitespace).unwrap_or((inside, ""));
                if name.chars().next().is_some_and(|c| c.is_ascii_alphabetic()) {
                    tokens.push(Token::Tag {
                        name: name.to_ascii_lowercase(),
                        closing,
                        attributes: attributes.trim().to_owned(),
                    });
                    rest = &stripped[end + 1..];
                    continue;
                }
            }
        }
        // The text runs to the next '<', and always takes at least its first character, which may
        // be more than one byte.
        let first = rest.chars().next().map_or(0, char::len_utf8);
        let next = rest[first..].find('<').map_or(rest.len(), |i| i + first);
        tokens.push(Token::Text(decode(&rest[..next])));
        rest = &rest[next..];
    }
    tokens
}

/// The text of some tokens, with each `<br>` as `line_break`, spaces collapsed and every line
/// trimmed. With a real line break, empty lines are dropped.
fn text_of(tokens: &[Token], line_break: &str) -> String {
    let mut text = String::new();
    for token in tokens {
        match token {
            Token::Text(t) => text.push_str(t),
            Token::Tag { name, closing: false, .. } if name == "br" => text.push('\n'),
            Token::Tag { .. } => {}
        }
    }
    let lines: Vec<String> = text
        .split('\n')
        .map(|line| line.split_whitespace().collect::<Vec<_>>().join(" "))
        .filter(|line| !line.is_empty())
        .collect();
    lines.join(line_break)
}

/// The text of a field, with line breaks as spaces: what the sentence says, whatever its markup.
pub fn plain(html: &str) -> String {
    text_of(&tokenize(html), " ")
}

/// The text of a field with its line breaks kept.
fn plain_lines(html: &str) -> String {
    text_of(&tokenize(html), "\n")
}

/// What two notes must share to be the same sentence.
pub fn sentence_key(front_html: &str) -> String {
    plain(front_html)
}

/// The files of the images in a field, in order.
pub fn images(html: &str) -> Vec<String> {
    tokenize(html)
        .into_iter()
        .filter_map(|token| match token {
            Token::Tag { name, closing: false, attributes } if name == "img" => attribute(&attributes, "src"),
            _ => None,
        })
        .map(|name| percent_decode(&name))
        .collect()
}

fn attribute(attributes: &str, wanted: &str) -> Option<String> {
    let lower = attributes.to_ascii_lowercase();
    let mut from = 0;
    while let Some(found) = lower[from..].find(wanted) {
        let start = from + found;
        let before_ok = start == 0 || lower.as_bytes()[start - 1].is_ascii_whitespace();
        let after = attributes[start + wanted.len()..].trim_start();
        if before_ok {
            if let Some(value) = after.strip_prefix('=') {
                let value = value.trim_start();
                return Some(match value.chars().next() {
                    Some(quote @ ('"' | '\'')) => value[1..].split(quote).next().unwrap_or("").to_owned(),
                    _ => value.split_whitespace().next().unwrap_or("").to_owned(),
                });
            }
        }
        from = start + wanted.len();
    }
    None
}

fn percent_decode(name: &str) -> String {
    let bytes = name.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' {
            let hex = bytes.get(i + 1..i + 3).and_then(|h| std::str::from_utf8(h).ok());
            if let Some(byte) = hex.and_then(|h| u8::from_str_radix(h, 16).ok()) {
                out.push(byte);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8(out).unwrap_or_else(|_| name.to_owned())
}

/// One `<strong>word</strong>: text` part of a vocabulary back.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Entry {
    word: String,
    text: String,
}

/// The entries of a vocabulary back: each starts with a `<strong>` at the beginning of a line (the
/// start of the field, or after a `<br>`). Bold in the middle of a line is part of the text.
fn entries(back_html: &str) -> Vec<Entry> {
    let tokens = tokenize(back_html);
    let mut entries: Vec<(String, Vec<Token>)> = Vec::new();
    let mut line_start = true;
    let mut i = 0;
    while i < tokens.len() {
        match &tokens[i] {
            Token::Tag { name, closing: false, .. } if name == "strong" && line_start => {
                let mut word = Vec::new();
                i += 1;
                while i < tokens.len() && !matches!(&tokens[i], Token::Tag { name, closing: true, .. } if name == "strong") {
                    word.push(tokens[i].clone());
                    i += 1;
                }
                entries.push((text_of(&word, " "), Vec::new()));
                line_start = false;
            }
            Token::Tag { name, closing: false, .. } if name == "br" => {
                if let Some((_, rest)) = entries.last_mut() {
                    rest.push(tokens[i].clone());
                }
                line_start = true;
            }
            Token::Text(text) if text.trim().is_empty() => {
                if let Some((_, rest)) = entries.last_mut() {
                    rest.push(tokens[i].clone());
                }
            }
            other => {
                if matches!(other, Token::Text(_)) {
                    line_start = false;
                }
                if let Some((_, rest)) = entries.last_mut() {
                    rest.push(other.clone());
                }
            }
        }
        i += 1;
    }
    entries
        .into_iter()
        .map(|(word, rest)| Entry {
            word: word.trim().trim_end_matches(':').trim().to_owned(),
            text: text_of(&rest, "\n").trim_start_matches(':').trim().to_owned(),
        })
        .collect()
}

/// The sentence of a vocabulary note. A sentence that is bold from end to end (the German and
/// Italian notes) has no bold that points at a word, so the bold is dropped and the studied word,
/// when there is one, is the only bold left.
fn sentence_html(front_html: &str) -> String {
    let html = front_html.replace("&nbsp;", " ");
    let tokens = tokenize(&html);
    let is_strong = |token: &Token, closing: bool| {
        matches!(token, Token::Tag { name, closing: c, .. } if name == "strong" && *c == closing)
    };
    let text_outside_bold = || {
        let mut depth = 0;
        tokens.iter().any(|token| {
            if is_strong(token, false) {
                depth += 1;
            } else if is_strong(token, true) {
                depth -= 1;
            }
            matches!(token, Token::Text(text) if depth == 0 && !text.trim().is_empty())
        })
    };
    let has_bold = tokens.iter().any(|t| is_strong(t, false));
    if has_bold && !text_outside_bold() {
        return plain(&html);
    }
    html.trim().to_owned()
}

/// A *Vocabulário* note as the fields of a vocabulary card.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Vocabulary {
    /// The sentence as HTML, with the studied word in bold.
    pub phrase: String,
    pub word_or_expression: String,
    pub meaning: String,
    /// The media files of the images in the back, to attach to the meaning.
    pub images: Vec<String>,
    /// Set when the note had to be filled in with a guess.
    pub warning: Option<String>,
}

/// One entry: the word and its text. Several entries: the words, and a line `word: text` for each.
/// No entry (no bold at the start of a line): the sentence is the word and the back is the meaning.
pub fn vocabulary(front_html: &str, back_html: &str) -> Vocabulary {
    let phrase = sentence_html(front_html);
    let found = entries(back_html);
    let images = images(back_html);
    let (word, mut meaning) = match found.as_slice() {
        [] => (plain(front_html), plain_lines(back_html)),
        [only] => (only.word.clone(), only.text.clone()),
        many => (
            many.iter().map(|e| e.word.as_str()).collect::<Vec<_>>().join(", "),
            many.iter()
                .map(|e| if e.text.is_empty() { e.word.clone() } else { format!("{}: {}", e.word, e.text) })
                .collect::<Vec<_>>()
                .join("\n"),
        ),
    };
    let mut word = word;
    let mut warning = None;
    if word.is_empty() {
        word = plain(front_html);
    }
    if meaning.is_empty() && images.is_empty() {
        // The card needs a meaning, so it gets the word and the note is reported.
        meaning = word.clone();
        warning = Some(format!("'{}' has no meaning in its back", plain(front_html)));
    }
    Vocabulary { phrase, word_or_expression: word, meaning, images, warning }
}

/// A *Pronúncia* note as the fields of a phonetic card.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Phonetic {
    /// The sentence as plain text.
    pub phrase: String,
    /// The lines `<b>word</b>: /ipa/`, as the generator writes them.
    pub ipa: String,
}

pub fn phonetic(front_html: &str, back_html: &str) -> Phonetic {
    let mut ipa = back_html.replace("&nbsp;", " ").replace("<strong>", "<b>").replace("</strong>", "</b>");
    ipa = ipa.trim().to_owned();
    while let Some(stripped) = ipa.strip_suffix("<br>").or_else(|| ipa.strip_suffix("<br/>")).or_else(|| ipa.strip_suffix("<br />")) {
        ipa = stripped.trim_end().to_owned();
    }
    Phonetic { phrase: plain(front_html), ipa }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_text_drops_the_markup_and_normalizes_spaces() {
        assert_eq!(plain("Le <strong>téléphone</strong>  vibre&nbsp;ici"), "Le téléphone vibre ici");
        assert_eq!(plain("a<br>b <br/> c"), "a b c");
        assert_eq!(plain("Tom &amp; Jerry &lt;3 &#233;"), "Tom & Jerry <3 é");
        assert_eq!(plain("<strong>Sie sollten die U-Bahn nehmen.</strong>"), "Sie sollten die U-Bahn nehmen.");
        assert_eq!(plain("a < b"), "a < b", "a bare angle bracket is text");
        assert_eq!(plain("él<b>à</b>ñandú <ü"), "élàñandú <ü", "accented characters are not cut");
        assert_eq!(plain("é"), "é");
    }

    #[test]
    fn two_notes_are_the_same_sentence_when_their_text_matches() {
        assert_eq!(
            sentence_key("Conseguimos, <strong>a lo más</strong>,  medio kilo de azúcar."),
            sentence_key("Conseguimos, a lo más, medio kilo de azúcar.")
        );
        assert_ne!(sentence_key("It's my elder brother."), sentence_key("It's my younger brother."));
    }

    #[test]
    fn a_single_entry_gives_the_word_and_its_meaning() {
        let v = vocabulary(
            "It's my <strong>elder</strong> brother.",
            "<strong>elder:</strong> (of people, especially two members of the same family) older",
        );
        assert_eq!(v.phrase, "It's my <strong>elder</strong> brother.");
        assert_eq!(v.word_or_expression, "elder");
        assert_eq!(v.meaning, "(of people, especially two members of the same family) older");
        assert!(v.images.is_empty() && v.warning.is_none());
    }

    #[test]
    fn a_sentence_that_is_bold_from_end_to_end_loses_that_bold() {
        let v = vocabulary("<strong>Sie sollten die U-Bahn nehmen.</strong>", "<strong>Sie</strong>: você");
        assert_eq!(v.phrase, "Sie sollten die U-Bahn nehmen.");
        let kept = vocabulary("Le <strong>téléphone</strong> vibre.", "<strong>téléphone:</strong> appareil");
        assert_eq!(kept.phrase, "Le <strong>téléphone</strong> vibre.", "a bold that points at a word stays");
        let several = vocabulary("<strong>a lo más</strong>, <strong>medio</strong> kilo", "<strong>a:</strong> b");
        assert_eq!(several.phrase, "<strong>a lo más</strong>, <strong>medio</strong> kilo", "text between two bolds is not bold");
        assert_eq!(vocabulary("sem negrito", "<strong>x:</strong> y").phrase, "sem negrito");
    }

    #[test]
    fn an_italian_note_has_the_sentence_as_the_word_and_the_translation_as_the_meaning() {
        let v = vocabulary(
            "<strong>Dovresti prendere la metropolitana.</strong>",
            "<strong>Dovresti prendere la metropolitana.:</strong> Você deveria pegar o metrô.",
        );
        assert_eq!(v.word_or_expression, "Dovresti prendere la metropolitana.");
        assert_eq!(v.meaning, "Você deveria pegar o metrô.");
    }

    #[test]
    fn several_entries_give_the_words_and_one_line_each() {
        let v = vocabulary(
            "<strong>Sie sollten die U-Bahn nehmen.</strong>",
            "<strong>Sie</strong>: você<br><strong>sollten</strong>: deveria(m) (sollen, konj. II)<br><strong>U-Bahn</strong>: metrô (f., die U-Bahn)",
        );
        assert_eq!(v.word_or_expression, "Sie, sollten, U-Bahn");
        assert_eq!(v.meaning, "Sie: você\nsollten: deveria(m) (sollen, konj. II)\nU-Bahn: metrô (f., die U-Bahn)");
    }

    #[test]
    fn bold_in_the_middle_of_a_line_is_part_of_the_text() {
        let v = vocabulary("x <strong>pot</strong>", "<strong>pot:</strong> a vessel (see <strong>pan</strong>)<br>second line");
        assert_eq!(v.word_or_expression, "pot");
        assert_eq!(v.meaning, "a vessel (see pan)\nsecond line");
    }

    #[test]
    fn an_image_is_a_media_file_and_can_be_the_whole_meaning() {
        let v = vocabulary(
            "As <strong>hebillas</strong> do carro",
            "<strong>hebillas:</strong>&nbsp;<br><img alt=\"4 Piezas Hebilla\" src=\"YWwud2VicA.jpg\">",
        );
        assert_eq!(v.images, vec!["YWwud2VicA.jpg"]);
        assert_eq!(v.meaning, "", "only the image says what it means");
        assert!(v.warning.is_none(), "an image satisfies the required meaning");

        let single = images("<img src='a%20b.png'><IMG SRC=c.webp>");
        assert_eq!(single, vec!["a b.png", "c.webp"]);
    }

    #[test]
    fn a_back_without_a_bold_start_falls_back_to_the_sentence_and_the_whole_back() {
        let v = vocabulary("The <strong>pan</strong> sat.", "a utensil for cooking");
        assert_eq!(v.word_or_expression, "The pan sat.");
        assert_eq!(v.meaning, "a utensil for cooking");
        let empty = vocabulary("The <strong>pan</strong> sat.", "<strong>pan:</strong>");
        assert_eq!(empty.meaning, "pan");
        assert!(empty.warning.is_some(), "a note with no meaning at all is reported");
    }

    #[test]
    fn the_pronunciation_keeps_its_lines_with_bold_words_as_b() {
        let p = phonetic(
            "Sie sollten die U-Bahn nehmen.",
            "<strong>sie</strong>: /ziː/<br><strong>nächstgelegene</strong>: <br><strong>ist</strong>: /ɪst/, /ɪs/<br>",
        );
        assert_eq!(p.phrase, "Sie sollten die U-Bahn nehmen.");
        assert_eq!(p.ipa, "<b>sie</b>: /ziː/<br><b>nächstgelegene</b>: <br><b>ist</b>: /ɪst/, /ɪs/");
    }
}
