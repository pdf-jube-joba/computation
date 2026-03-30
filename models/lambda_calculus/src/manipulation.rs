pub mod utility {
    use crate::machine::LambdaTerm;
    use utils::identifier::Var;

    pub fn apps(first: LambdaTerm, remains: Vec<LambdaTerm>) -> LambdaTerm {
        let mut term = first;
        for remain in remains {
            term = LambdaTerm::App(term.into(), remain.into());
        }
        term
    }

    pub fn app_with_nonepmty(all: Vec<LambdaTerm>) -> LambdaTerm {
        assert!(!all.is_empty());
        let term = all[0].clone();
        let remains = all[1..].to_vec();
        apps(term, remains)
    }

    pub fn lambdas(pres: Vec<Var>, last: LambdaTerm) -> LambdaTerm {
        let mut term = last;
        for pre in pres.into_iter().rev() {
            term = LambdaTerm::Abs(pre, term.into());
        }
        term
    }

    #[macro_export]
    macro_rules! lam {
        ($v:expr, $t:expr) => {
            $crate::machine::LambdaTerm::Abs($v, Box::new($t))
        };
    }

    // e1 e2 ... en = (((e1 e2) e3) ... en)
    #[macro_export]
    macro_rules! app {
        ($( $x:expr ),*) => {
            {
                let alls = vec![$($x),*];
                $crate::manipulation::utility::app_with_nonepmty(alls)
            }
        };
    }

    pub use {app, lam};

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::machine::LambdaTerm;

        fn v(var: &Var) -> LambdaTerm {
            LambdaTerm::Var(var.clone())
        }
        fn abs(var: &Var, body: LambdaTerm) -> LambdaTerm {
            LambdaTerm::Abs(var.clone(), body.into())
        }
        fn app(lhs: LambdaTerm, rhs: LambdaTerm) -> LambdaTerm {
            LambdaTerm::App(lhs.into(), rhs.into())
        }

        #[test]
        fn test_var() {
            let term: Var = "x".into();
            assert_eq!(term.as_str(), "x");
        }
        #[test]
        fn test_lam() {
            let x = Var::from("x");
            let y = Var::from("y");
            let term = lam!(x.clone(), v(&y));
            assert_eq!(term, abs(&x, v(&y)));
        }
        #[test]
        fn test_app() {
            let x = Var::from("x");
            let y = Var::from("y");
            let z = Var::from("z");

            // "x"
            let term = app!(v(&x));
            assert_eq!(term, v(&x));

            // "x y"
            let term = app!(v(&x), v(&y));
            assert_eq!(term, app(v(&x), v(&y)));

            // "(x y) z"
            let term = app!(v(&x), v(&y), v(&z));
            assert_eq!(term, app(app(v(&x), v(&y)), v(&z)));
        }
    }
}

pub mod parse {
    use utils::identifier::Var;
    use utils::{lex_tree, DelimKind, Token as LexToken, Tree};

    use crate::{
        machine::LambdaTerm,
        manipulation::utility::{self, app_with_nonepmty},
    };

    pub fn parse_lambda(code: &str) -> Result<LambdaTerm, String> {
        parse_lambda_read_to_end(code)
    }

    pub fn parse_lambda_read_to_end(code: &str) -> Result<LambdaTerm, String> {
        let trees = lex_tree(code).map_err(|e| e.to_string())?;
        let mut parser = Parser::new(normalize_trees(trees)?);
        let term = parser.parse_application(&mut Vec::new())?;
        if !parser.is_eof() {
            return Err(parser.error_here("unexpected trailing tokens"));
        }
        Ok(term)
    }

    #[derive(Clone, Debug)]
    enum Node {
        Token(LexToken),
        Paren(Vec<Node>),
    }

