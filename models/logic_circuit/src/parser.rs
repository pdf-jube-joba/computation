use anyhow::{bail, Result};
use either::Either;
use utils::identifier::Identifier;
use utils::lexer::{lex_tree, DelimKind, Token, Tree};

use crate::machine::*;
use crate::manipulation::{init_maps, List};

type GraphPin = (Identifier, Either<NamedPin, Identifier>);

#[derive(Debug)]
struct FingraphParse {
    name: Identifier,
    inpin: Vec<Identifier>,
    otpin: Vec<(Identifier, NamedPin)>,
    lcs: Vec<(Identifier, Identifier, Vec<GraphPin>)>,
}

#[derive(Debug)]
struct IterParse {
    name: Identifier,
    initlc: Identifier,
    next: Vec<(Pin, Pin)>,
    prev: Vec<(Pin, Pin)>,
}

pub fn parse(code: &str, maps: &mut List) -> Result<()> {
    let trees = clean_trees(lex_tree(code).map_err(|err| anyhow::anyhow!(err.to_string()))?);
    let mut parser = Parser::new(&trees);

    while parser.peek().is_some() {
        if parser.is_keyword("graph") {
            let FingraphParse {
                name,
                inpin: inpins,
                otpin: otpin_maps,
                lcs,
            } = parse_fingraph(&mut parser)?;
            eprintln!("{name}");
            let mut new_lcs = vec![];
            let mut edges = vec![];
            let mut inpin_maps: Vec<(Identifier, NamedPin)> = vec![];

            for (lcname, usename, inout) in lcs {
                let Some(c) = maps.get(&usename) else {
                    bail!("not found name {usename}");
                };
                new_lcs.push((lcname.clone(), c.clone()));
                for (inpin, out) in inout {
                    match out {
                        Either::Left((name, otpin)) => {
                            edges.push(((name, otpin), (lcname.clone(), inpin)));
                        }
                        Either::Right(i) => {
                            if !inpins.contains(&i) {
                                bail!("not found inpin {i}");
                            }
                            inpin_maps.push((i, (lcname.clone(), inpin)));
                        }
                    }
                }
            }

            let graphlc =
                LogicCircuit::new_mix(name.clone(), new_lcs, edges, inpin_maps, otpin_maps)?;
            maps.insert((name, graphlc));
        } else if parser.is_keyword("iter") {
            let IterParse {
                name,
                initlc,
                next,
                prev,
            } = parse_iter(&mut parser)?;
            eprintln!("{name}");
            let Some(initlc) = maps.get(&initlc) else {
                bail!("not found name {initlc}");
            };
            let iterlc = LogicCircuit::new_iter(name.clone(), initlc.clone(), next, prev)?;
            maps.insert((name, iterlc));
        } else {
            bail!("expected graph or iter");
        }
    }

    Ok(())
}

pub fn parse_main_with_maps(code: &str, mut maps: List) -> Result<LogicCircuit> {
    parse(code, &mut maps)?;
    match maps.get(&Identifier::new("main").unwrap()) {
        Some(lc) => Ok(lc.clone()),
        None => bail!("not found main"),
    }
}

pub fn parse_main(code: &str) -> Result<LogicCircuit> {
    let mut maps: List = init_maps();
    parse(code, &mut maps)?;
    match maps.get(&Identifier::new("main").unwrap()) {
        Some(lc) => Ok(lc.clone()),
        None => bail!("not found main"),
    }
}

fn parse_fingraph(parser: &mut Parser<'_>) -> Result<FingraphParse> {
    parser.expect_keyword("graph")?;
    parser.expect_symbol(':')?;
    let name = parse_identifier(parser)?;
    let mut body = parser.expect_delim(DelimKind::Brace)?;

    body.expect_keyword("in")?;
    let inpin = parse_name_block(&mut body)?;

    body.expect_keyword("out")?;
    let otpin = parse_output_block(&mut body)?;

    let mut lcs = Vec::new();
    while body.peek().is_some() {
        lcs.push(parse_lc_graph(&mut body)?);
    }

    Ok(FingraphParse {
        name,
        inpin,
        otpin,
        lcs,
    })
}

