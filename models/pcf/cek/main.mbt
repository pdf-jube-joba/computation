///|
priv enum Frame {
  BinOpLeft(@s.Binop, @s.Expr, @s.Env)
  BinOpRight(@s.Value, @s.Binop)
  Unary(@s.Unop)
  AppFun(@s.Expr, @s.Env)
  AppArg(@s.Value)
  IfExpr(@s.Expr, @s.Expr, @s.Env)
  LetFrame(String, @s.Env)
  SeqFrame(@s.Stmt)
  IfStmtFrame(@s.Stmt, @s.Env)
  WhileFrame(@s.Expr, @s.Stmt, @s.Env)
  ScopeOut(@s.Expr)
  Init
}

///|
priv enum Phase {
  EvalExpr(@s.Expr, @s.Env, Array[Frame])
  Return(@s.Value, Array[Frame])
  EvalStmt(@s.Stmt, @s.Env, Array[Frame])
  Done(@s.Env, Array[Frame])
}

///|
struct State {
  code : @s.Expr
  state : Phase
}

///|
fn push(f : Frame, k : Array[Frame]) -> Array[Frame] {
  let a = [f]
  a.append(k)
  a
}

///|
fn transition(
  phase : Phase,
  statements : Bool,
) -> (Phase, @s.Output) raise @utils.Fault {
  match phase {
    EvalExpr(e, env, k) =>
      match e {
        @s.Nat(n) => (Return(@s.VNat(n), k), @s.Silent)
        @s.Bool(b) => (Return(@s.VBool(b), k), @s.Silent)
        @s.Unit => (Return(@s.VUnit, k), @s.Silent)
        @s.Print(n) => {
          @utils.require(
            !statements,
            "print is a statement in the expression/statement language",
          )
          (Return(@s.VUnit, k), @s.Printed(n))
        }
        @s.Var(x) => (Return(@s.lookup(env, x), k), @s.Silent)
        @s.Fun(x, b) => (Return(@s.VFun(x, b, env), k), @s.Silent)
        @s.Rec(f, x, b) => (Return(@s.VRec(f, x, b, env), k), @s.Silent)
        @s.BinOp(a, o, b) =>
          (EvalExpr(a, env, push(BinOpLeft(o, b, env), k)), @s.Silent)
        @s.UnOp(o, e) => (EvalExpr(e, env, push(Unary(o), k)), @s.Silent)
        @s.App(f, a) => (EvalExpr(f, env, push(AppFun(a, env), k)), @s.Silent)
        @s.If(c, t, e) =>
          (EvalExpr(c, env, push(IfExpr(t, e, env), k)), @s.Silent)
        @s.Block(s, e) => {
          @utils.require(
            statements, "blocks belong to the expression/statement machine",
          )
          (EvalStmt(s, env, push(ScopeOut(e), k)), @s.Silent)
        }
      }
    Return(v, frames) => {
      @utils.require(!frames.is_empty(), "empty continuation")
      let k = frames[1:].to_owned()
      let next = match frames[0] {
        Init => Return(v, push(Init, k))
        BinOpLeft(o, b, env) => EvalExpr(b, env, push(BinOpRight(v, o), k))
        BinOpRight(a, o) => Return(@s.value_binop(o, a, v), k)
        Unary(o) => Return(@s.value_unop(o, v), k)
        AppFun(a, env) => EvalExpr(a, env, push(AppArg(v), k))
        AppArg(f) => {
          let (b, env) = @s.apply(f, v)
          EvalExpr(b, env, k)
        }
        IfExpr(t, e, env) =>
          EvalExpr(if @s.boolean("if", v) { t } else { e }, env, k)
        LetFrame(x, env) => Done(@s.bind(env, x, v), k)
        IfStmtFrame(s, env) =>
          if @s.boolean("if", v) {
            EvalStmt(s, env, k)
          } else {
            Done(env, k)
          }
        WhileFrame(c, s, env) =>
          if @s.boolean("while", v) {
            EvalStmt(s, env, push(SeqFrame(@s.While(c, s)), k))
          } else {
            Done(env, k)
          }
        _ => @utils.error("internal CEK error: statement frame during return")
      }
      (next, @s.Silent)
    }
    EvalStmt(s, env, k) =>
      match s {
        @s.Skip => (Done(env, k), @s.Silent)
        @s.PrintStmt(n) => (Done(env, k), @s.Printed(n))
        @s.Seq(a, b) => (EvalStmt(a, env, push(SeqFrame(b), k)), @s.Silent)
        @s.Let(x, e) => (EvalExpr(e, env, push(LetFrame(x, env), k)), @s.Silent)
        @s.IfStmt(c, s) =>
          (EvalExpr(c, env, push(IfStmtFrame(s, env), k)), @s.Silent)
        @s.While(c, s) =>
          (EvalExpr(c, env, push(WhileFrame(c, s, env), k)), @s.Silent)
      }
    Done(env, frames) => {
      @utils.require(!frames.is_empty(), "empty continuation")
      let k = frames[1:].to_owned()
      (
        match frames[0] {
          SeqFrame(s) => EvalStmt(s, env, k)
          ScopeOut(e) => EvalExpr(e, env, k)
          Init => Done(env, push(Init, k))
          _ =>
            @utils.error(
              "internal CEK error: unexpected frame after statement completion",
            )
        },
        @s.Silent,
      )
    }
  }
}

