use lambda_calculus::machine::LambdaTerm;
use recursive_function::machine::RecursiveFunctions;
use utils::variable::Var;
use utils::{number::*, Compiler};

fn v(var: &Var) -> LambdaTerm {
    LambdaTerm::Var(var.clone())
}

fn abs(var: &Var, body: LambdaTerm) -> LambdaTerm {
    LambdaTerm::Abs(var.clone(), body.into())
}

fn app(lhs: LambdaTerm, rhs: LambdaTerm) -> LambdaTerm {
    LambdaTerm::App(lhs.into(), rhs.into())
}

fn abs_vs(vars: Vec<Var>, mut term: LambdaTerm) -> LambdaTerm {
    for var in vars.into_iter().rev() {
        term = abs(&var, term);
    }
    term
}

fn fold_left(list: Vec<LambdaTerm>) -> LambdaTerm {
    list.into_iter().reduce(app).unwrap()
}

fn if_lambda(l: LambdaTerm, m: LambdaTerm, n: LambdaTerm) -> LambdaTerm {
    app(app(l, m), n)
}

// \f.(\x.f(xx))(\x.f(xx))
fn y_combinator() -> LambdaTerm {
    let x = Var::from("f");
    let f = Var::from("x");
    let inner = abs(&x, app(v(&f), app(v(&x), v(&x))));
    abs(&f, app(inner.clone(), inner))
}

// \xy.x
fn true_lambda() -> LambdaTerm {
    let x = Var::from("x");
    let y = Var::from("y");
    abs(&x, abs(&y, v(&x)))
}

// \xy.y
fn false_lambda() -> LambdaTerm {
    let x = Var::from("x");
    let y = Var::from("y");
    abs(&x, abs(&y, v(&y)))
}

// church encoding: \s z. (s (z (... (s z)...))) where v0 is applied `num`` times
pub fn number_to_lambda_term(num: Number) -> LambdaTerm {
    let zero = Var::from("s");
    let one = Var::from("z");

    let mut body = v(&one);

    for _ in 0_usize..num.into() {
        body = app(v(&zero), body);
    }

    abs(&zero, abs(&one, body))
}

