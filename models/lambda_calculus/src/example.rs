use crate::machine::*;

pub fn true_lambda() -> LambdaTerm {
    LambdaTerm::abs(0, LambdaTerm::abs(1, LambdaTerm::var(0)))
}

pub fn false_lambda() -> LambdaTerm {
    LambdaTerm::abs(0, LambdaTerm::abs(1, LambdaTerm::var(1)))
}

pub fn is_true(term: LambdaTerm) -> bool {
    let l = true_lambda();
    alpha_eq(&l, &term)
}

pub fn is_false(term: LambdaTerm) -> bool {
    let l = false_lambda();
    alpha_eq(&l, &term)
}

pub fn if_lambda(l: LambdaTerm, m: LambdaTerm, n: LambdaTerm) -> LambdaTerm {
    LambdaTerm::app(LambdaTerm::app(l, m), n)
}

pub fn pair(m: LambdaTerm, n: LambdaTerm) -> LambdaTerm {
    LambdaTerm::abs(
        0,
        LambdaTerm::app(LambdaTerm::app(LambdaTerm::var(0), m), n),
    )
}

pub fn first() -> LambdaTerm {
    LambdaTerm::abs(0, LambdaTerm::app(LambdaTerm::var(0), true_lambda()))
}

pub fn second() -> LambdaTerm {
    LambdaTerm::abs(0, LambdaTerm::app(LambdaTerm::var(0), false_lambda()))
}

pub fn take_n_abs(list: Vec<usize>, term: LambdaTerm) -> LambdaTerm {
    if let Some((head, tail)) = list.split_first() {
        LambdaTerm::abs(*head, take_n_abs(tail.to_owned(), term))
    } else {
        term
    }
}

pub fn fold_left(list: Vec<LambdaTerm>) -> LambdaTerm {
    list.into_iter().reduce(LambdaTerm::app).unwrap()
}

pub fn y_combinator() -> LambdaTerm {
    LambdaTerm::abs(
        0,
        LambdaTerm::app(
            LambdaTerm::abs(
                1,
                LambdaTerm::app(
                    LambdaTerm::var(0),
                    LambdaTerm::app(LambdaTerm::var(1), LambdaTerm::var(1)),
                ),
            ),
            LambdaTerm::abs(
                1,
                LambdaTerm::app(
                    LambdaTerm::var(0),
                    LambdaTerm::app(LambdaTerm::var(1), LambdaTerm::var(1)),
                ),
            ),
        ),
    )
}
