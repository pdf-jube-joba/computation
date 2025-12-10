use lambda_calculus::machine::LambdaTerm;
use recursive_function::machine::RecursiveFunctions;
use utils::number::*;
use utils::variable::Var;

fn v(var: &Var) -> LambdaTerm {
    LambdaTerm::Var(var.clone())
}

fn abs(var: &Var, body: LambdaTerm) -> LambdaTerm {
    LambdaTerm::Abs(var.clone(), body.into())
}

fn app(lhs: LambdaTerm, rhs: LambdaTerm) -> LambdaTerm {
    LambdaTerm::App(lhs.into(), rhs.into())
}

fn take_n_abs(vars: Vec<Var>, term: LambdaTerm) -> LambdaTerm {
    if let Some((head, tail)) = vars.split_first() {
        abs(head, take_n_abs(tail.to_owned(), term))
    } else {
        term
    }
}

fn fold_left(list: Vec<LambdaTerm>) -> LambdaTerm {
    list.into_iter().reduce(app).unwrap()
}

fn if_lambda(l: LambdaTerm, m: LambdaTerm, n: LambdaTerm) -> LambdaTerm {
    app(app(l, m), n)
}

// \f.(\x.f(xx))(\x.f(xx))
fn y_combinator() -> LambdaTerm {
    let x = Var::from("0");
    let f = Var::from("1");
    let inner = abs(&x, app(v(&f), app(v(&x), v(&x))));
    abs(&f, app(inner.clone(), inner))
}

// \xy.x
fn true_lambda() -> LambdaTerm {
    let x = Var::from("0");
    let y = Var::from("1");
    abs(&x, abs(&y, v(&x)))
}

// \xy.y
fn false_lambda() -> LambdaTerm {
    let x = Var::from("0");
    let y = Var::from("1");
    abs(&x, abs(&y, v(&y)))
}

pub fn number_to_lambda_term(num: Number) -> LambdaTerm {
    fn term(num: Number, zero: &Var, one: &Var) -> LambdaTerm {
        if num.is_zero() {
            v(one)
        } else {
            app(v(zero), term(num.pred(), zero, one))
        }
    }

    let zero = Var::from("0");
    let one = Var::from("1");
    abs(&zero, abs(&one, term(num, &zero, &one)))
}

pub fn lambda_term_to_number(term: LambdaTerm) -> Option<Number> {
    let (var1, var2, term) = if let LambdaTerm::Abs(var1, term) = term {
        if let LambdaTerm::Abs(var2, term) = *term {
            (var1, var2, term)
        } else {
            return None;
        }
    } else {
        return None;
    };
    let mut iter_term = *term;
    for i in 0.. {
        match &iter_term {
            LambdaTerm::Var(var) => {
                if *var == var2 {
                    return Some(i.into());
                } else {
                    return None;
                }
            }
            LambdaTerm::App(var, term2) => {
                if let LambdaTerm::Var(v) = *var.clone() {
                    if v == var1 {
                        iter_term = *term2.to_owned();
                        continue;
                    }
                } else {
                    return None;
                }
            }
            _ => {
                return None;
            }
        }
    }
    unreachable!()
}

// \xyz.x(\pq.q(py))(\v.z)(\v.v) = \xyz.xMNL
pub fn pred() -> LambdaTerm {
    let v0 = Var::from("0");
    let v1 = Var::from("1");
    let v2 = Var::from("2");
    let v3 = Var::from("3");
    let v4 = Var::from("4");
    let v5 = Var::from("5");

    let m = abs(&v3, abs(&v4, app(v(&v4), app(v(&v3), v(&v0)))));
    let n = abs(&v5, v(&v1));
    let l = abs(&v5, v(&v5));
    abs(&v2, abs(&v0, abs(&v1, app(app(app(v(&v2), m), n), l))))
}

pub fn is_zero() -> LambdaTerm {
    let v0 = Var::from("0");
    let v1 = Var::from("1");
    abs(
        &v0,
        app(app(v(&v0), abs(&v1, false_lambda())), true_lambda()),
    )
}

// \xyz.y(xyz)
pub fn succ() -> LambdaTerm {
    let v0 = Var::from("0");
    let v1 = Var::from("1");
    let v2 = Var::from("2");
    abs(
        &v2,
        abs(&v0, abs(&v1, app(v(&v0), app(app(v(&v2), v(&v0)), v(&v1))))),
    )
}

