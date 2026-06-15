//! Port of tinyxml2 `XMLUtil`: entity encode/decode, bool parsing, BOM, predicates.

/// UTF-8 BOM byte sequence.
pub const BOM: &str = "\u{feff}";

/// Strip a leading UTF-8 BOM. Returns the remainder and whether a BOM was present.
pub fn strip_bom(input: &str) -> (&str, bool) {
    match input.strip_prefix(BOM) {
        Some(rest) => (rest, true),
        None => (input, false),
    }
}

/// Decode the five predefined XML entities plus numeric character references.
/// Unrecognized `&...;` sequences are passed through verbatim (tinyxml2 behavior).
pub fn decode_entities(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let bytes = input.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'&' {
            if let Some((decoded, consumed)) = decode_one_entity(&input[i..]) {
                out.push(decoded);
                i += consumed;
                continue;
            }
        }
        let ch = input[i..].chars().next().unwrap();
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}

/// Try to decode a single entity starting at `&`. Returns (char, bytes_consumed).
fn decode_one_entity(s: &str) -> Option<(char, usize)> {
    let end = s.find(';')?;
    let body = &s[1..end];
    let consumed = end + 1;
    let ch = match body {
        "lt" => '<',
        "gt" => '>',
        "amp" => '&',
        "apos" => '\'',
        "quot" => '"',
        _ if body.starts_with("#x") || body.starts_with("#X") => {
            let code = u32::from_str_radix(&body[2..], 16).ok()?;
            char::from_u32(code)?
        }
        _ if body.starts_with('#') => {
            let code = body[1..].parse::<u32>().ok()?;
            char::from_u32(code)?
        }
        _ => return None,
    };
    Some((ch, consumed))
}

/// Escape characters that must be escaped in element text / attribute values.
/// Note: apostrophe is NOT escaped (matches tinyxml2 default text behavior).
pub fn encode_text(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            _ => out.push(ch),
        }
    }
    out
}

/// Parse a bool the way tinyxml2 does: accepts true/false/1/0, case-insensitive.
pub fn parse_bool(s: &str) -> Option<bool> {
    let s = s.trim();
    match s {
        _ if s.eq_ignore_ascii_case("true") => Some(true),
        _ if s.eq_ignore_ascii_case("false") => Some(false),
        "1" => Some(true),
        "0" => Some(false),
        _ => None,
    }
}

/// tinyxml2 `IsWhiteSpace`: space, tab, CR, LF.
pub fn is_whitespace(ch: char) -> bool {
    matches!(ch, ' ' | '\t' | '\r' | '\n')
}

/// Collapse runs of whitespace to a single space and trim ends (tinyxml2 COLLAPSE_WHITESPACE).
pub fn collapse_whitespace(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut in_ws = false;
    for ch in input.trim().chars() {
        if is_whitespace(ch) {
            if !in_ws {
                out.push(' ');
                in_ws = true;
            }
        } else {
            out.push(ch);
            in_ws = false;
        }
    }
    out
}

/// tinyxml2 `IsNameStartChar`.
pub fn is_name_start_char(ch: char) -> bool {
    ch as u32 >= 128 || ch.is_ascii_alphabetic() || ch == ':' || ch == '_'
}

/// tinyxml2 `IsNameChar`.
pub fn is_name_char(ch: char) -> bool {
    is_name_start_char(ch) || ch.is_ascii_digit() || ch == '.' || ch == '-'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_named_and_numeric_entities() {
        assert_eq!(
            decode_entities("a &lt; b &amp; c &gt; &apos;&quot;"),
            "a < b & c > '\""
        );
        assert_eq!(decode_entities("&#65;&#x42;"), "AB");
        assert_eq!(decode_entities("no entities"), "no entities");
        assert_eq!(decode_entities("&unknown;"), "&unknown;");
    }

    #[test]
    fn encode_escapes_required_chars() {
        assert_eq!(
            encode_text("a < b & c > \"x\" 'y'"),
            "a &lt; b &amp; c &gt; &quot;x&quot; 'y'"
        );
    }

    #[test]
    fn parse_bool_accepts_tinyxml2_forms() {
        assert_eq!(parse_bool("true"), Some(true));
        assert_eq!(parse_bool("false"), Some(false));
        assert_eq!(parse_bool("1"), Some(true));
        assert_eq!(parse_bool("0"), Some(false));
        assert_eq!(parse_bool("True"), Some(true));
        assert_eq!(parse_bool(" true "), Some(true));
        assert_eq!(parse_bool(" 0 "), Some(false));
        assert_eq!(parse_bool("maybe"), None);
    }

    #[test]
    fn strip_bom_removes_utf8_bom() {
        let with = "\u{feff}<a/>";
        assert_eq!(strip_bom(with), ("<a/>", true));
        assert_eq!(strip_bom("<a/>"), ("<a/>", false));
    }
}