    fn normalize_trees(trees: Vec<Tree>) -> Result<Vec<Node>, String> {
        let mut nodes = Vec::new();
        for tree in trees {
            match tree {
                Tree::Token(LexToken::Whitespace(_) | LexToken::Comment(_)) => {}
                Tree::Token(token) => nodes.push(Node::Token(token)),
                Tree::Delim {
                    delim: DelimKind::Paren,
                    child,
                } => nodes.push(Node::Paren(normalize_trees(child)?)),
                Tree::Delim { delim, .. } => {
                    return Err(format!("unexpected delimiter in lambda term: {:?}", delim));
                }
            }
        }
        Ok(nodes)
    }

    struct Parser {
        nodes: Vec<Node>,
        pos: usize,
    }

    impl Parser {
        fn new(nodes: Vec<Node>) -> Self {
            Self { nodes, pos: 0 }
        }

        fn is_eof(&self) -> bool {
            self.pos >= self.nodes.len()
        }

        fn peek(&self) -> Option<&Node> {
            self.nodes.get(self.pos)
        }

        fn next(&mut self) -> Option<Node> {
            let node = self.nodes.get(self.pos).cloned();
            if node.is_some() {
                self.pos += 1;
            }
            node
        }

        fn error_here(&self, message: impl Into<String>) -> String {
            match self.peek() {
                Some(node) => format!("{} near {:?}", message.into(), node),
                None => format!("{} at end of input", message.into()),
            }
        }

        fn starts_atom(&self) -> bool {
            matches!(
                self.peek(),
                Some(Node::Token(LexToken::Ident(_)))
                    | Some(Node::Token(LexToken::Symbol('\\')))
                    | Some(Node::Paren(_))
            )
        }

        fn parse_application(&mut self, ref_vars: &mut Vec<Var>) -> Result<LambdaTerm, String> {
            let mut terms = Vec::new();
            while self.starts_atom() {
                terms.push(self.parse_atom(ref_vars)?);
            }

            if terms.is_empty() {
                return Err(self.error_here("expected lambda term"));
            }

            Ok(app_with_nonepmty(terms))
        }

        fn parse_atom(&mut self, ref_vars: &mut Vec<Var>) -> Result<LambdaTerm, String> {
            if matches!(self.peek(), Some(Node::Token(LexToken::Symbol('\\')))) {
                return self.parse_abs(ref_vars);
            }
            if matches!(self.peek(), Some(Node::Paren(_))) {
                let Some(Node::Paren(inner)) = self.next() else {
                    unreachable!();
                };
                let mut parser = Parser::new(inner);
                let term = parser.parse_application(ref_vars)?;
                if !parser.is_eof() {
                    return Err(parser.error_here("unexpected trailing tokens in paren group"));
                }
                return Ok(term);
            }
            match self.next() {
                Some(Node::Token(LexToken::Ident(name))) => {
                    let var = ref_vars
                        .iter()
                        .rev()
                        .find_map(|v| {
                            if v.as_str() == name {
                                Some(v.clone())
                            } else {
                                None
                            }
                        })
                        .unwrap_or_else(|| Var::new(&name));
                    Ok(LambdaTerm::Var(var))
                }
                _ => Err(self.error_here("expected variable, abstraction, or parenthesized term")),
            }
        }

        fn parse_abs(&mut self, ref_vars: &mut Vec<Var>) -> Result<LambdaTerm, String> {
            match self.next() {
                Some(Node::Token(LexToken::Symbol('\\'))) => {}
                _ => return Err(self.error_here("expected '\\'")),
            }

            let mut vars = Vec::new();
            while matches!(self.peek(), Some(Node::Token(LexToken::Ident(_)))) {
                let Some(Node::Token(LexToken::Ident(name))) = self.next() else {
                    unreachable!();
                };
                let var = Var::new(&name);
                ref_vars.push(var.clone());
                vars.push(var);
            }

            if vars.is_empty() {
                return Err(self.error_here("expected binder after '\\'"));
            }

            match self.next() {
                Some(Node::Token(LexToken::Symbol('.'))) => {}
                _ => return Err(self.error_here("expected '.'")),
            }
            let body = self.parse_application(ref_vars)?;
            for _ in 0..vars.len() {
                ref_vars.pop();
            }
            Ok(utility::lambdas(vars, body))
        }
    }
}