// \x1,,,xn.xi
pub fn projection(n: usize, i: usize) -> Option<LambdaTerm> {
    if n < i {
        None
    } else {
        let vars: Vec<Var> = (0..n).map(|idx| Var::from(idx.to_string())).collect();
        let target = vars.get(i)?.clone();
        Some(take_n_abs(vars, v(&target)))
    }
}

// \x1,,,xn. outer (inner x1,,,xn) ,,, (inner x1,,,xn)
pub fn composition(n: usize, inner: Vec<LambdaTerm>, outer: LambdaTerm) -> LambdaTerm {
    let vars: Vec<Var> = (0..n).map(|idx| Var::from(idx.to_string())).collect();
    let mut v = vec![outer];
    v.extend(inner.into_iter().map(|term| {
        fold_left({
            let mut v2 = vec![term];
            v2.extend(vars.iter().map(|var| var.into()));
            v2
        })
    }));
    let fold = fold_left(v);
    take_n_abs(vars, fold)
}

// THIS = \x0,,,xn. if (iszero x0) (f x1,,,xn) (g (THIS (pred x0) x1,,,xn) (pred x0) x1,,,xn)
pub fn primitive_recursion(n: usize, f: LambdaTerm, g: LambdaTerm) -> LambdaTerm {
    let vars: Vec<Var> = (0..=n).map(|idx| Var::from(idx.to_string())).collect();
    let n_plus_one = Var::from((n + 1).to_string());

    // is_zero 0
    let is_zero = app(is_zero(), v(&vars[0]));

    // f x1 ... xn
    let f_new = fold_left({
        let mut v = vec![f];
        v.extend(vars.iter().skip(1).map(|var| var.into()));
        v
    });

    // g (n+1 (pred 0) 1 ... n) (pred 0) 1 ... n
    let g_new = {
        let pred_0 = app(pred(), v(&vars[0]));
        let p = {
            let mut v = vec![v(&n_plus_one), pred_0.clone()];
            v.extend(vars.iter().skip(1).map(|var| var.into()));
            fold_left(v)
        };
        let mut t = vec![g, p, pred_0];
        t.extend(vars.iter().skip(1).map(v));
        fold_left(t)
    };
    let fix = if_lambda(is_zero, f_new, g_new);

    // Y (\n+1 1...n. if_lambda is_zero f g) =>
    // n+1 <=> H として H x0 x1 ... xn = if_lambda is_zero f g
    let mut abs_vars = Vec::with_capacity(n + 2);
    abs_vars.push(n_plus_one.clone());
    abs_vars.extend(vars);
    app(y_combinator(), take_n_abs(abs_vars, fix))
}

pub fn mu_recursion(n: usize, f: LambdaTerm) -> LambdaTerm {
    let vars: Vec<Var> = (0..=n).map(|idx| Var::from(idx.to_string())).collect();
    let n_plus_one = Var::from((n + 1).to_string());

    let is_zero = take_n_abs(
        vars.clone(),
        fold_left({
            let mut v = vec![f];
            v.extend(vars.iter().map(|var| var.into()));
            v
        }),
    );
    let rec = fold_left({
        let mut v = vec![v(&n_plus_one), app(succ(), v(&vars[0]))];
        v.extend(vars.iter().skip(1).map(|var| var.into()));
        v
    });
    let fix = if_lambda(is_zero, v(&vars[0]), rec);

    let mut abs_vars = Vec::with_capacity(n + 2);
    abs_vars.push(n_plus_one.clone());
    abs_vars.extend(vars);
    app(y_combinator(), take_n_abs(abs_vars, fix))
}

pub fn compile(func: &RecursiveFunctions) -> LambdaTerm {
    match func {
        RecursiveFunctions::ZeroConstant => number_to_lambda_term(0.into()),
        RecursiveFunctions::Successor => succ(),
        RecursiveFunctions::Projection {
            parameter_length,
            projection_num,
        } => projection(*parameter_length, *projection_num).unwrap(),
        RecursiveFunctions::Composition {
            parameter_length,
            outer_func,
            inner_funcs,
        } => composition(
            *parameter_length,
            inner_funcs.iter().map(compile).collect(),
            compile(outer_func.as_ref()),
        ),
        RecursiveFunctions::PrimitiveRecursion {
            zero_func,
            succ_func,
        } => primitive_recursion(
            zero_func.parameter_length() + 1,
            compile(zero_func.as_ref()),
            compile(succ_func.as_ref()),
        ),
        RecursiveFunctions::MuOperator { mu_func } => {
            mu_recursion(mu_func.parameter_length(), compile(mu_func.as_ref()))
        }
    }
}
