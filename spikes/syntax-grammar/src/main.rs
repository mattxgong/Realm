use std::collections::BTreeMap;
use std::process::ExitCode;
use std::str;

use unicode_ident::{UNICODE_VERSION as IDENT_UNICODE_VERSION, is_xid_continue, is_xid_start};
use unicode_normalization::{
    UNICODE_VERSION as NORMALIZATION_UNICODE_VERSION, UnicodeNormalization,
};

const KEYWORDS: &[&str] = &[
    "actor",
    "as",
    "async",
    "await",
    "break",
    "catch",
    "const",
    "continue",
    "dyn",
    "else",
    "enum",
    "false",
    "fn",
    "for",
    "if",
    "import",
    "in",
    "interface",
    "let",
    "loop",
    "macro",
    "match",
    "move",
    "mut",
    "pub",
    "return",
    "select",
    "spawn",
    "struct",
    "throw",
    "throws",
    "trait",
    "true",
    "type",
    "unsafe",
    "var",
    "where",
    "while",
    "yield",
];

const SYMBOLS: &[&str] = &[
    "<<=", ">>=", "::", "->", "=>", "+=", "-=", "*=", "/=", "%=", "&=", "|=", "^=", "<<", ">>",
    "&&", "||", "==", "!=", "<=", ">=", "(", ")", "{", "}", "[", "]", ",", ";", ":", ".", "=", "+",
    "-", "*", "/", "%", "!", "&", "|", "^", "<", ">",
];

