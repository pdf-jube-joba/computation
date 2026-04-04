use anyhow::{bail, Result};
use utils::lexer::{lex_tree, DelimKind, Token, Tree};

use crate::machine::Value;

pub fn parse_value(text: &str) -> Result<Value> {
    let trees = clean_trees(lex_tree(text).map_err(|err| anyhow::anyhow!(err.to_string()))?);
    let mut parser = Parser::new(&trees);
    let value = parse_value_inner(&mut parser)?;
    parser.expect_eof()?;
    Ok(value)
}

fn parse_value_inner(parser: &mut Parser<'_>) -> Result<Value> {
    if parser.is_prefixed_ident('#', "true") {
        parser.expect_symbol('#')?;
        parser.expect_ident("true")?;
        return Ok(Value::Bit(utils::bool::Bool::T));
    }
    if parser.is_prefixed_ident('#', "false") {
        parser.expect_symbol('#')?;
        parser.expect_ident("false")?;
        return Ok(Value::Bit(utils::bool::Bool::F));
    }

    match parser.peek() {
        Some(Tree::Delim {
            delim: DelimKind::Bracket,
            ..
        }) => parse_array(parser),
        Some(Tree::Delim {
            delim: DelimKind::Brace,
            ..
        }) => parse_struct(parser),
        Some(Tree::Token(Token::Symbol('<'))) => parse_enum(parser),
        _ => bail!("expected value"),
    }
}

fn parse_array(parser: &mut Parser<'_>) -> Result<Value> {
    let mut inner = parser.expect_delim(DelimKind::Bracket)?;
    let mut values = Vec::new();
    inner.skip_commas();
    while inner.peek().is_some() {
        values.push(parse_value_inner(&mut inner)?);
        inner.skip_commas();
    }
    Ok(Value::Array(values))
}

fn parse_struct(parser: &mut Parser<'_>) -> Result<Value> {
    let mut inner = parser.expect_delim(DelimKind::Brace)?;
    let mut fields = Vec::new();
    inner.skip_commas();
    while inner.peek().is_some() {
        let name = inner.parse_name()?;
        inner.expect_symbol(':')?;
        let value = parse_value_inner(&mut inner)?;
        fields.push((name, value));
        inner.skip_commas();
    }
    Value::new_strct(fields).ok_or_else(|| anyhow::anyhow!("field 名がかぶってそう"))
}

fn parse_enum(parser: &mut Parser<'_>) -> Result<Value> {
    parser.expect_symbol('<')?;
    let name = parser.parse_name()?;
    parser.expect_symbol('>')?;
    Ok(Value::new_enume(name))
}

fn clean_trees(items: Vec<Tree>) -> Vec<Tree> {
    items
        .into_iter()
        .filter_map(|tree| match tree {
            Tree::Token(Token::Whitespace(_) | Token::Comment(_)) => None,
            Tree::Token(token) => Some(Tree::Token(token)),
            Tree::Delim { delim, child } => Some(Tree::Delim {
                delim,
                child: clean_trees(child),
            }),
        })
        .collect()
}

struct Parser<'a> {
    items: &'a [Tree],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(items: &'a [Tree]) -> Self {
        Self { items, pos: 0 }
    }

    fn peek(&self) -> Option<&'a Tree> {
        self.items.get(self.pos)
    }

    fn next(&mut self) -> Option<&'a Tree> {
        let item = self.items.get(self.pos);
        if item.is_some() {
            self.pos += 1;
        }
        item
    }

    fn expect_eof(&self) -> Result<()> {
        if self.peek().is_none() {
            Ok(())
        } else {
            bail!("unexpected trailing input")
        }
    }

    fn expect_symbol(&mut self, ch: char) -> Result<()> {
        match self.next() {
            Some(Tree::Token(Token::Symbol(found))) if *found == ch => Ok(()),
            _ => bail!("expected symbol '{ch}'"),
        }
    }

    fn consume_symbol(&mut self, ch: char) -> bool {
        match self.peek() {
            Some(Tree::Token(Token::Symbol(found))) if *found == ch => {
                self.pos += 1;
                true
            }
            _ => false,
        }
    }

    fn skip_commas(&mut self) {
        while self.consume_symbol(',') {}
    }

    fn expect_ident(&mut self, expected: &str) -> Result<()> {
        match self.next() {
            Some(Tree::Token(Token::Ident(found))) if found == expected => Ok(()),
            _ => bail!("expected identifier '{expected}'"),
        }
    }

    fn is_prefixed_ident(&self, ch: char, ident: &str) -> bool {
        matches!(
            (self.items.get(self.pos), self.items.get(self.pos + 1)),
            (
                Some(Tree::Token(Token::Symbol(found))),
                Some(Tree::Token(Token::Ident(found_ident)))
            ) if *found == ch && found_ident == ident
        )
    }

    fn expect_delim(&mut self, delim: DelimKind) -> Result<Parser<'a>> {
        match self.next() {
            Some(Tree::Delim { delim: found, child }) if *found == delim => Ok(Parser::new(child)),
            _ => bail!("expected delimiter {:?}", delim),
        }
    }

    fn parse_name(&mut self) -> Result<String> {
        let mut out = match self.next() {
            Some(Tree::Token(Token::Ident(text))) => text.clone(),
            Some(Tree::Token(Token::Symbol(ch @ ('-' | '_')))) => ch.to_string(),
            _ => bail!("expected name"),
        };

        loop {
            let Some(Tree::Token(Token::Symbol(ch @ ('-' | '_')))) = self.peek() else {
                break;
            };
            out.push(*ch);
            self.pos += 1;
            match self.next() {
                Some(Tree::Token(Token::Ident(text))) | Some(Tree::Token(Token::Number(text))) => {
                    out.push_str(text);
                }
                _ => bail!("expected name"),
            }
        }

        Ok(out)
    }
}
