use std::fmt;

use logos::{Lexer, Logos};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Ident(String),
    Number(String),
    StringLiteral(String),
    Whitespace(String),
    Comment(String),
    Symbol(char),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DelimKind {
    Paren,
    Bracket,
    Brace,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Tree {
    Token(Token),
    Delim { delim: DelimKind, child: Vec<Tree> },
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SpannedToken {
    token: Token,
    byte: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexError {
    pub message: String,
    pub byte: usize,
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} at byte {}", self.message, self.byte)
    }
}

#[derive(Logos, Debug, Clone, Copy, PartialEq, Eq)]
enum RawToken {
    #[regex(r"[ \t\r\n]+")]
    Whitespace,
    #[regex(r"[A-Za-z_][A-Za-z0-9_]*")]
    Ident,
    #[regex(r"[0-9]+")]
    Number,
    #[token("\"", lex_string_literal)]
    StringLiteral,
    #[token("//", lex_line_comment)]
    LineComment,
    #[token("/*", lex_block_comment)]
    BlockComment,
    #[regex(r"[!#$%&'()*+,\-./:;<=>?@\[\\\]^`{|}~]")]
    Symbol,
}

pub fn lex(text: &str) -> Result<Vec<Token>, LexError> {
    Ok(lex_spanned(text)?
        .into_iter()
        .map(|token| token.token)
        .collect())
}

pub fn lex_tree(text: &str) -> Result<Vec<Tree>, LexError> {
    nest_spanned(lex_spanned(text)?)
}

pub fn nest(tokens: Vec<Token>) -> Result<Vec<Tree>, LexError> {
    let spanned = tokens
        .into_iter()
        .map(|token| SpannedToken { token, byte: 0 })
        .collect();
    nest_spanned(spanned)
}

fn lex_spanned(text: &str) -> Result<Vec<SpannedToken>, LexError> {
    let mut lexer = RawToken::lexer(text);
    let mut tokens = Vec::new();

    while let Some(result) = lexer.next() {
        let span = lexer.span();
        let raw = result.map_err(|_| LexError {
            message: invalid_character_message(text, span.start),
            byte: span.start,
        })?;
        let slice = lexer.slice();

        let token = match raw {
            RawToken::Whitespace => Token::Whitespace(slice.to_string()),
            RawToken::Ident => Token::Ident(slice.to_string()),
            RawToken::Number => Token::Number(slice.to_string()),
            RawToken::StringLiteral => {
                ensure_ascii(slice, span.start, "non-ASCII string literal")?;
                Token::StringLiteral(slice.to_string())
            }
            RawToken::LineComment | RawToken::BlockComment => Token::Comment(slice.to_string()),
            RawToken::Symbol => Token::Symbol(slice.chars().next().unwrap()),
        };

        tokens.push(SpannedToken {
            token,
            byte: span.start,
        });
    }

    Ok(tokens)
}

fn nest_spanned(tokens: Vec<SpannedToken>) -> Result<Vec<Tree>, LexError> {
    let mut root = Vec::new();
    let mut stack: Vec<(DelimKind, Vec<Tree>, usize)> = Vec::new();

    for token in tokens {
        match token.token {
            Token::Symbol(ch) => {
                if let Some(delim) = opening_delim(ch) {
                    stack.push((delim, Vec::new(), token.byte));
                    continue;
                }

                if let Some(found) = closing_delim(ch) {
                    let Some((expected, child, _open_byte)) = stack.pop() else {
                        return Err(LexError {
                            message: format!("unexpected closing delimiter '{}'", ch),
                            byte: token.byte,
                        });
                    };

                    if expected != found {
                        return Err(LexError {
                            message: format!(
                                "mismatched delimiter: expected '{}' but found '{}'",
                                opening_char(expected),
                                ch
                            ),
                            byte: token.byte,
                        });
                    }

                    push_tree(
                        &mut root,
                        &mut stack,
                        Tree::Delim {
                            delim: found,
                            child,
                        },
                    );
                    continue;
                }

                push_tree(&mut root, &mut stack, Tree::Token(Token::Symbol(ch)));
            }
            other => push_tree(&mut root, &mut stack, Tree::Token(other)),
        }
    }

    if let Some((delim, _, byte)) = stack.pop() {
        return Err(LexError {
            message: format!("unclosed delimiter '{}'", opening_char(delim)),
            byte,
        });
    }

    Ok(root)
}

fn push_tree(root: &mut Vec<Tree>, stack: &mut [(DelimKind, Vec<Tree>, usize)], tree: Tree) {
    if let Some((_, child, _)) = stack.last_mut() {
        child.push(tree);
    } else {
        root.push(tree);
    }
}

fn opening_delim(ch: char) -> Option<DelimKind> {
    match ch {
        '(' => Some(DelimKind::Paren),
        '[' => Some(DelimKind::Bracket),
        '{' => Some(DelimKind::Brace),
        _ => None,
    }
}

fn closing_delim(ch: char) -> Option<DelimKind> {
    match ch {
        ')' => Some(DelimKind::Paren),
        ']' => Some(DelimKind::Bracket),
        '}' => Some(DelimKind::Brace),
        _ => None,
    }
}

fn opening_char(delim: DelimKind) -> char {
    match delim {
        DelimKind::Paren => '(',
        DelimKind::Bracket => '[',
        DelimKind::Brace => '{',
    }
}

fn lex_string_literal(lexer: &mut Lexer<'_, RawToken>) -> Result<(), ()> {
    let bytes = lexer.remainder().as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        match bytes[i] {
            b'"' => {
                lexer.bump(i + 1);
                return Ok(());
            }
            b'\\' => {
                i += 1;
                if i >= bytes.len() {
                    return Err(());
                }
                i += 1;
            }
            _ => {
                i += 1;
            }
        }
    }

    Err(())
}