pub fn lambda_term_to_number(term: LambdaTerm) -> Option<Number> {
    let LambdaTerm::Abs(var1, term) = term else {
        return None;
    };
    let LambdaTerm::Abs(var2, term) = *term else {
        return None;
    };

    let mut iter_term = *term;
    for i in 0.. {
        match &iter_term {
            LambdaTerm::Var(var) if *var == var2 => {
                return Some(i.into());
            }
            LambdaTerm::App(var, term2) => {
                let LambdaTerm::Var(v) = *var.clone() else {
                    return None;
                };
                if v == var1 {
                    iter_term = *term2.to_owned();
                    continue;
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

// \xyz.y(xyz)
pub fn succ() -> LambdaTerm {
    let x = Var::from("x_succ");
    let y = Var::from("y_succ");
    let z = Var::from("z_succ");
    abs(
        &x,
        abs(&y, abs(&z, app(v(&y), app(app(v(&x), v(&y)), v(&z))))),
    )
}

// \xyz.xMNL where M = (\pq.q(py)), N = (\v.z), L = (\v.v)
pub fn pred() -> LambdaTerm {
    let x = Var::from("x_pred");
    let y = Var::from("y_pred");
    let z = Var::from("z_pred");
    let p = Var::from("p_pred");
    let q = Var::from("q_pred");
    let v_var = Var::from("v_pred");

    let m = abs(&p, abs(&q, app(v(&q), app(v(&p), v(&y)))));
    let n = abs(&v_var, v(&z));
    let l = abs(&v_var, v(&v_var));
    abs(&x, abs(&y, abs(&z, app(app(app(v(&x), m), n), l))))
}

// \n. n (\_.false) true
pub fn is_zero() -> LambdaTerm {
    let n = Var::from("n");
    // \_. false
    let f = abs(&Var::dummy(), false_lambda());
    abs(&n, app(f, true_lambda()))
}

// \x1,,,xn.xi
pub fn projection(n: usize, i: usize) -> Option<LambdaTerm> {
    let vars: Vec<Var> = (0..n).map(|idx| Var::from(format!("x{idx}"))).collect();
    let target = vars.get(i)?.clone();
    Some(abs_vs(vars, v(&target)))
}

// \x1,,,xn. outer (inner x1,,,xn) ,,, (inner x1,,,xn)
pub fn composition(n: usize, inner: Vec<LambdaTerm>, outer: LambdaTerm) -> LambdaTerm {
    let vars: Vec<Var> = (0..n).map(|idx| Var::from(format!("v{idx}"))).collect();
    let mut v = vec![outer];
    v.extend(inner.into_iter().map(|term| {
        fold_left({
            let mut v2 = vec![term];
            v2.extend(vars.iter().map(|var| var.into()));
            v2
        })
    }));
    let fold = fold_left(v);
    abs_vs(vars, fold)
}

// THIS = \x0,,,xn. if (iszero x0) (f x1,,,xn) (g (THIS (pred x0) x1,,,xn) (pred x0) x1,,,xn)
pub fn primitive_recursion(n: usize, f: LambdaTerm, g: LambdaTerm) -> LambdaTerm {
    let vars: Vec<Var> = (0..=n).map(|idx| Var::from(format!("v{idx}"))).collect();
    let n_plus_one = Var::from(format!("v{}", n + 1));

    // is_zero 0
    let is_zero = app(is_zero(), v(&vars[0]));

    // f x1 ... xn
    let f_new = fold_left({
        let mut v = vec![f];
        v.extend(vars.iter().skip(1).map(|var| var.into()));
        v
    });

    // g (x{n+1} (pred x0) x1 ... xn) (pred x0) x1 ... xn
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
    app(y_combinator(), abs_vs(abs_vars, fix))
}

pub fn mu_recursion(n: usize, f: LambdaTerm) -> LambdaTerm {
    let vars: Vec<Var> = (0..=n).map(|idx| Var::from(format!("v{idx}"))).collect();
    let n_plus_one = Var::from(format!("v{}", n + 1));

    let is_zero = abs_vs(
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
    app(y_combinator(), abs_vs(abs_vars, fix))
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

pub struct Rec2LamCompiler {}

impl Compiler for Rec2LamCompiler {
    type Source = recursive_function::machine::Program;
    type Target = LambdaTerm;

    fn compile(
        source: <<Self as Compiler>::Source as utils::Machine>::Code,
    ) -> Result<<<Self as Compiler>::Target as utils::Machine>::Code, String> {
        Ok(compile(&source))
    }

    fn encode_ainput(
        ainput: <<Self as Compiler>::Source as utils::Machine>::AInput,
    ) -> Result<<<Self as Compiler>::Target as utils::Machine>::AInput, String> {
        let encoded = ainput
            .into_iter()
            .map(number_to_lambda_term)
            .collect::<Vec<LambdaTerm>>();
        Ok(lambda_calculus::web::AInput(encoded))
    }

    fn encode_rinput(
        rinput: <<Self as Compiler>::Source as utils::Machine>::RInput,
    ) -> Result<<<Self as Compiler>::Target as utils::Machine>::RInput, String> {
        let _: () = rinput;
        // leftmost outermost reduction
        Ok(0.into())
    }

    fn decode_output(
        output: <<Self as Compiler>::Target as utils::Machine>::Output,
    ) -> Result<<<Self as Compiler>::Source as utils::Machine>::Output, String> {
        if let Some(num) = lambda_term_to_number(output.clone()) {
            Ok(num)
        } else {
            Err(format!("failed to decode: {output:?}"))
        }
    }
}

#[cfg(test)]
mod tests {
    use lambda_calculus::machine::{alpha_eq, mark_redex, step, unmark_redex};

    fn normalize(term: &LambdaTerm, limit: usize) -> LambdaTerm {
        let mut marked = mark_redex(term);
        for _ in 0..limit {
            if let Some(next) = step(&marked, 0) {
                marked = mark_redex(&next);
            } else {
                return unmark_redex(marked);
            }
        }
        panic!("normalization did not finish within the limit");
    }

    use super::*;
    #[test]
    fn test1() {
        for i in 0..4 {
            let i_exp = number_to_lambda_term(i.into());
            let succ_exp = succ();
            let applied = app(succ_exp, i_exp);
            eprintln!("applied: {}", applied);
            let normalized = normalize(&applied, 100);
            let expected = number_to_lambda_term((i + 1).into());
            eprintln!("expected: {}", expected);
            assert!(alpha_eq(&normalized, &expected))
        }
        // alpha_eq(pred 0, 0)
        {
            let zero = number_to_lambda_term(0.into());
            let pred_exp = pred();
            let applied = app(pred_exp, zero);
            eprintln!("applied: {}", applied);
            let normalized = normalize(&applied, 100);
            let expected = number_to_lambda_term(0.into());
            eprintln!("expected: {}", expected);
            assert!(alpha_eq(&normalized, &expected))
        }
        // alpha_eq(pred (n + 1), n)
        for i in 1..4 {
            let i_exp = number_to_lambda_term(i.into());
            let pred_exp = pred();
            let applied = app(pred_exp, i_exp);
            eprintln!("applied: {}", applied);
            let normalized = normalize(&applied, 100);
            let expected = number_to_lambda_term((i - 1).into());
            eprintln!("expected: {}", expected);
            assert!(alpha_eq(&normalized, &expected))
        }
    }
}
