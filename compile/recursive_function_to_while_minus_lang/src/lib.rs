use recursive_function::machine::RecursiveFunctions;
use while_minus_lang::machine::*;

// 入力と出力に使うレジスターは指定できるようにする、また触ってはいけないレジスターを指定できるようにする。
// input, output, used は交差してはいけない。
// また、入力となるレジスターは値を変更しないこと

// 0...n-1,n を入出力に使ってる lang を、 (temp, ... temp+n) を入出力に使いそれ移行を一時変数として使う関数に変更する。
fn shift_vars(prog: WhileLanguage, n: usize, temp: usize) -> WhileLanguage {
    let mut vec: Vec<(Var, Var)> = (0..=n).map(|i| (i.into(), (temp + i).into())).collect();
    vec.extend(
        prog.changed_var()
            .into_iter()
            .enumerate()
            .map(|(i, v)| (v, (temp + n + 1 + i).into())),
    );
    prog.change_var(&vec.into_iter().collect())
}

fn zero() -> WhileLanguage {
    vec![WhileStatement::init(0.into())].into()
}

fn succ() -> WhileLanguage {
    vec![
        WhileStatement::copy(1.into(), 0.into()),
        WhileStatement::inc(1.into()),
    ]
    .into()
}

fn proj(n: usize, i: usize) -> WhileLanguage {
    vec![WhileStatement::copy(n.into(), i.into())].into()
}

// p_outer: ((0,...m-1) -> Var(m))
// p_inner = [p_0,...p_{m-1}]: vec of ((0,...n-1) -> Var(n))
// (Var(0) = x_0, ... Var(n-1) = x_{n-1}) ->
fn composition(n: usize, p_outer: WhileLanguage, p_inner: Vec<WhileLanguage>) -> WhileLanguage {
    let m = p_inner.len();

    let mut vec: Vec<WhileStatement> = vec![];

    // (n + m + 0, ... n + m + m) に (p_innter[0] (Var(n)), ... p_inner[m-1] (Var(n))) をコピーして取っておく
    let shift_inner = n + m;

    // (0,...,n-1) -> (n+m+0, ... n+m+n-1) とコピー
    for i in 0..n {
        vec.push(WhileStatement::copy((shift_inner + i).into(), i.into()))
    }

    // p_inner[i] を (n+m+0,...,n+m+n-1) を入力にして動かした後に出力の n+m+n を n+i に
    for (i, p) in p_inner.into_iter().enumerate() {
        vec.extend::<Vec<_>>(shift_vars(p, n, shift_inner).into());
        vec.push(WhileStatement::copy(
            (n + i).into(),
            (shift_inner + n).into(),
        ))
    }

    let shift_outer = n;
    vec.extend::<Vec<_>>(shift_vars(p_outer, m, shift_outer).into());
    vec.push(WhileStatement::copy(n.into(), (n + m).into()));

    vec.into()
}

// (0,...,n-1,n) で一つ目の変数で原始再帰する
fn primitive_recursion(n: usize, zero: WhileLanguage, succ: WhileLanguage) -> WhileLanguage {
    let mut vec: Vec<WhileStatement> = vec![];
    // (x_0,... , x_{n-1})
    for i in 0..n {
        vec.push(WhileStatement::copy((n + i).into(), i.into()));
    }

    // (x_0,... , x_{n-1}, x_0,... ,x_{n-1})
    vec.extend::<Vec<_>>(shift_vars(zero, n - 1, n + 1).into());

    // (x_0,... , x_{n-1}, x_0,... ,x_{n-1}, zero(x_1,... x_{n-1}))
    vec.push(WhileStatement::copy(n.into(), (2 * n).into()));
    for i in 0..n {
        vec.push(WhileStatement::copy((n + i + 1).into(), i.into()));
    }
    // (x_0,... ,x_{n-1}, zero(x_1, ... x_n), x_0, ..., x_{n-1})

    vec.push(WhileStatement::while_not_zero((n + 1).into(), {
        let mut inner = vec![];
        inner.extend::<Vec<_>>(shift_vars(succ, n + 1, n).into());
        inner.push(WhileStatement::copy(n.into(), (2 * n + 1).into()));
        inner.push(WhileStatement::dec((n + 1).into()));
        inner
    }));

    vec.into()
}

// (0,...n-1) で一つ目の変数で mu 再帰する
fn mu_recursion(n: usize, muop: WhileLanguage) -> WhileLanguage {
    let mut vec: Vec<WhileStatement> = vec![WhileStatement::init(n.into())];
    for i in 0..n {
        vec.push(WhileStatement::copy((n + i + 1).into(), i.into()));
    }
    // (x_0,... ,x_{n-1}, 0, x_0,... ,x_{n-1})
    vec.extend::<Vec<_>>(shift_vars(muop.clone(), n - 1, n).into());

    // (x_0,... ,x_{n-1}, 0, x_0,... ,x_{n-1}, m(0, x_0,... ,x_{n-1}))
    vec.push(WhileStatement::while_not_zero((2 * n + 1).into(), {
        let mut inner = vec![];
        // (x_0,... ,x_{n-1}, t, x_0,..., x_{n-1}, ?)
        inner.push(WhileStatement::inc(n.into()));
        // (x_0,... ,x_{n-1}, t+1, x_0,..., x_{n-1}, ?)
        inner.extend::<Vec<_>>(shift_vars(muop, n - 1, n).into());
        // (x_0,... ,x_{n-1}, t+1, x_0,..., x_{n-1}, m(t+1, x_0,... x_{n-1}))
        inner
    }));
    vec.into()
}

pub fn compile(func: &RecursiveFunctions) -> WhileLanguage {
    match func {
        RecursiveFunctions::ZeroConstant => zero(),
        RecursiveFunctions::Successor => succ(),
        RecursiveFunctions::Projection(projection) => {
            proj(projection.parameter_length(), projection.projection_num())
        }
        RecursiveFunctions::Composition(comp) => composition(
            comp.parameter_length,
            compile(comp.outer_func.as_ref()),
            (comp.inner_func.as_ref()).iter().map(compile).collect(),
        ),
        RecursiveFunctions::PrimitiveRecursion(prim) => {
            let n = &prim.zero_func.parameter_length() + 1;
            primitive_recursion(
                n,
                compile(prim.zero_func.as_ref()),
                compile(prim.succ_func.as_ref()),
            )
        }
        RecursiveFunctions::MuOperator(muop) => {
            let n = muop.mu_func.parameter_length();
            mu_recursion(n, compile(muop.mu_func.as_ref()))
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn add() {
        // let func = RecursiveFunctions::
    }
}