fn lex_line_comment(lexer: &mut Lexer<'_, RawToken>) {
    let remainder = lexer.remainder();
    let len = remainder.find('\n').unwrap_or(remainder.len());
    lexer.bump(len);
}

fn lex_block_comment(lexer: &mut Lexer<'_, RawToken>) -> Result<(), ()> {
    let remainder = lexer.remainder();
    let len = remainder.find("*/").ok_or(())?;
    lexer.bump(len + 2);
    Ok(())
}

fn ensure_ascii(slice: &str, byte: usize, kind: &str) -> Result<(), LexError> {
    if slice.is_ascii() {
        Ok(())
    } else {
        Err(LexError {
            message: kind.to_string(),
            byte,
        })
    }
}

fn invalid_character_message(text: &str, byte: usize) -> String {
    let ch = text[byte..].chars().next().unwrap();
    if ch.is_ascii() {
        format!("unexpected character '{}'", ch)
    } else {
        "non-ASCII character outside comment".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::{lex, lex_tree, DelimKind, LexError, Token, Tree};

    #[test]
    fn lexes_common_tokens() {
        let tokens = lex(r#"foo := "bar" // コメント"#).unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Ident("foo".to_string()),
                Token::Whitespace(" ".to_string()),
                Token::Symbol(':'),
                Token::Symbol('='),
                Token::Whitespace(" ".to_string()),
                Token::StringLiteral(r#""bar""#.to_string()),
                Token::Whitespace(" ".to_string()),
                Token::Comment("// コメント".to_string()),
            ]
        );
    }

    #[test]
    fn keeps_whitespace_runs() {
        let tokens = lex("a \n\t b").unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Ident("a".to_string()),
                Token::Whitespace(" \n\t ".to_string()),
                Token::Ident("b".to_string()),
            ]
        );
    }

    #[test]
    fn comments_can_contain_utf8() {
        let tokens = lex("/* 日本語コメント */x").unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Comment("/* 日本語コメント */".to_string()),
                Token::Ident("x".to_string()),
            ]
        );
    }

    #[test]
    fn line_comments_can_contain_utf8() {
        let tokens = lex("// 日本語\nx").unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Comment("// 日本語".to_string()),
                Token::Whitespace("\n".to_string()),
                Token::Ident("x".to_string()),
            ]
        );
    }

    #[test]
    fn rejects_utf8_outside_comments() {
        let err = lex("変数").unwrap_err();
        assert_eq!(
            err,
            LexError {
                message: "non-ASCII character outside comment".to_string(),
                byte: 0,
            }
        );
    }

    #[test]
    fn rejects_utf8_in_string_literals() {
        let err = lex(r#""日本語""#).unwrap_err();
        assert_eq!(
            err,
            LexError {
                message: "non-ASCII string literal".to_string(),
                byte: 0,
            }
        );
    }

    #[test]
    fn string_literal_shields_delimiters() {
        let tree = lex_tree(r#"f("(", [x])"#).unwrap();
        assert_eq!(
            tree,
            vec![
                Tree::Token(Token::Ident("f".to_string())),
                Tree::Delim {
                    delim: DelimKind::Paren,
                    child: vec![
                        Tree::Token(Token::StringLiteral(r#""(""#.to_string())),
                        Tree::Token(Token::Symbol(',')),
                        Tree::Token(Token::Whitespace(" ".to_string())),
                        Tree::Delim {
                            delim: DelimKind::Bracket,
                            child: vec![Tree::Token(Token::Ident("x".to_string()))],
                        },
                    ],
                },
            ]
        );
    }

    #[test]
    fn builds_nested_delimiter_tree() {
        let tree = lex_tree("a(b[c]{d})").unwrap();
        assert_eq!(
            tree,
            vec![
                Tree::Token(Token::Ident("a".to_string())),
                Tree::Delim {
                    delim: DelimKind::Paren,
                    child: vec![
                        Tree::Token(Token::Ident("b".to_string())),
                        Tree::Delim {
                            delim: DelimKind::Bracket,
                            child: vec![Tree::Token(Token::Ident("c".to_string()))],
                        },
                        Tree::Delim {
                            delim: DelimKind::Brace,
                            child: vec![Tree::Token(Token::Ident("d".to_string()))],
                        },
                    ],
                },
            ]
        );
    }

    #[test]
    fn reports_unclosed_delimiter() {
        let err = lex_tree("(x").unwrap_err();
        assert_eq!(
            err,
            LexError {
                message: "unclosed delimiter '('".to_string(),
                byte: 0,
            }
        );
    }

    #[test]
    fn reports_mismatched_delimiter() {
        let err = lex_tree("(]").unwrap_err();
        assert_eq!(
            err,
            LexError {
                message: "mismatched delimiter: expected '(' but found ']'".to_string(),
                byte: 1,
            }
        );
    }
}
