pub mod parse {
    use std::ops::Range;

    use crate::machine::LambdaTerm;

    pub fn brackets(str: &str, start: usize) -> Result<usize, ()> {
        if str.len() <= start {
            return Err(());
        }
        let mut stack = Vec::new();
    
        for (index, character) in str[start..].chars().enumerate() {
            if character == '(' {
                stack.push(index);
            } else if character == ')' {
                if stack.pop().is_none() {
                    return Err(());
                };
                if stack.is_empty() {
                    return Ok(start + index);
                }
            }
        }
        return Err(());
    }

    // Some(index) = index -> index <= self.str.len()
    struct SplitWhiteBracket<'a> {
        str: &'a str,
        index: Option<usize>,
    }

    impl<'a> SplitWhiteBracket<'a> {
        fn new(str: &'a str) -> Self {
            SplitWhiteBracket { str, index: Some(0) }
        }
    }

    impl<'a> Iterator for SplitWhiteBracket<'a> {
        type Item = Range<usize>;
        fn next(&mut self) -> Option<Self::Item> {
            if let Some(start) = self.index {
                let end: usize = {
                    let first_char = self.str.chars().nth(start).unwrap();
                    if first_char == '(' {
                        brackets(self.str, start)
                        .map(|i| i + 1)
                        .unwrap_or(self.str.len())
                    } else {
                        self.str[start..]
                        .find(|char: char| char == '(' || char.is_whitespace())
                        .map(|off_set| start + off_set)
                        .unwrap_or(self.str.len())
                    }
                };

                // end <= str.len()
                let range = start..end;

                // end == str.len() -> self.index = None; -> iter.next() = None;
                self.index = if let Some(off_set) = self.str[end..].find(|char: char| !char.is_whitespace()) {
                    if end + off_set < self.str.len() { Some(end + off_set) } else { None }
                } else {
                    None
                };
                Some(range)
            } else {
                None
            }
        }
    }

    pub fn parse_lambda(str: &str) -> Result<LambdaTerm, ()> {
        let first_char = if let Some(index) = str.chars().nth(0) {
            index
        } else {
            return Err(());
        };
        if first_char == '\\' {
            if let Some(index)  = str.find('.') {
                let var: usize = str[1..index].parse::<usize>().map_err(|_| ())?;
                let term = parse_lambda(&str[index+1..])?;
                Ok(LambdaTerm::abs(var, term))
            } else {
                Err(())
            }
        } else {
            let term: Option<LambdaTerm> = SplitWhiteBracket::new(str.trim()).map(|range|{
                if !str[range.clone()].contains(|char: char| char == '(' || char == ')') {
                    let u = str[range].parse().map_err(|_| ());
                    Ok(LambdaTerm::var(u?))
                } else {
                    parse_lambda(&str[range.start+1..range.end-1])
                }
            }).collect::<Result<Vec<LambdaTerm>, ()>>()?
            .into_iter().reduce(LambdaTerm::app);
            term.ok_or_else(||())
        }
    }

    #[cfg(test)]
    mod tests {
        #[test]
        fn curly_test() {
            let tests = vec![
                ("()", 0, Ok(1)),
                (")", 0, Err(())),
                ("(", 0, Err(())),
                ("(())", 0, Ok(3)),
                ("(()())", 0, Ok(5)),
                ("(()())", 1, Ok(2)),
                ("(()())", 3, Ok(4)),
            ];
    
            for (str, index, expect) in tests {
                assert_eq!(brackets(str, index), expect);
            }
        }
        #[test]
        fn split_test() {
            let str = "0";
            let mut iter = SplitWhiteBracket::new(str);
            assert_eq!(iter.next(), Some(0..1));
            assert_eq!(iter.next(), None);

            let str = "0  ";
            let mut iter = SplitWhiteBracket::new(str);
            assert_eq!(iter.next(), Some(0..1));
            assert_eq!(iter.next(), None);

            let str = "0 1";
            let mut iter = SplitWhiteBracket::new(str);
            assert_eq!(iter.next(), Some(0..1));
            assert_eq!(iter.next(), Some(2..3));
            assert_eq!(iter.next(), None);

            let str = "0 1 ";
            let mut iter = SplitWhiteBracket::new(str);
            assert_eq!(iter.next(), Some(0..1));
            assert_eq!(iter.next(), Some(2..3));
            assert_eq!(iter.next(), None);

            let str = "0 (0 1) 1";
            let mut iter = SplitWhiteBracket::new(str);
            assert_eq!(iter.next(), Some(0..1));
            assert_eq!(iter.next(), Some(2..7));
            assert_eq!(iter.next(), Some(8..9));
            assert_eq!(iter.next(), None);

            let str = "0(0 1)1";
            let mut iter = SplitWhiteBracket::new(str);
            assert_eq!(iter.next(), Some(0..1));
            assert_eq!(iter.next(), Some(1..6));
            assert_eq!(iter.next(), Some(6..7));
            assert_eq!(iter.next(), None);

            let str = "0  ((0) 1)1";
            let mut iter = SplitWhiteBracket::new(str);
            assert_eq!(iter.next(), Some(0..1));
            assert_eq!(iter.next(), Some(3..10));
            assert_eq!(iter.next(), Some(10..11));
            assert_eq!(iter.next(), None);

            let str = "0  ((0) 1)1 ";
            let mut iter = SplitWhiteBracket::new(str);
            assert_eq!(iter.next(), Some(0..1));
            assert_eq!(iter.next(), Some(3..10));
            assert_eq!(iter.next(), Some(10..11));
            assert_eq!(iter.next(), None);
        }
        #[test]
        fn parse_test() {
            let tests = vec![
                ("0", Ok(LambdaTerm::var(0))),
                ("1", Ok(LambdaTerm::var(1))),
                ("\\0.0", Ok(
                    LambdaTerm::abs(0, LambdaTerm::var(0))
                )),
                ("\\0.\\1.0", Ok(
                    LambdaTerm::abs(0,
                        LambdaTerm::abs(1, LambdaTerm::Variable(0.into()))
                    )
                )),
                ("0 1", Ok(
                    LambdaTerm::app(LambdaTerm::var(0), LambdaTerm::var(1))
                )),
                ("(0 1) 2", Ok(
                    LambdaTerm::app(LambdaTerm::app(LambdaTerm::var(0), LambdaTerm::var(1)), LambdaTerm::var(2))
                )),
                ("\\0.1 2", Ok(
                    LambdaTerm::abs(0, LambdaTerm::app(LambdaTerm::var(1), LambdaTerm::var(2)))
                )),
                ("\\0.(1 2)", Ok(
                    LambdaTerm::abs(0, LambdaTerm::app(LambdaTerm::var(1), LambdaTerm::var(2)))
                )),
            ];
            for (str, expect) in tests {
                assert_eq!(parse_lambda(str), expect)
            }
        }
        use super::*;

    }
}
