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

// \f.(\x.f(xx))(\x.f(xx)) =: Y
// Y (f) = f (Y f)
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

    let num = num
        .as_usize()
        .map_err(|_| "number too large".to_string())
        .unwrap();

    for _ in 0_usize..num {
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
    abs_vs(
        vec![x.clone(), y.clone(), z.clone()],
        app(v(&y), fold_left(vec![v(&x), v(&y), v(&z)])),
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
    abs_vs(
        vec![x.clone(), y.clone(), z.clone()],
        fold_left(vec![v(&x), m, n, l]),
    )
}

// \n. n (\_.false) true
pub fn is_zero() -> LambdaTerm {
    let n = Var::from("n");
    // \_. false
    let f = abs(&Var::dummy(), false_lambda());
    abs(&n, fold_left(vec![v(&n), f, true_lambda()]))
}

// \x_0,,,x_{n-1}.x_i
pub fn projection(n: usize, i: usize) -> Option<LambdaTerm> {
    let vars: Vec<Var> = (0..n).map(|idx| Var::from(format!("x{idx}"))).collect();
    let target = vars.get(i)?.clone();
    Some(abs_vs(vars, v(&target)))
}

// \x_0,,,x_{n-1}. outer (inner[0] x_0,,,x_{n-1}) ,,, (inner[k] x_0,,,x_{n-1})
pub fn composition(n: usize, inner: Vec<LambdaTerm>, outer: LambdaTerm) -> LambdaTerm {
    let vars: Vec<Var> = (0..n).map(|idx| Var::from(format!("x{idx}"))).collect();

    let mut v = vec![outer];
    for inner_func in inner {
        let mut v2 = vec![inner_func];
        v2.extend(vars.iter().map(|var| var.into()));
        v.push(fold_left(v2));
    }

    abs_vs(vars, fold_left(v))
}

// THIS 0        x_1,,,x_n = f x_1,,,x_n
// THIS (succ x) x_1,,,x_n = g (THIS x x_1,,,x_n) x x_1,,,x_n
// ... => THIS x_0,,,x_n = "if" (iszero x_0) (f x_1,,,x_n) (g (THIS (pred x_0) x_1,,,x_n) (pred x_0) x_1,,,x_n)
//   given by Y (\THIS. \x_0,,,x_n. "if" ...)
pub fn primitive_recursion(n: usize, f: LambdaTerm, g: LambdaTerm) -> LambdaTerm {
    // AI によると `0..=n` ではなくて `0..n` で良いらしい
    // 確かにそれで動いていて、ちょっとわからなかった。
    let vars: Vec<Var> = (0..n).map(|idx| Var::from(format!("x{idx}"))).collect();
    let this = Var::from("THIS");

    // is_zero x_0
    let is_zero = app(is_zero(), v(&vars[0]));

    // f x1 ... xn =: f_new
    let f_new = fold_left({
        let mut v = vec![f];
        v.extend(vars.iter().skip(1).map(|var| var.into()));
        v
    });

    // g (THIS (pred x0) x1 ... xn) (pred x0) x1 ... xn =: g_new
    let g_new = {
        let pred_0 = app(pred(), v(&vars[0]));
        // THIS (pred x0) x1 ... xn =: g_first
        let g_first = {
            let mut v = vec![v(&this), pred_0.clone()];
            v.extend(vars.iter().skip(1).map(|var| var.into()));
            fold_left(v)
        };
        let mut t = vec![g, g_first, pred_0];
        t.extend(vars.iter().skip(1).map(v));
        fold_left(t)
    };
    let fix = if_lambda(is_zero, f_new, g_new);

    // Y (\THIS x0...xn. if_lambda is_zero f g) =>
    // THIS <=> H として H x0 x1 ... xn = if_lambda is_zero f g
    let mut abs_vars = vec![this.clone()];
    abs_vars.extend(vars);
    app(y_combinator(), abs_vs(abs_vars, fix))
}

// THIS x_1,,,x_n = x where 0 == f x x_1,,,x_n (new variable x)
// how to implement:
// INC x_0,x_1,,,x_n = if (iszero (f x_0 x_1,,,x_n)) x_0 (INC (succ x_0) x_1,,,x_n)
//   given by Y (\INC x_0,x_1,,,x_n. "right hand of INC")
// THIS = \x_0,,,x_n. INC 0 x_0,,,x_n (there is no x)
pub fn mu_recursion(n: usize, f: LambdaTerm) -> LambdaTerm {
    let vars: Vec<Var> = (0..=n).map(|idx| Var::from(format!("v{idx}"))).collect();
    let inc = Var::from("INC");

    // iszero (f x_0 x_1,,,x_n)
    let is_zero_f = {
        let mut v = vec![f];
        v.extend(vars.iter().map(|var| var.into()));
        app(is_zero(), fold_left(v))
    };

    // INC (succ x_0) x_1,,,x_n
    let else_clause = {
        let mut v = vec![v(&inc), app(succ(), v(&vars[0]))];
        v.extend(vars.iter().skip(1).map(|var| var.into()));
        fold_left(v)
    };

    let right_hand_of_inc = if_lambda(is_zero_f, v(&vars[0]), else_clause);

    let mut abs_vars = vec![inc.clone()];
    abs_vars.extend(vars.iter().cloned());
    let inc_term = app(y_combinator(), abs_vs(abs_vars, right_hand_of_inc));

    // INC 0 x1,,,xn
    let inc = {
        let mut v = vec![inc_term, number_to_lambda_term(0.into())];
        v.extend(vars.iter().skip(1).map(|var| var.into()));
        fold_left(v)
    };
    abs_vs(vars.iter().skip(1).cloned().collect(), inc)
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
        Ok(lambda_calculus::AInput(encoded))
    }

    fn encode_rinput(
        rinput: <<Self as Compiler>::Source as utils::Machine>::RInput,
    ) -> Result<<<Self as Compiler>::Target as utils::Machine>::RInput, String> {
        let _: () = rinput;
        // leftmost outermost reduction
        Ok(0)
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
    use utils::TextCodec;

    fn normalize(term: &LambdaTerm, limit: usize) -> LambdaTerm {
        let mut marked = mark_redex(term);
        for _ in 0..limit {
            eprintln!("marked: {}", unmark_redex(marked.clone()).print());
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
    fn test_y_combinator() {
        let f = Var::from("f_test");
        let y = y_combinator();
        let applied = app(y.clone(), f.into());
        eprintln!("applied: {}", applied.print());
        let mut marked = mark_redex(&applied);
        for _ in 0..5 {
            eprintln!("marked: {}", unmark_redex(marked.clone()).print());
            if let Some(next) = step(&marked, 0) {
                marked = mark_redex(&next);
            } else {
                break;
            }
        }
    }
    #[test]
    fn test1() {
        // is_zero test
        let is_zero_exp = is_zero();
        for i in 0..3 {
            let i_exp = number_to_lambda_term(i.into());
            let applied = app(is_zero_exp.clone(), i_exp);
            eprintln!("applied: {}", applied.print());
            let normalized = normalize(&applied, 100);
            let expected = if i == 0 {
                true_lambda()
            } else {
                false_lambda()
            };
            eprintln!("expected: {}", expected.print());
            assert!(alpha_eq(&normalized, &expected))
        }

        // succ test
        let succ_exp = succ();
        for i in 0..4 {
            let i_exp = number_to_lambda_term(i.into());
            let applied = app(succ_exp.clone(), i_exp);
            eprintln!("applied: {}", applied.print());
            let normalized = normalize(&applied, 100);
            let expected = number_to_lambda_term((i + 1).into());
            eprintln!("expected: {}", expected.print());
            assert!(alpha_eq(&normalized, &expected))
        }

        // pred_test
        let pred_exp = pred();
        // alpha_eq(pred (n + 1), n)
        for i in 0..4 {
            let i_exp = number_to_lambda_term(i.into());
            let applied = app(pred_exp.clone(), i_exp);
            eprintln!("applied: {}", applied.print());
            let normalized = normalize(&applied, 100);
            let expected = number_to_lambda_term((if i == 0 { 0 } else { i - 1 }).into());
            eprintln!("expected: {}", expected.print());
            assert!(alpha_eq(&normalized, &expected))
        }
    }
    #[test]
    fn compile_primitive_recursion() {
        let f = Var::from("f");
        let g = Var::from("g");
        // THIS 0 = f
        // THIS (succ(x)) = g (THIS x) x
        let e = primitive_recursion(1, v(&f), v(&g));

        // THIS 0 = f
        let e_app0 = app(e.clone(), number_to_lambda_term(0.into()));
        let normal = normalize(&e_app0, 200);
        eprintln!("normalized: {}", normal.print());
        let expected = v(&f);
        eprintln!("expected: {}", expected.print());
        assert!(alpha_eq(&normal, &expected));

        // THIS 1 = g (THIS 0) 0 = g f 0
        let e_app1 = app(e.clone(), number_to_lambda_term(1.into()));
        let normal = normalize(&e_app1, 200);
        eprintln!("normalized: {}", normal.print());

        // THIS 2 = g (THIS 1) 1 = g (g f 0) 1
        let e_app1 = app(e.clone(), number_to_lambda_term(2.into()));
        let normal = normalize(&e_app1, 200);
        eprintln!("normalized: {}", normal.print());
    }
    #[test]
    fn compile_test() {
        let code = r"PRIM[z:PROJ[1,0] s:COMP[SUCC: PROJ[3,0]]]";
        let program = recursive_function::manipulation::parse(code).unwrap();
        let lambda_term = compile(&program);
        let arg = vec![1.into(), 1.into()]
            .into_iter()
            .map(number_to_lambda_term)
            .collect::<Vec<LambdaTerm>>();
        let applied = fold_left({
            let mut v = vec![lambda_term];
            v.extend(arg);
            v
        });
        eprintln!("applied: {}", applied.print());
        let normalized = normalize(&applied, 200);
        let expected = number_to_lambda_term(2.into());
        eprintln!("expected: {}", expected.print());
        assert!(alpha_eq(&normalized, &expected))
    }
}