const MAX_SYNTAX_DIAGNOSTICS_PER_CONSTRUCT: usize = 8;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TokenKind {
    Trivia,
    Discard,
    Identifier,
    Keyword,
    Number,
    Character,
    String,
    Symbol,
    InvalidUtf8,
    InvalidCharacter,
    EndOfFile,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Token {
    kind: TokenKind,
    start: usize,
    end: usize,
}

impl Token {
    fn text<'source>(&self, source: &'source [u8]) -> &'source str {
        str::from_utf8(&source[self.start..self.end]).unwrap_or("")
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Diagnostic {
    code: &'static str,
    start: usize,
    end: usize,
}

#[derive(Debug)]
struct Lexed {
    tokens: Vec<Token>,
    diagnostics: Vec<Diagnostic>,
}

fn decode_at(source: &[u8], offset: usize) -> Result<(char, usize), usize> {
    let tail = &source[offset..];
    match str::from_utf8(tail) {
        Ok(valid) => {
            let character = valid
                .chars()
                .next()
                .expect("offset is before end of source");
            Ok((character, character.len_utf8()))
        }
        Err(error) if error.valid_up_to() > 0 => {
            let valid_prefix = str::from_utf8(&tail[..error.valid_up_to()])
                .expect("valid_up_to identifies a valid UTF-8 prefix");
            let character = valid_prefix
                .chars()
                .next()
                .expect("a non-empty valid prefix has a character");
            Ok((character, character.len_utf8()))
        }
        Err(error) => Err(error.error_len().unwrap_or(tail.len()).max(1)),
    }
}

fn is_default_ignorable(character: char) -> bool {
    matches!(
        character as u32,
        0x00AD
            | 0x034F
            | 0x061C
            | 0x115F..=0x1160
            | 0x17B4..=0x17B5
            | 0x180B..=0x180F
            | 0x200B..=0x200F
            | 0x202A..=0x202E
            | 0x2060..=0x206F
            | 0x3164
            | 0xFE00..=0xFE0F
            | 0xFEFF
            | 0xFFA0
            | 0x1BCA0..=0x1BCA3
            | 0x1D173..=0x1D17A
            | 0xE0000..=0xE0FFF
    )
}

fn is_line_terminator(character: char) -> bool {
    matches!(
        character,
        '\n' | '\r' | '\u{000B}' | '\u{000C}' | '\u{0085}' | '\u{2028}' | '\u{2029}'
    )
}

fn is_horizontal_pattern_whitespace(character: char) -> bool {
    matches!(character, '\t' | ' ')
}

fn is_identifier_start(character: char) -> bool {
    (character == '_' || is_xid_start(character)) && !is_default_ignorable(character)
}

fn is_identifier_continue(character: char) -> bool {
    is_xid_continue(character) && !is_default_ignorable(character)
}

fn push_token(tokens: &mut Vec<Token>, kind: TokenKind, start: usize, end: usize) {
    tokens.push(Token { kind, start, end });
}

fn push_literal_diagnostic(
    diagnostics: &mut Vec<Diagnostic>,
    code: &'static str,
    start: usize,
    end: usize,
) {
    diagnostics.push(Diagnostic { code, start, end });
}

fn validate_literal(source: &[u8], token: Token, diagnostics: &mut Vec<Diagnostic>) {
    let delimiter = match token.kind {
        TokenKind::Character => '\'',
        TokenKind::String => '"',
        _ => return,
    };
    let text = str::from_utf8(&source[token.start..token.end])
        .expect("literal tokens contain valid UTF-8 segments");
    let terminated = text.len() >= 2 && text.ends_with(delimiter);
    let content_end = text.len() - usize::from(terminated);
    let mut offset = delimiter.len_utf8();
    let mut scalar_count = 0usize;

    while offset < content_end {
        let character = text[offset..]
            .chars()
            .next()
            .expect("offset points inside literal text");
        let character_start = token.start + offset;
        offset += character.len_utf8();

        if character == '\0' {
            push_literal_diagnostic(
                diagnostics,
                "LEX_INVALID_LITERAL_CHARACTER",
                character_start,
                character_start + 1,
            );
            scalar_count += 1;
            continue;
        }

        if character != '\\' {
            scalar_count += 1;
            continue;
        }

        let escape_start = character_start;
        if offset >= content_end {
            push_literal_diagnostic(
                diagnostics,
                "LEX_INVALID_ESCAPE",
                escape_start,
                token.start + content_end,
            );
            scalar_count += 1;
            continue;
        }

        let escaped = text[offset..]
            .chars()
            .next()
            .expect("escape has a following character");
        offset += escaped.len_utf8();
        if matches!(escaped, '0' | '\\' | '"' | '\'' | 'n' | 'r' | 't') {
            scalar_count += 1;
            continue;
        }

        if escaped == 'u' && text[offset..content_end].starts_with('{') {
            offset += 1;
            let digits_start = offset;
            while offset < content_end && text.as_bytes()[offset].is_ascii_hexdigit() {
                offset += 1;
            }
            let digits = &text[digits_start..offset];
            let has_closing_brace = offset < content_end && text.as_bytes()[offset] == b'}';
            if has_closing_brace {
                offset += 1;
            }
            let scalar_is_valid = (1..=6).contains(&digits.len())
                && has_closing_brace
                && u32::from_str_radix(digits, 16)
                    .ok()
                    .and_then(char::from_u32)
                    .is_some();
            if !scalar_is_valid {
                push_literal_diagnostic(
                    diagnostics,
                    "LEX_INVALID_ESCAPE",
                    escape_start,
                    token.start + offset,
                );
            }
            scalar_count += 1;
            continue;
        }

        push_literal_diagnostic(
            diagnostics,
            "LEX_INVALID_ESCAPE",
            escape_start,
            token.start + offset,
        );
        scalar_count += 1;
    }

    if token.kind == TokenKind::Character && scalar_count != 1 {
        push_literal_diagnostic(
            diagnostics,
            "LEX_INVALID_CHARACTER_LITERAL",
            token.start,
            token.end,
        );
    }
}

fn scan(source: &[u8]) -> Lexed {
    let mut tokens = Vec::new();
    let mut diagnostics = Vec::new();
    let mut offset = 0;

    if source.starts_with(&[0xEF, 0xBB, 0xBF]) {
        push_token(&mut tokens, TokenKind::Trivia, 0, 3);
        offset = 3;
    }

    while offset < source.len() {
        if source[offset..].starts_with(b"//") {
            let start = offset;
            offset += 2;
            while offset < source.len() {
                match decode_at(source, offset) {
                    Ok((character, _)) if is_line_terminator(character) => break,
                    Ok((_, width)) => offset += width,
                    Err(_) => break,
                }
            }
            push_token(&mut tokens, TokenKind::Trivia, start, offset);
            continue;
        }

        if source[offset..].starts_with(b"/*") {
            let start = offset;
            let mut depth = 1usize;
            offset += 2;
            while offset < source.len() && depth > 0 {
                if source[offset..].starts_with(b"/*") {
                    depth += 1;
                    offset += 2;
                } else if source[offset..].starts_with(b"*/") {
                    depth -= 1;
                    offset += 2;
                } else {
                    match decode_at(source, offset) {
                        Ok((_, width)) => offset += width,
                        Err(_) => break,
                    }
                }
            }
            if offset < source.len() && decode_at(source, offset).is_err() {
                push_token(&mut tokens, TokenKind::Trivia, start, offset);
                continue;
            }
            if depth > 0 {
                diagnostics.push(Diagnostic {
                    code: "LEX_UNTERMINATED_BLOCK_COMMENT",
                    start,
                    end: (start + 2).min(source.len()),
                });
            }
            push_token(&mut tokens, TokenKind::Trivia, start, offset);
            continue;
        }

        let (character, width) = match decode_at(source, offset) {
            Ok(decoded) => decoded,
            Err(invalid_width) => {
                let end = (offset + invalid_width).min(source.len());
                push_token(&mut tokens, TokenKind::InvalidUtf8, offset, end);
                diagnostics.push(Diagnostic {
                    code: "LEX_INVALID_UTF8",
                    start: offset,
                    end,
                });
                offset = end;
                continue;
            }
        };

        if is_line_terminator(character) {
            let start = offset;
            offset += width;
            if character == '\r'
                && offset < source.len()
                && let Ok(('\n', next_width)) = decode_at(source, offset)
            {
                offset += next_width;
            }
            push_token(&mut tokens, TokenKind::Trivia, start, offset);
            continue;
        }

        if matches!(character, '\u{200E}' | '\u{200F}') {
            push_token(&mut tokens, TokenKind::Trivia, offset, offset + width);
            offset += width;
            continue;
        }

        if is_horizontal_pattern_whitespace(character) {
            let start = offset;
            offset += width;
            while offset < source.len() {
                match decode_at(source, offset) {
                    Ok((next, next_width)) if is_horizontal_pattern_whitespace(next) => {
                        offset += next_width;
                    }
                    _ => break,
                }
            }
            push_token(&mut tokens, TokenKind::Trivia, start, offset);
            continue;
        }

        if is_identifier_start(character) {
            let start = offset;
            offset += width;
            while offset < source.len() {
                match decode_at(source, offset) {
                    Ok((next, next_width)) if is_identifier_continue(next) => offset += next_width,
                    _ => break,
                }
            }
            let spelling = str::from_utf8(&source[start..offset])
                .expect("identifier characters are valid UTF-8");
            let kind = if spelling == "_" {
                TokenKind::Discard
            } else if KEYWORDS.contains(&spelling) {
                TokenKind::Keyword
            } else {
                TokenKind::Identifier
            };
            push_token(&mut tokens, kind, start, offset);
            continue;
        }

        if character.is_ascii_digit() {
            let start = offset;
            offset = scan_number_end(source, offset);
            let spelling = str::from_utf8(&source[start..offset])
                .expect("numeric candidates contain ASCII only");
            if !is_valid_number(spelling) {
                diagnostics.push(Diagnostic {
                    code: "LEX_MALFORMED_NUMBER",
                    start,
                    end: offset,
                });
            }
            push_token(&mut tokens, TokenKind::Number, start, offset);
            continue;
        }

        if matches!(character, '\'' | '"') {
            let delimiter = character;
            let kind = if delimiter == '\'' {
                TokenKind::Character
            } else {
                TokenKind::String
            };
            let start = offset;
            offset += width;
            let mut terminated = false;
            while offset < source.len() {
                match decode_at(source, offset) {
                    Ok((next, _)) if next == delimiter => {
                        offset += next.len_utf8();
                        terminated = true;
                        break;
                    }
                    Ok((next, _)) if is_line_terminator(next) => break,
                    Ok(('\\', slash_width)) => {
                        offset += slash_width;
                        if offset < source.len() {
                            offset += decode_at(source, offset).map_or_else(
                                |invalid_width| invalid_width,
                                |(_, next_width)| next_width,
                            );
                        }
                    }
                    Ok((_, next_width)) => offset += next_width,
                    Err(_) => break,
                }
            }
            if offset < source.len() && decode_at(source, offset).is_err() {
                push_token(&mut tokens, kind, start, offset);
                continue;
            }
            if !terminated {
                diagnostics.push(Diagnostic {
                    code: "LEX_UNTERMINATED_LITERAL",
                    start,
                    end: offset,
                });
            }
            let token = Token {
                kind,
                start,
                end: offset,
            };
            validate_literal(source, token, &mut diagnostics);
            tokens.push(token);
            continue;
        }

        let symbol = SYMBOLS
            .iter()
            .find(|symbol| source[offset..].starts_with(symbol.as_bytes()));
        if let Some(symbol) = symbol {
            let end = offset + symbol.len();
            push_token(&mut tokens, TokenKind::Symbol, offset, end);
            offset = end;
            continue;
        }

        let end = offset + width;
        let code = if matches!(character, '\0' | '\u{FEFF}') {
            "LEX_INVALID_CHARACTER"
        } else if is_default_ignorable(character) {
            "LEX_DISALLOWED_IDENTIFIER_CHARACTER"
        } else {
            "LEX_INVALID_CHARACTER"
        };
        diagnostics.push(Diagnostic {
            code,
            start: offset,
            end,
        });
        push_token(&mut tokens, TokenKind::InvalidCharacter, offset, end);
        offset = end;
    }

    push_token(
        &mut tokens,
        TokenKind::EndOfFile,
        source.len(),
        source.len(),
    );
    diagnostics.sort_by_key(|diagnostic| (diagnostic.start, diagnostic.end, diagnostic.code));

    Lexed {
        tokens,
        diagnostics,
    }
}

fn scan_number_end(source: &[u8], start: usize) -> usize {
    if source[start..].starts_with(b"0b")
        || source[start..].starts_with(b"0B")
        || source[start..].starts_with(b"0o")
        || source[start..].starts_with(b"0O")
        || source[start..].starts_with(b"0x")
        || source[start..].starts_with(b"0X")
    {
        let mut end = start + 2;
        while end < source.len() && (source[end].is_ascii_alphanumeric() || source[end] == b'_') {
            end += 1;
        }
        return end;
    }

    let mut end = start;
    let mut saw_exponent = false;
    while end < source.len() {
        match source[end] {
            byte if byte.is_ascii_digit() || byte == b'_' || byte == b'.' => end += 1,
            b'e' | b'E' if !saw_exponent => {
                saw_exponent = true;
                end += 1;
                if end < source.len() && matches!(source[end], b'+' | b'-') {
                    end += 1;
                }
            }
            _ => break,
        }
    }
    end
}

fn valid_digit_sequence(text: &str, valid_digit: impl Fn(char) -> bool) -> bool {
    !text.is_empty()
        && !text.starts_with('_')
        && !text.ends_with('_')
        && !text.contains("__")
        && text
            .chars()
            .all(|character| character == '_' || valid_digit(character))
}

fn is_valid_number(text: &str) -> bool {
    for (prefix, radix) in [
        ("0b", 2),
        ("0B", 2),
        ("0o", 8),
        ("0O", 8),
        ("0x", 16),
        ("0X", 16),
    ] {
        if let Some(digits) = text.strip_prefix(prefix) {
            return valid_digit_sequence(digits, |character| character.is_digit(radix));
        }
    }

    let mut exponent_parts = text.split(['e', 'E']);
    let mantissa = exponent_parts.next().unwrap_or("");
    let exponent = exponent_parts.next();
    if exponent_parts.next().is_some() {
        return false;
    }

    let exponent_is_valid = exponent.is_none_or(|part| {
        let digits = part.strip_prefix(['+', '-']).unwrap_or(part);
        valid_digit_sequence(digits, |character| character.is_ascii_digit())
    });
    if !exponent_is_valid {
        return false;
    }

    let mut decimal_parts = mantissa.split('.');
    let integer = decimal_parts.next().unwrap_or("");
    let fraction = decimal_parts.next();
    if decimal_parts.next().is_some() {
        return false;
    }

    valid_digit_sequence(integer, |character| character.is_ascii_digit())
        && fraction.is_none_or(|digits| {
            digits.is_empty()
                || valid_digit_sequence(digits, |character| character.is_ascii_digit())
        })
}

fn validate_partition(source: &[u8], tokens: &[Token]) -> bool {
    let mut expected_start = 0;
    for token in tokens
        .iter()
        .filter(|token| token.kind != TokenKind::EndOfFile)
    {
        if token.start != expected_start || token.end <= token.start {
            return false;
        }
        expected_start = token.end;
    }
    expected_start == source.len()
}

fn normalized_identifier_collisions(source: &[u8], tokens: &[Token]) -> Vec<(String, usize)> {
    let mut identities: BTreeMap<String, usize> = BTreeMap::new();
    let mut collisions = Vec::new();
    for token in tokens
        .iter()
        .filter(|token| token.kind == TokenKind::Identifier)
    {
        let identity: String = token.text(source).nfc().collect();
        if let Some(previous) = identities.insert(identity.clone(), token.start) {
            collisions.push((identity, previous));
        }
    }
    collisions
}

fn representative_confusable_skeleton(identifier: &str) -> String {
    identifier
        .chars()
        .map(|character| match character {
            '\u{0430}' => 'a',
            '\u{043E}' | '\u{03BF}' => 'o',
            _ => character,
        })
        .collect()
}

fn push_syntax_diagnostic(
    diagnostics: &mut Vec<Diagnostic>,
    syntax_diagnostic_count: &mut usize,
    diagnostic: Diagnostic,
) {
    if *syntax_diagnostic_count < MAX_SYNTAX_DIAGNOSTICS_PER_CONSTRUCT {
        diagnostics.push(diagnostic);
    } else if *syntax_diagnostic_count == MAX_SYNTAX_DIAGNOSTICS_PER_CONSTRUCT {
        diagnostics.push(Diagnostic {
            code: "PARSE_DIAGNOSTICS_SUPPRESSED",
            start: diagnostic.start,
            end: diagnostic.end,
        });
    }
    *syntax_diagnostic_count += 1;
}

fn is_statement_starter(token: &Token, source: &[u8]) -> bool {
    matches!(
        token.text(source),
        "let" | "var" | "return" | "break" | "continue" | "throw"
    )
}

fn is_operator(token: &Token, source: &[u8]) -> bool {
    matches!(
        token.text(source),
        "=" | "+" | "-" | "*" | "/" | "%" | "&" | "|" | "^" | "<<" | ">>"
    )
}

fn parse_recovery(source: &[u8]) -> Vec<Diagnostic> {
    let lexed = scan(source);
    let tokens: Vec<Token> = lexed
        .tokens
        .into_iter()
        .filter(|token| token.kind != TokenKind::Trivia)
        .collect();
    let mut diagnostics = lexed.diagnostics;
    let mut syntax_diagnostic_count = 0usize;
    let mut index = tokens
        .iter()
        .position(|token| token.text(source) == "{")
        .map_or(0, |brace| brace + 1);

    while index < tokens.len() {
        let current = tokens[index];
        if matches!(current.kind, TokenKind::EndOfFile) || current.text(source) == "}" {
            break;
        }

        let statement_start = index;
        let mut delimiter_stack: Vec<&str> = Vec::new();
        let mut previous_was_operator = false;
        let starts_with_statement_keyword = is_statement_starter(&current, source);

        loop {
            let token = tokens[index];
            let text = token.text(source);

            if index > statement_start
                && delimiter_stack.is_empty()
                && is_statement_starter(&token, source)
            {
                push_syntax_diagnostic(
                    &mut diagnostics,
                    &mut syntax_diagnostic_count,
                    Diagnostic {
                        code: "PARSE_MISSING_SEMICOLON",
                        start: token.start,
                        end: token.start,
                    },
                );
                break;
            }

            if text == ";" {
                while delimiter_stack.pop().is_some() {
                    push_syntax_diagnostic(
                        &mut diagnostics,
                        &mut syntax_diagnostic_count,
                        Diagnostic {
                            code: "PARSE_MISSING_CLOSING_DELIMITER",
                            start: token.start,
                            end: token.start,
                        },
                    );
                }
                if previous_was_operator {
                    push_syntax_diagnostic(
                        &mut diagnostics,
                        &mut syntax_diagnostic_count,
                        Diagnostic {
                            code: "PARSE_MISSING_EXPRESSION",
                            start: token.start,
                            end: token.start,
                        },
                    );
                }
                index += 1;
                break;
            }

            if matches!(token.kind, TokenKind::EndOfFile) {
                while delimiter_stack.pop().is_some() {
                    push_syntax_diagnostic(
                        &mut diagnostics,
                        &mut syntax_diagnostic_count,
                        Diagnostic {
                            code: "PARSE_MISSING_CLOSING_DELIMITER",
                            start: token.start,
                            end: token.start,
                        },
                    );
                }
                if starts_with_statement_keyword {
                    push_syntax_diagnostic(
                        &mut diagnostics,
                        &mut syntax_diagnostic_count,
                        Diagnostic {
                            code: "PARSE_MISSING_SEMICOLON",
                            start: token.start,
                            end: token.start,
                        },
                    );
                }
                index = tokens.len();
                break;
            }

            if delimiter_stack.is_empty() && text == "}" {
                if starts_with_statement_keyword {
                    push_syntax_diagnostic(
                        &mut diagnostics,
                        &mut syntax_diagnostic_count,
                        Diagnostic {
                            code: "PARSE_MISSING_SEMICOLON",
                            start: token.start,
                            end: token.start,
                        },
                    );
                }
                break;
            }

            match text {
                "(" => delimiter_stack.push(")"),
                "[" => delimiter_stack.push("]"),
                ")" | "]" => {
                    if delimiter_stack.last().copied() == Some(text) {
                        delimiter_stack.pop();
                    } else {
                        push_syntax_diagnostic(
                            &mut diagnostics,
                            &mut syntax_diagnostic_count,
                            Diagnostic {
                                code: "PARSE_UNEXPECTED_CLOSING_DELIMITER",
                                start: token.start,
                                end: token.end,
                            },
                        );
                    }
                }
                _ => {}
            }

            previous_was_operator = is_operator(&token, source);
            index += 1;
            if index >= tokens.len() {
                break;
            }
        }
    }

    diagnostics.sort_by_key(|diagnostic| (diagnostic.start, diagnostic.end, diagnostic.code));
    diagnostics
}

fn normalize_index(index: i64, length: usize) -> Option<usize> {
    let length = length as u128;
    if index >= 0 {
        let normalized = index as u128;
        (normalized < length).then_some(normalized as usize)
    } else {
        let magnitude = index.unsigned_abs() as u128;
        (magnitude <= length && magnitude != 0).then(|| (length - magnitude) as usize)
    }
}

fn normalize_bound(bound: i64, length: usize) -> Option<usize> {
    let length = length as u128;
    if bound >= 0 {
        let normalized = bound as u128;
        (normalized <= length).then_some(normalized as usize)
    } else {
        let magnitude = bound.unsigned_abs() as u128;
        (magnitude <= length).then(|| (length - magnitude) as usize)
    }
}

fn normalize_slice(start: Option<i64>, end: Option<i64>, length: usize) -> Option<(usize, usize)> {
    let start = start.map_or(Some(0), |bound| normalize_bound(bound, length))?;
    let end = end.map_or(Some(length), |bound| normalize_bound(bound, length))?;
    (start <= end).then_some((start, end))
}

fn run_conformance() -> Result<(), String> {
    if IDENT_UNICODE_VERSION != (17, 0, 0) {
        return Err(format!(
            "REQ-002 expected Unicode 17.0.0 identifier tables, got {IDENT_UNICODE_VERSION:?}"
        ));
    }
    if NORMALIZATION_UNICODE_VERSION != (17, 0, 0) {
        return Err(format!(
            "REQ-002 expected Unicode 17.0.0 normalization tables, got {NORMALIZATION_UNICODE_VERSION:?}"
        ));
    }

    let valid = "let café = values[-1]; /* outer /* nested */ done */".as_bytes();
    let lexed = scan(valid);
    if !lexed.diagnostics.is_empty() || !validate_partition(valid, &lexed.tokens) {
        return Err("REQ-001/REQ-005 valid-source partition failed".into());
    }

    let malformed = b"let value = \xF0\x28\x8C\x28;";
    let malformed_lexed = scan(malformed);
    if !validate_partition(malformed, &malformed_lexed.tokens)
        || !malformed_lexed
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "LEX_INVALID_UTF8")
    {
        return Err("REQ-001 malformed UTF-8 recovery failed".into());
    }

    let equivalent = "let café = 1; let cafe\u{301} = 2;".as_bytes();
    let equivalent_lexed = scan(equivalent);
    if normalized_identifier_collisions(equivalent, &equivalent_lexed.tokens).len() != 1 {
        return Err("REQ-002 NFC identity check failed".into());
    }
    if representative_confusable_skeleton("v\u{0430}r") != "var" {
        return Err("REQ-002 Unicode 17 confusable fixture failed".into());
    }

    let recovery = br#"fn recover(input: [i32]) -> i32 {
    let first = input[0]
    let middle = input[1:3;
    return first + ;
}"#;
    let recovery_diagnostics = parse_recovery(recovery);
    if recovery_diagnostics.len() != 3 {
        return Err(format!(
            "REQ-003/REQ-004 expected 3 recovery diagnostics, got {recovery_diagnostics:?}"
        ));
    }

    if normalize_index(-1, 4) != Some(3)
        || normalize_index(-4, 4) != Some(0)
        || normalize_index(-5, 4).is_some()
        || normalize_index(i64::MIN, 4).is_some()
    {
        return Err("REQ-006 index normalization failed".into());
    }

    if normalize_slice(None, None, 4) != Some((0, 4))
        || normalize_slice(Some(-2), None, 4) != Some((2, 4))
        || normalize_slice(Some(3), Some(1), 4).is_some()
        || normalize_slice(None, Some(-5), 4).is_some()
    {
        return Err("REQ-007 slice normalization failed".into());
    }

    Ok(())
}