fn parse_iter(parser: &mut Parser<'_>) -> Result<IterParse> {
    parser.expect_keyword("iter")?;
    parser.expect_symbol(':')?;
    let name = parse_identifier(parser)?;
    let mut body = parser.expect_delim(DelimKind::Brace)?;

    let initlc = parse_identifier(&mut body)?;
    body.expect_symbol(',')?;

    body.expect_keyword("next")?;
    let next = parse_conn_iter_block(&mut body)?;

    body.expect_keyword("prev")?;
    let prev = parse_conn_iter_block(&mut body)?;

    body.expect_eof()?;

    Ok(IterParse {
        name,
        initlc,
        next,
        prev,
    })
}

fn parse_name_block(parser: &mut Parser<'_>) -> Result<Vec<Identifier>> {
    let mut inner = parser.expect_delim(DelimKind::Brace)?;
    let mut names = Vec::new();
    inner.skip_commas();
    while inner.peek().is_some() {
        names.push(parse_identifier(&mut inner)?);
        inner.skip_commas();
    }
    Ok(names)
}

fn parse_output_block(parser: &mut Parser<'_>) -> Result<Vec<(Identifier, NamedPin)>> {
    let mut inner = parser.expect_delim(DelimKind::Brace)?;
    let mut items = Vec::new();
    inner.skip_commas();
    while inner.peek().is_some() {
        let external = parse_identifier(&mut inner)?;
        inner.expect_symbol('=')?;
        let name = parse_identifier(&mut inner)?;
        inner.expect_symbol('.')?;
        let pin = parse_identifier(&mut inner)?;
        items.push((external, (name, pin)));
        inner.skip_commas();
    }
    Ok(items)
}

fn parse_lc_graph(parser: &mut Parser<'_>) -> Result<(Identifier, Identifier, Vec<GraphPin>)> {
    let name = parse_identifier(parser)?;
    parser.expect_symbol(',')?;
    let usename = parse_identifier(parser)?;
    let mut block = parser.expect_delim(DelimKind::Brace)?;
    let mut conns = Vec::new();
    block.skip_commas();
    while block.peek().is_some() {
        conns.push(parse_conn_graph(&mut block)?);
        block.skip_commas();
    }
    Ok((name, usename, conns))
}

fn parse_conn_graph(parser: &mut Parser<'_>) -> Result<GraphPin> {
    let inpin = parse_identifier(parser)?;
    parser.expect_symbol('=')?;
    let first = parse_identifier(parser)?;
    if parser.consume_symbol('.') {
        let second = parse_identifier(parser)?;
        Ok((inpin, Either::Left((first, second))))
    } else {
        Ok((inpin, Either::Right(first)))
    }
}

fn parse_conn_iter_block(parser: &mut Parser<'_>) -> Result<Vec<(Pin, Pin)>> {
    let mut inner = parser.expect_delim(DelimKind::Brace)?;
    let mut items = Vec::new();
    inner.skip_commas();
    while inner.peek().is_some() {
        let left = parse_identifier(&mut inner)?;
        inner.expect_symbol('=')?;
        let right = parse_identifier(&mut inner)?;
        items.push((left, right));
        inner.skip_commas();
    }
    Ok(items)
}

fn parse_identifier(parser: &mut Parser<'_>) -> Result<Identifier> {
    Identifier::new(parser.parse_name()?)
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

    fn expect_keyword(&mut self, keyword: &str) -> Result<()> {
        let name = self.parse_name()?;
        if name == keyword {
            Ok(())
        } else {
            bail!("expected keyword '{keyword}'")
        }
    }

    fn is_keyword(&self, keyword: &str) -> bool {
        let mut clone = Self {
            items: self.items,
            pos: self.pos,
        };
        clone.parse_name().is_ok_and(|name| name == keyword)
    }

    fn expect_delim(&mut self, delim: DelimKind) -> Result<Parser<'a>> {
        match self.next() {
            Some(Tree::Delim { delim: found, child }) if *found == delim => Ok(Parser::new(child)),
            _ => bail!("expected delimiter {:?}", delim),
        }
    }

    fn parse_name(&mut self) -> Result<String> {
        let mut out = match self.next() {
            Some(Tree::Token(Token::Ident(text))) | Some(Tree::Token(Token::Number(text))) => {
                text.clone()
            }
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
