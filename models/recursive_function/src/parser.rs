use std::collections::HashMap;

use utils::lexer::{lex_tree, DelimKind, Token, Tree};
use utils::TextCodec;

use crate::machine::RecursiveFunctions;

pub fn parse(text: &str) -> Result<RecursiveFunctions, String> {
    let trees = clean_trees(lex_tree(text).map_err(|err| err.to_string())?);
    let mut parser = Parser::new(&trees);
    let mut map = HashMap::new();

    while parser.is_keyword("let") {
        let (name, func) = parse_let_statement(&mut parser, &map)?;
        map.insert(name, func);
    }

    let func = parse_func(&mut parser, &map)?;
    parser.expect_eof()?;
    Ok(func)
}

impl TextCodec for RecursiveFunctions {
    fn parse(text: &str) -> Result<Self, String> {
        parse(text)
    }

    fn write_fmt(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

fn parse_let_statement(
    parser: &mut Parser<'_>,
    map: &HashMap<String, RecursiveFunctions>,
) -> Result<(String, RecursiveFunctions), String> {
    parser.expect_keyword("let")?;
    let name = parser.parse_name()?;
    parser.expect_symbol('=')?;
    let func = parse_func(parser, map)?;
    parser.expect_symbol('.')?;
    if map.contains_key(&name) {
        return Err(format!("Function {name} already exists"));
    }
    Ok((name, func))
}

fn parse_func(
    parser: &mut Parser<'_>,
    map: &HashMap<String, RecursiveFunctions>,
) -> Result<RecursiveFunctions, String> {
    let name = parser.parse_name()?;
    match name.as_str() {
        "ZERO" => Ok(RecursiveFunctions::zero()),
        "SUCC" => Ok(RecursiveFunctions::succ()),
        "PROJ" => parse_proj(parser),
        "COMP" => parse_comp(parser, map),
        "PRIM" => parse_prim(parser, map),
        "MUOP" => parse_muop(parser, map),
        _ => map
            .get(&name)
            .cloned()
            .ok_or_else(|| format!("Function {name} not found")),
    }
}

fn parse_proj(parser: &mut Parser<'_>) -> Result<RecursiveFunctions, String> {
    let mut inner = parser.expect_delim(DelimKind::Bracket)?;
    let length = inner.parse_number()?;
    inner.expect_symbol(',')?;
    let number = inner.parse_number()?;
    inner.expect_eof()?;
    RecursiveFunctions::projection(length, number)
}

fn parse_comp(
    parser: &mut Parser<'_>,
    map: &HashMap<String, RecursiveFunctions>,
) -> Result<RecursiveFunctions, String> {
    let mut inner = parser.expect_delim(DelimKind::Bracket)?;
    let outer = parse_func(&mut inner, map)?;
    inner.expect_symbol(':')?;
    let inner_funcs = parse_func_list(&mut inner, map)?;
    inner.skip_commas();
    inner.expect_eof()?;
    RecursiveFunctions::composition(outer, inner_funcs)
}

fn parse_prim(
    parser: &mut Parser<'_>,
    map: &HashMap<String, RecursiveFunctions>,
) -> Result<RecursiveFunctions, String> {
    let mut inner = parser.expect_delim(DelimKind::Bracket)?;
    inner.expect_keyword("z")?;
    inner.expect_symbol(':')?;
    let zero = parse_func(&mut inner, map)?;
    inner.expect_keyword("s")?;
    inner.expect_symbol(':')?;
    let succ = parse_func(&mut inner, map)?;
    inner.expect_eof()?;
    RecursiveFunctions::primitive_recursion(zero, succ)
}

fn parse_muop(
    parser: &mut Parser<'_>,
    map: &HashMap<String, RecursiveFunctions>,
) -> Result<RecursiveFunctions, String> {
    let mut inner = parser.expect_delim(DelimKind::Bracket)?;
    let func = parse_func(&mut inner, map)?;
    inner.expect_eof()?;
    RecursiveFunctions::muoperator(func)
}

fn parse_func_list(
    parser: &mut Parser<'_>,
    map: &HashMap<String, RecursiveFunctions>,
) -> Result<Vec<RecursiveFunctions>, String> {
    let mut items = Vec::new();
    if parser.peek().is_none() {
        return Ok(items);
    }

    loop {
        if let Some(Tree::Delim {
            delim: DelimKind::Paren,
            child,
        }) = parser.peek()
        {
            parser.pos += 1;
            let mut inner = Parser::new(child);
            items.extend(parse_func_list(&mut inner, map)?);
            inner.skip_commas();
            inner.expect_eof()?;
        } else {
            items.push(parse_func(parser, map)?);
        }

        if !parser.consume_symbol(',') {
            break;
        }
        parser.skip_commas();
        if parser.peek().is_none() {
            break;
        }
    }

    Ok(items)
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

    fn expect_eof(&self) -> Result<(), String> {
        if self.peek().is_none() {
            Ok(())
        } else {
            Err("unexpected trailing input".to_string())
        }
    }

    fn expect_symbol(&mut self, ch: char) -> Result<(), String> {
        match self.next() {
            Some(Tree::Token(Token::Symbol(found))) if *found == ch => Ok(()),
            _ => Err(format!("expected symbol '{ch}'")),
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

    fn expect_keyword(&mut self, keyword: &str) -> Result<(), String> {
        let name = self.parse_name()?;
        if name == keyword {
            Ok(())
        } else {
            Err(format!("expected keyword '{keyword}'"))
        }
    }

    fn is_keyword(&self, keyword: &str) -> bool {
        let mut clone = Self {
            items: self.items,
            pos: self.pos,
        };
        clone.parse_name().is_ok_and(|name| name == keyword)
    }

    fn expect_delim(&mut self, delim: DelimKind) -> Result<Parser<'a>, String> {
        match self.next() {
            Some(Tree::Delim { delim: found, child }) if *found == delim => Ok(Parser::new(child)),
            _ => Err(format!("expected delimiter {:?}", delim)),
        }
    }

    fn parse_number(&mut self) -> Result<usize, String> {
        match self.next() {
            Some(Tree::Token(Token::Number(num))) => num
                .parse::<usize>()
                .map_err(|_| format!("invalid number '{num}'")),
            _ => Err("expected number".to_string()),
        }
    }

    fn parse_name(&mut self) -> Result<String, String> {
        let mut out = match self.next() {
            Some(Tree::Token(Token::Ident(text))) => text.clone(),
            Some(Tree::Token(Token::Symbol(ch @ ('-' | '_')))) => ch.to_string(),
            _ => return Err("expected name".to_string()),
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
                _ => return Err("expected name".to_string()),
            }
        }

        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_test_simple() {
        let code = "ZERO";
        let func = parse(code).unwrap();
        assert_eq!(func, RecursiveFunctions::zero());

        let code = "SUCC";
        let func = parse(code).unwrap();
        assert_eq!(func, RecursiveFunctions::succ());

        let code = "PROJ[3, 0]";
        let func = parse(code).unwrap();
        assert_eq!(func, RecursiveFunctions::projection(3, 0).unwrap());
    }

    #[test]
    fn parse_test_rec() {
        let code = "COMP[SUCC: (ZERO)]";
        let func = parse(code).unwrap();
        assert_eq!(
            func,
            RecursiveFunctions::composition(
                RecursiveFunctions::succ(),
                vec![RecursiveFunctions::zero()],
            )
            .unwrap()
        );

        let code = "PRIM[z: ZERO s: PROJ[2, 0] ]";
        let func = parse(code).unwrap();
        assert_eq!(
            func,
            RecursiveFunctions::primitive_recursion(
                RecursiveFunctions::zero(),
                RecursiveFunctions::projection(2, 0).unwrap()
            )
            .unwrap()
        );

        let code = "MUOP[SUCC]";
        let func = parse(code).unwrap();
        assert_eq!(
            func,
            RecursiveFunctions::muoperator(RecursiveFunctions::succ()).unwrap()
        );
    }

    #[test]
    fn parse_test_rec2() {
        let code = "MUOP[MUOP[PROJ[2, 0]]]";
        let func = parse(code).unwrap();
        assert_eq!(
            func,
            RecursiveFunctions::muoperator(
                RecursiveFunctions::muoperator(RecursiveFunctions::projection(2, 0).unwrap())
                    .unwrap()
            )
            .unwrap()
        );

        let code = "COMP[PROJ[2, 0]: (MUOP[SUCC], MUOP[SUCC])]";
        let func = parse(code).unwrap();
        assert_eq!(
            func,
            RecursiveFunctions::composition(
                RecursiveFunctions::projection(2, 0).unwrap(),
                vec![
                    RecursiveFunctions::muoperator(RecursiveFunctions::succ()).unwrap(),
                    RecursiveFunctions::muoperator(RecursiveFunctions::succ()).unwrap()
                ]
            )
            .unwrap()
        );
    }

    #[test]
    fn parse_test_with_name() {
        let code = "let f = ZERO.\nf";
        let func = parse(code).unwrap();
        assert_eq!(func, RecursiveFunctions::zero());

        let code = "let f = ZERO.\nlet g = SUCC.\nf";
        let func = parse(code).unwrap();
        assert_eq!(func, RecursiveFunctions::zero());

        let code = "let f = PROJ[2,1].\nlet g = SUCC.\nCOMP[f: (g, g)]";
        let func = parse(code).unwrap();
        assert_eq!(
            func,
            RecursiveFunctions::composition(
                RecursiveFunctions::projection(2, 1).unwrap(),
                vec![RecursiveFunctions::succ(), RecursiveFunctions::succ()]
            )
            .unwrap()
        );
    }
}
