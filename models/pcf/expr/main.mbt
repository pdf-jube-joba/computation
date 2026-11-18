///|
priv struct State {
  code : @s.Expr
  expr : @s.Expr
}

///|
fn small(e : @s.Expr) -> (@s.Expr, String) raise @base.Fault {
  match e {
    @s.Print(n) => (@s.Unit, n.to_string())
    @s.BinOp(a, o, b) =>
      if !@s.is_value(a) {
        let (a, out) = small(a)
        (@s.BinOp(a, o, b), out)
      } else if !@s.is_value(b) {
        let (b, out) = small(b)
        (@s.BinOp(a, o, b), out)
      } else {
        (@s.binop(o, a, b), "")
      }
    @s.UnOp(o, e) =>
      if !@s.is_value(e) {
        let (e, out) = small(e)
        (@s.UnOp(o, e), out)
      } else {
        (@s.unop(o, e), "")
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
          @s.Fun(x, b) => (@s.substitute(b, x, a), "")
          @s.Rec(f, x, b) as r =>
            (@s.substitute(@s.substitute(b, f, r), x, a), "")
          _ =>
            @base.error(
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
          @s.Bool(b) => (if b { t } else { e }, "")
          _ =>
            @base.error("if expects a boolean condition, found " + @s.print(c))
        }
      }
    @s.Var(x) => @base.error("unbound variable: " + x)
    @s.Block(_, _) =>
      @base.error("blocks belong to the expression/statement machine")
    _ => @base.error("cannot step a value")
  }
}

///|
fn make(code : String, input : String) -> State raise @base.Fault {
  let code = @s.parse(code)
  { code, expr: @s.apply_inputs(code, @s.inputs(input)), }
}

///|
fn step(
  s : State,
  input : String,
) -> @runtime.Transition[State] raise @base.Fault {
  @base.unit_input(input)
  if @s.is_value(s.expr) {
    match s.expr {
      @s.Nat(n) => @runtime.Halt(n.to_string())
      _ => @base.error("final value is not a natural number")
    }
  } else {
    let (expr, out) = small(s.expr)
    @runtime.Continue({ ..s, expr, }, out)
  }
}

///|
fn encode(s : State) -> Json {
  @base.object([("code", @s.expr_json(s.code)), ("expr", @s.expr_json(s.expr))])
}

///|
fn decode(j : Json) -> State raise @base.Fault {
  {
    code: @s.decode_expr(@base.field(j, "code")),
    expr: @s.decode_expr(@base.field(j, "expr")),
  }
}

///|
pub fn model() -> @runtime.Model {
  @runtime.model(make, step, encode, decode, s => {
    @base.jarray([
      @base.titled("code", @s.print(s.code)),
      @base.titled("current", @s.print(s.expr)),
    ])
  })
}

///|
test "capture avoidance and strict natural subtraction" {
  let m = model()
  m.make("(\\fun f => f(3))(\\fun x => \\inc x)", "()")
  let mut output = None
  for _ in 0..<20 {
    let j = @base.parse_json(m.step(""))
    if @base.string(@base.field(j, "kind")) == "halt" {
      output = Some(@base.string(@base.field(j, "foutput")))
      break
    }
  }
  assert_eq(output, Some("4"))
  m.make("0 - 1", "()")
  let saved = m.snapshot()
  assert_true(
    try {
      ignore(m.step(""))
      false
    } catch {
      _ => true
    },
  )
  assert_eq(m.snapshot(), saved)
}
