pub mod parse {
    use std::ops::Range;

    use crate::machine::LambdaTerm;

    pub fn brackets(str: &str, start: usize) -> Option<usize> {
        if str.len() <= start {
            return None;
        }
        let mut stack = Vec::new();

        for (index, character) in str[start..].chars().enumerate() {
            if character == '(' {
                stack.push(index);
            } else if character == ')' {
                stack.pop()?;
                if stack.is_empty() {
                    return Some(start + index);
                }
            }
        }
        None
    }

    // Some(index) = index -> index <= self.str.len()
    struct SplitWhiteBracket<'a> {
        str: &'a str,
        index: Option<usize>,
    }

    impl<'a> SplitWhiteBracket<'a> {
        fn new(str: &'a str) -> Self {
            SplitWhiteBracket {
                str,
                index: Some(0),
            }
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
                self.index = if let Some(off_set) =
                    self.str[end..].find(|char: char| !char.is_whitespace())
                {
                    if end + off_set < self.str.len() {
                        Some(end + off_set)
                    } else {
                        None
                    }
                } else {
                    None
                };
                Some(range)
            } else {
                None
            }
        }
    }

    pub fn parse_lambda(str: &str) -> Option<LambdaTerm> {
        let first_char = str.chars().next()?;
        if first_char == '\\' {
            if let Some(index) = str.find('.') {
                let var: usize = str[1..index].parse::<usize>().ok()?;
                let term = parse_lambda(&str[index + 1..])?;
                Some(LambdaTerm::abs(var, term))
            } else {
                None
            }
        } else {
            SplitWhiteBracket::new(str.trim())
                .map(|range| {
                    if !str[range.clone()].contains(|char: char| char == '(' || char == ')') {
                        let u: usize = str[range].parse().ok()?;
                        Some(LambdaTerm::var(u))
                    } else {
                        parse_lambda(&str[range.start + 1..range.end - 1])
                    }
                })
                .collect::<Option<Vec<LambdaTerm>>>()?
                .into_iter()
                .reduce(LambdaTerm::app)
        }
    }

    #[cfg(test)]
    mod tests {
        #[test]
        fn curly_test() {
            let tests = vec![
                ("()", 0, Some(1)),
                (")", 0, None),
                ("(", 0, None),
                ("(())", 0, Some(3)),
                ("(()())", 0, Some(5)),
                ("(()())", 1, Some(2)),
                ("(()())", 3, Some(4)),
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
                ("0", Some(LambdaTerm::var(0))),
                ("1", Some(LambdaTerm::var(1))),
                ("\\0.0", Some(LambdaTerm::abs(0, LambdaTerm::var(0)))),
                (
                    "\\0.\\1.0",
                    Some(LambdaTerm::abs(
                        0,
                        LambdaTerm::abs(1, LambdaTerm::Variable(0.into())),
                    )),
                ),
                (
                    "0 1",
                    Some(LambdaTerm::app(LambdaTerm::var(0), LambdaTerm::var(1))),
                ),
                (
                    "(0 1) 2",
                    Some(LambdaTerm::app(
                        LambdaTerm::app(LambdaTerm::var(0), LambdaTerm::var(1)),
                        LambdaTerm::var(2),
                    )),
                ),
                (
                    "\\0.1 2",
                    Some(LambdaTerm::abs(
                        0,
                        LambdaTerm::app(LambdaTerm::var(1), LambdaTerm::var(2)),
                    )),
                ),
                (
                    "\\0.(1 2)",
                    Some(LambdaTerm::abs(
                        0,
                        LambdaTerm::app(LambdaTerm::var(1), LambdaTerm::var(2)),
                    )),
                ),
            ];
            for (str, expect) in tests {
                assert_eq!(parse_lambda(str), expect)
            }
        }
        use super::*;
    }
}