///|
fn make(code : @s.Expr, input : @s.Inputs) -> State {
  { code, state: EvalExpr(@s.apply_inputs(code, input.values), [], [Init]), }
}

///|
fn step(
  s : State,
  _input : Rinput,
  statements : Bool,
) -> @definition.Transition[State, Routput, Foutput] raise @utils.Fault {
  match s.state {
    Return(v, k) if k.length() == 1 =>
      match k[0] {
        Init => @definition.Halt(@s.as_nat(v))
        _ => {
          let (state, out) = transition(s.state, statements)
          @definition.Continue({ ..s, state, }, out)
        }
      }
    _ => {
      let (state, out) = transition(s.state, statements)
      @definition.Continue({ ..s, state, }, out)
    }
  }
}

///|
fn encode(s : State, statements : Bool) -> Json {
  @utils.object([
    ("code", @s.expr_json(s.code)),
    ("state", phase_json(s.state, statements)),
  ])
}

///|
fn decode(j : Json, statements : Bool) -> State raise @utils.Fault {
  {
    code: @s.decode_expr(@utils.field(j, "code")),
    state: decode_phase(@utils.field(j, "state"), statements),
  }
}

///|
fn render(s : State, statements : Bool) -> Array[@definition.Block] {
  fn env_from(k : Array[Frame]) -> @s.Env {
    for f in k {
      match f {
        BinOpLeft(_, _, e)
        | AppFun(_, e)
        | IfExpr(_, _, e)
        | LetFrame(_, e)
        | IfStmtFrame(_, e)
        | WhileFrame(_, _, e) => return e
        _ => ()
      }
    }
    []
  }
  let (mode, focus, env, k) = match s.state {
    EvalExpr(e, env, k) =>
      (if statements { "eval-expr" } else { "eval" }, @s.print(e), env, k)
    EvalStmt(s, env, k) => ("eval-stmt", @s.print_stmt(s), env, k)
    Return(v, k) => ("return", @s.value_text(v), env_from(k), k)
    Done(env, k) => ("done", "scope complete", env, k)
  }
  [
    @definition.titled("code", @s.print(s.code)),
    @definition.text_table("state", ["field", "value"], [
      ["mode", mode],
      ["focus", focus],
    ]),
    @s.render_env(env),
    @definition.text_table(
      "kont",
      ["depth", "frame"],
      k.mapi((i, f) => [i.to_string(), frame_json(f, statements).stringify()]),
    ),
  ]
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
    step(state, input, false)
  })
}

///|
pub fn statement_model() -> @definition.Model[
  @s.StatementCode,
  Ainput,
  Rinput,
  Routput,
  Foutput,
  StatementState,
] {
  @definition.Model(
    make=(code, input) => { value: make(code.expression, input), },
    step=(state, input) => {
      match step(state.value, input, true) {
        @definition.Continue(next, output) =>
          @definition.Continue({ value: next, }, output)
        @definition.Halt(output) => @definition.Halt(output)
      }
    },
  )
}
