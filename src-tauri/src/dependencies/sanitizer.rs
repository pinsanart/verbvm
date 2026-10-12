//! Sanitizing of user-written HTML.
//!
//! Rich flashcard fields hold HTML and the view renders it as HTML, so it is sanitized before it
//! is stored.

use std::collections::HashSet;

/// Cleans user-written HTML or reduces it to plain text.
pub trait HtmlSanitizer: Send + Sync {
    /// Removes everything outside a small allow-list of formatting tags, including scripts, event
    /// handler attributes and `javascript:` links.
    fn sanitize_html(&self, html: &str) -> String;

    /// Removes every tag, keeping the text.
    fn strip_tags(&self, html: &str) -> String;
}

/// [`HtmlSanitizer`] backed by the `ammonia` crate.
pub struct AmmoniaSanitizer {
    rich: ammonia::Builder<'static>,
    plain: ammonia::Builder<'static>,
}

impl AmmoniaSanitizer {
    const RICH_TAGS: [&'static str; 14] = [
        "b", "strong", "i", "em", "u", "s", "br", "p", "div", "span", "ul", "ol", "li", "sub",
    ];

    pub fn new() -> Self {
        let mut rich = ammonia::Builder::default();
        rich.tags(Self::RICH_TAGS.into_iter().collect::<HashSet<_>>())
            .tag_attributes(Default::default())
            .generic_attributes(Default::default());

        let mut plain = ammonia::Builder::default();
        plain.tags(HashSet::new()).tag_attributes(Default::default()).generic_attributes(Default::default());

        Self { rich, plain }
    }
}

impl Default for AmmoniaSanitizer {
    fn default() -> Self {
        Self::new()
    }
}

impl HtmlSanitizer for AmmoniaSanitizer {
    fn sanitize_html(&self, html: &str) -> String {
        self.rich.clean(html).to_string()
    }

    fn strip_tags(&self, html: &str) -> String {
        // ammonia escapes the text for HTML. Plain fields hold text, so undo that escaping.
        decode_entities(&self.plain.clean(html).to_string())
    }
}

fn decode_entities(text: &str) -> String {
    text.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&amp;", "&")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_formatting_tags_and_drops_scripts() {
        let s = AmmoniaSanitizer::new();
        assert_eq!(s.sanitize_html("<b>raining</b> <i>cats</i>"), "<b>raining</b> <i>cats</i>");
        let cleaned = s.sanitize_html("hi<script>alert(1)</script><b>there</b>");
        assert!(!cleaned.contains("script") && !cleaned.contains("alert"));
        assert!(cleaned.contains("<b>there</b>"));
    }

    #[test]
    fn drops_event_handlers_and_links() {
        let s = AmmoniaSanitizer::new();
        let cleaned = s.sanitize_html(r#"<span onclick="x()">a</span><a href="javascript:x()">b</a><img src=x onerror=y>"#);
        assert!(!cleaned.contains("onclick") && !cleaned.contains("javascript") && !cleaned.contains("<img"));
    }

    #[test]
    fn strip_tags_keeps_the_text() {
        let s = AmmoniaSanitizer::new();
        assert_eq!(s.strip_tags("<b>Tom</b> & Jerry"), "Tom & Jerry");
        assert_eq!(s.strip_tags("a<script>alert(1)</script>b"), "ab");
    }
}
