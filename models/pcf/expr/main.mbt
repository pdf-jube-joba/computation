///|
struct State {
  code : @s.Expr
  expr : @s.Expr
}

///|
fn small(e : @s.Expr) -> (@s.Expr, @s.Output) raise @utils.Fault {
  match e {
    @s.Print(n) => (@s.Unit, @s.Printed(n))
    @s.BinOp(a, o, b) =>
      if !@s.is_value(a) {
        let (a, out) = small(a)
        (@s.BinOp(a, o, b), out)
      } else if !@s.is_value(b) {
        let (b, out) = small(b)
        (@s.BinOp(a, o, b), out)
      } else {
        (@s.binop(o, a, b), @s.Silent)
      }
    @s.UnOp(o, e) =>
      if !@s.is_value(e) {
        let (e, out) = small(e)
        (@s.UnOp(o, e), out)
      } else {
        (@s.unop(o, e), @s.Silent)
      }
    @s.App(f, a) =>
      if !@s.is_value(f) {
        let (f, out) = small(f)
        (@s.App(f, a), out)
      } else if !@s.is_value(a) {
        let (a, out) = small(a)
        (@s.App(f, a), out)
      } else {
        match f {
          @s.Fun(x, b) => (@s.substitute(b, x, a), @s.Silent)
          @s.Rec(f, x, b) as r =>
            (@s.substitute(@s.substitute(b, f, r), x, a), @s.Silent)
          _ =>
            @utils.error(
              "application expects a function value, found " + @s.print(f),
            )
        }
      }
    @s.If(c, t, e) =>
      if !@s.is_value(c) {
        let (c, out) = small(c)
        (@s.If(c, t, e), out)
      } else {
        match c {
          @s.Bool(b) => (if b { t } else { e }, @s.Silent)
          _ =>
            @utils.error("if expects a boolean condition, found " + @s.print(c))
        }
      }
    @s.Var(x) => @utils.error("unbound variable: " + x)
    @s.Block(_, _) =>
      @utils.error("blocks belong to the expression/statement machine")
    _ => @utils.error("cannot step a value")
  }
}

///|
fn make(code : @s.Expr, input : @s.Inputs) -> State {
  { code, expr: @s.apply_inputs(code, input.values), }
}

///|
fn step(
  s : State,
  _input : Rinput,
) -> @definition.Transition[State, Routput, Foutput] raise @utils.Fault {
  if @s.is_value(s.expr) {
    match s.expr {
      @s.Nat(n) => @definition.Halt(n)
      _ => @utils.error("final value is not a natural number")
    }
  } else {
    let (expr, out) = small(s.expr)
    @definition.Continue({ ..s, expr, }, out)
  }
}

///|
fn encode(s : State) -> Json {
  @utils.object([("code", @s.expr_json(s.code)), ("expr", @s.expr_json(s.expr))])
}

///|
fn decode(j : Json) -> State raise @utils.Fault {
  {
    code: @s.decode_expr(@utils.field(j, "code")),
    expr: @s.decode_expr(@utils.field(j, "expr")),
  }
}

///|
pub fn model() -> @definition.Model[
  Code,
  Ainput,
  Rinput,
  Routput,
  Foutput,
  State,
] {
  @definition.Model(make=(code, input) => make(code, input), step=(state, input) => {
    step(state, input)
  })
}