fn main() -> ExitCode {
    match run_conformance() {
        Ok(()) => {
            println!("RLM-0001 initial spike checks passed; full conformance remains pending.");
            ExitCode::SUCCESS
        }
        Err(message) => {
            eprintln!("RLM-0001 conformance spike failed: {message}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn req_001_recovers_invalid_utf8_without_losing_bytes() {
        let source = b"ok \xF0\x28\x8C\x28 end \xE2\x82";
        let lexed = scan(source);

        assert!(validate_partition(source, &lexed.tokens));
        assert!(
            lexed
                .diagnostics
                .iter()
                .filter(|diagnostic| diagnostic.code == "LEX_INVALID_UTF8")
                .count()
                >= 2
        );
    }

    #[test]
    fn req_002_uses_nfc_identity_and_rejects_invisible_identifier_parts() {
        assert_eq!(IDENT_UNICODE_VERSION, (17, 0, 0));
        assert_eq!(NORMALIZATION_UNICODE_VERSION, (17, 0, 0));

        let source = "let café = 1; let cafe\u{301} = 2; let hidden\u{200D}name = 3;".as_bytes();
        let lexed = scan(source);

        assert_eq!(
            normalized_identifier_collisions(source, &lexed.tokens).len(),
            1
        );
        assert!(
            lexed
                .diagnostics
                .iter()
                .any(|diagnostic| { diagnostic.code == "LEX_DISALLOWED_IDENTIFIER_CHARACTER" })
        );
    }

    #[test]
    fn req_003_requires_explicit_semicolons() {
        let source = b"fn sample() { let first = 1 let second = 2; }";
        let diagnostics = parse_recovery(source);

        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "PARSE_MISSING_SEMICOLON")
        );
    }

    #[test]
    fn req_004_recovers_three_independent_errors() {
        let source = br#"fn recover(input: [i32]) -> i32 {
    let first = input[0]
    let middle = input[1:3;
    return first + ;
}"#;
        let diagnostics = parse_recovery(source);

        assert_eq!(
            diagnostics
                .iter()
                .map(|diagnostic| diagnostic.code)
                .collect::<Vec<_>>(),
            vec![
                "PARSE_MISSING_SEMICOLON",
                "PARSE_MISSING_CLOSING_DELIMITER",
                "PARSE_MISSING_EXPRESSION",
            ]
        );
    }

    #[test]
    fn req_005_tokens_and_trivia_round_trip_all_source_bytes() {
        let source = "\u{FEFF}// comment\r\nlet Δ = [1, 2]; /* nested /* block */ */".as_bytes();
        let lexed = scan(source);

        assert!(validate_partition(source, &lexed.tokens));
        let reconstructed = lexed
            .tokens
            .iter()
            .filter(|token| token.kind != TokenKind::EndOfFile)
            .flat_map(|token| &source[token.start..token.end])
            .copied()
            .collect::<Vec<_>>();
        assert_eq!(reconstructed, source);
    }

    #[test]
    fn req_006_normalizes_strict_negative_indices_without_overflow() {
        assert_eq!(normalize_index(0, 4), Some(0));
        assert_eq!(normalize_index(3, 4), Some(3));
        assert_eq!(normalize_index(-1, 4), Some(3));
        assert_eq!(normalize_index(-4, 4), Some(0));
        assert_eq!(normalize_index(4, 4), None);
        assert_eq!(normalize_index(-5, 4), None);
        assert_eq!(normalize_index(i64::MIN, 4), None);
        assert_eq!(normalize_index(0, 0), None);
    }

    #[test]
    fn req_007_normalizes_strict_half_open_slices() {
        assert_eq!(normalize_slice(None, None, 4), Some((0, 4)));
        assert_eq!(normalize_slice(None, Some(2), 4), Some((0, 2)));
        assert_eq!(normalize_slice(Some(-2), None, 4), Some((2, 4)));
        assert_eq!(normalize_slice(Some(1), Some(-1), 4), Some((1, 3)));
        assert_eq!(normalize_slice(Some(2), Some(2), 4), Some((2, 2)));
        assert_eq!(normalize_slice(Some(3), Some(1), 4), None);
        assert_eq!(normalize_slice(None, Some(-5), 4), None);
    }

    #[test]
    fn malformed_numeric_candidates_produce_one_diagnostic_each() {
        let source = b"0b102 1__000 1e+";
        let lexed = scan(source);

        assert_eq!(
            lexed
                .diagnostics
                .iter()
                .filter(|diagnostic| diagnostic.code == "LEX_MALFORMED_NUMBER")
                .count(),
            3
        );
    }

    #[test]
    fn representative_unicode_17_confusable_mappings_match_skeletons() {
        assert_eq!(representative_confusable_skeleton("v\u{0430}r"), "var");
        assert_eq!(representative_confusable_skeleton("sc\u{03BF}pe"), "scope");
        assert_eq!(representative_confusable_skeleton("f\u{043E}r"), "for");
    }

    #[test]
    fn line_terminators_and_pattern_whitespace_are_partitioned_exactly() {
        let source = "\n\r\r\n\u{000B}\u{000C}\u{0085}\u{2028}\u{2029}\t \u{200E}\u{200F}\u{00A0}"
            .as_bytes();
        let lexed = scan(source);

        assert!(validate_partition(source, &lexed.tokens));
        assert_eq!(
            lexed
                .tokens
                .iter()
                .filter(|token| token.kind == TokenKind::Trivia)
                .count(),
            11
        );
        assert_eq!(
            lexed
                .diagnostics
                .iter()
                .filter(|diagnostic| diagnostic.code == "LEX_INVALID_CHARACTER")
                .count(),
            1
        );

        let separated = "left\u{200E}middle\u{200F}right".as_bytes();
        let separated_lexed = scan(separated);
        assert!(separated_lexed.diagnostics.is_empty());
        assert_eq!(
            separated_lexed
                .tokens
                .iter()
                .filter(|token| token.kind == TokenKind::Identifier)
                .map(|token| token.text(separated))
                .collect::<Vec<_>>(),
            vec!["left", "middle", "right"]
        );
    }

    #[test]
    fn malformed_utf8_classes_are_diagnosed_without_losing_bytes() {
        for source in [
            b"\xE2\x82".as_slice(),
            b"\xC0\xAF".as_slice(),
            b"\xED\xA0\x80".as_slice(),
            b"\xF4\x90\x80\x80".as_slice(),
            b"\x80".as_slice(),
        ] {
            let lexed = scan(source);

            assert!(validate_partition(source, &lexed.tokens));
            assert!(
                lexed
                    .diagnostics
                    .iter()
                    .any(|diagnostic| diagnostic.code == "LEX_INVALID_UTF8")
            );
        }

        let valid = "AéΔ😀".as_bytes();
        let valid_lexed = scan(valid);
        assert!(validate_partition(valid, &valid_lexed.tokens));
        assert!(
            valid_lexed
                .diagnostics
                .iter()
                .all(|diagnostic| diagnostic.code != "LEX_INVALID_UTF8")
        );
    }

    #[test]
    fn literals_validate_cardinality_characters_and_escapes() {
        let valid = br#"'a' '\n' '\u{1F600}' "ok\t""#;
        assert!(scan(valid).diagnostics.is_empty());

        let invalid = br#"'' 'ab' '\q' '\u{}' '\u{D800}'"#;
        let invalid_lexed = scan(invalid);
        assert_eq!(
            invalid_lexed
                .diagnostics
                .iter()
                .filter(|diagnostic| diagnostic.code == "LEX_INVALID_CHARACTER_LITERAL")
                .count(),
            2
        );
        assert_eq!(
            invalid_lexed
                .diagnostics
                .iter()
                .filter(|diagnostic| diagnostic.code == "LEX_INVALID_ESCAPE")
                .count(),
            3
        );

        let nul_lexed = scan(b"\"nul\0\"");
        assert!(
            nul_lexed
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "LEX_INVALID_LITERAL_CHARACTER")
        );
    }

    #[test]
    fn comments_cover_line_nested_and_unterminated_forms() {
        let valid = b"// line\r\n/* outer /* nested */ outer */";
        assert!(scan(valid).diagnostics.is_empty());

        let invalid = scan(b"/* unclosed");
        assert_eq!(
            invalid
                .diagnostics
                .iter()
                .filter(|diagnostic| diagnostic.code == "LEX_UNTERMINATED_BLOCK_COMMENT")
                .count(),
            1
        );
    }

    #[test]
    fn empty_bom_and_invalid_source_characters_follow_source_rules() {
        assert!(scan(b"").diagnostics.is_empty());
        assert!(scan("\u{FEFF}".as_bytes()).diagnostics.is_empty());

        let source = "\u{FEFF}let\0name\u{FEFF}".as_bytes();
        let lexed = scan(source);

        assert_eq!(lexed.tokens[0].kind, TokenKind::Trivia);
        assert_eq!(
            lexed
                .diagnostics
                .iter()
                .filter(|diagnostic| diagnostic.code == "LEX_INVALID_CHARACTER")
                .count(),
            2
        );

        let unassigned = scan("\u{0378}".as_bytes());
        assert_eq!(unassigned.tokens[0].kind, TokenKind::InvalidCharacter);
        assert_eq!(unassigned.diagnostics[0].code, "LEX_INVALID_CHARACTER");
    }

    #[test]
    fn every_keyword_and_fixed_token_is_recognized() {
        let keyword_source = KEYWORDS.join(" ");
        let keyword_lexed = scan(keyword_source.as_bytes());
        assert_eq!(
            keyword_lexed
                .tokens
                .iter()
                .filter(|token| token.kind == TokenKind::Keyword)
                .count(),
            KEYWORDS.len()
        );

        let symbol_source = SYMBOLS.join(" ");
        let symbol_lexed = scan(symbol_source.as_bytes());
        let recognized = symbol_lexed
            .tokens
            .iter()
            .filter(|token| token.kind == TokenKind::Symbol)
            .map(|token| token.text(symbol_source.as_bytes()))
            .collect::<Vec<_>>();
        assert_eq!(recognized, SYMBOLS);

        for symbol in SYMBOLS {
            let has_shorter_prefix = SYMBOLS
                .iter()
                .any(|prefix| prefix.len() < symbol.len() && symbol.starts_with(prefix));
            if has_shorter_prefix {
                let lexed = scan(symbol.as_bytes());
                let significant = lexed
                    .tokens
                    .iter()
                    .filter(|token| token.kind != TokenKind::EndOfFile)
                    .collect::<Vec<_>>();
                assert_eq!(significant.len(), 1, "longest match for {symbol}");
                assert_eq!(significant[0].kind, TokenKind::Symbol);
                assert_eq!(significant[0].text(symbol.as_bytes()), *symbol);
            }
        }
    }

    #[test]
    fn single_underscore_is_discard_but_longer_names_are_identifiers() {
        let source = b"_ _value value_";
        let lexed = scan(source);
        let significant_kinds = lexed
            .tokens
            .iter()
            .filter(|token| !matches!(token.kind, TokenKind::Trivia | TokenKind::EndOfFile))
            .map(|token| token.kind)
            .collect::<Vec<_>>();

        assert_eq!(
            significant_kinds,
            vec![
                TokenKind::Discard,
                TokenKind::Identifier,
                TokenKind::Identifier
            ]
        );
    }

    #[test]
    fn diagnostics_are_ordered_deterministically() {
        let lexed = scan(b"\x80\0\x81");
        let keys = lexed
            .diagnostics
            .iter()
            .map(|diagnostic| (diagnostic.start, diagnostic.end, diagnostic.code))
            .collect::<Vec<_>>();
        let mut sorted = keys.clone();
        sorted.sort();

        assert_eq!(keys, sorted);
    }

    #[test]
    fn recovery_caps_diagnostics_per_delimited_construct() {
        let source = b"fn sample() { let a = 0 let b = 1 let c = 2 let d = 3 let e = 4 let f = 5 let g = 6 let h = 7 let i = 8 let j = 9 }";
        let diagnostics = parse_recovery(source);

        assert_eq!(
            diagnostics
                .iter()
                .filter(|diagnostic| diagnostic.code == "PARSE_MISSING_SEMICOLON")
                .count(),
            MAX_SYNTAX_DIAGNOSTICS_PER_CONSTRUCT
        );
        assert_eq!(
            diagnostics
                .iter()
                .filter(|diagnostic| diagnostic.code == "PARSE_DIAGNOSTICS_SUPPRESSED")
                .count(),
            1
        );
    }

    #[test]
    fn malformed_utf8_inside_comments_and_literals_remains_separate() {
        for source in [
            b"// before \xF0\x28 after".as_slice(),
            b"/* before \xF0\x28 after */".as_slice(),
            b"\"before \xF0\x28 after\"".as_slice(),
        ] {
            let lexed = scan(source);

            assert!(validate_partition(source, &lexed.tokens));
            assert!(
                lexed
                    .diagnostics
                    .iter()
                    .any(|diagnostic| diagnostic.code == "LEX_INVALID_UTF8")
            );
            assert!(
                lexed
                    .tokens
                    .iter()
                    .any(|token| token.kind == TokenKind::InvalidUtf8)
            );
        }
    }

    #[test]
    fn recovery_handles_unclosed_delimiters_at_eof() {
        let diagnostics = parse_recovery(b"fn sample() { let value = (items[0");

        assert_eq!(
            diagnostics
                .iter()
                .filter(|diagnostic| diagnostic.code == "PARSE_MISSING_CLOSING_DELIMITER")
                .count(),
            2
        );
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "PARSE_MISSING_SEMICOLON")
        );
    }

    #[test]
    fn final_block_expression_does_not_require_a_semicolon() {
        let diagnostics = parse_recovery(b"fn value() { 1 + 2 }");

        assert!(diagnostics.is_empty());
    }
}
