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
priv struct State {
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
) -> (Phase, String) raise @base.Fault {
  match phase {
    EvalExpr(e, env, k) =>
      match e {
        @s.Nat(n) => (Return(@s.VNat(n), k), "")
        @s.Bool(b) => (Return(@s.VBool(b), k), "")
        @s.Unit => (Return(@s.VUnit, k), "")
        @s.Print(n) => {
          @base.require(
            !statements,
            "print is a statement in the expression/statement language",
          )
          (Return(@s.VUnit, k), n.to_string())
        }
        @s.Var(x) => (Return(@s.lookup(env, x), k), "")
        @s.Fun(x, b) => (Return(@s.VFun(x, b, env), k), "")
        @s.Rec(f, x, b) => (Return(@s.VRec(f, x, b, env), k), "")
        @s.BinOp(a, o, b) =>
          (EvalExpr(a, env, push(BinOpLeft(o, b, env), k)), "")
        @s.UnOp(o, e) => (EvalExpr(e, env, push(Unary(o), k)), "")
        @s.App(f, a) => (EvalExpr(f, env, push(AppFun(a, env), k)), "")
        @s.If(c, t, e) => (EvalExpr(c, env, push(IfExpr(t, e, env), k)), "")
        @s.Block(s, e) => {
          @base.require(
            statements, "blocks belong to the expression/statement machine",
          )
          (EvalStmt(s, env, push(ScopeOut(e), k)), "")
        }
      }
    Return(v, frames) => {
      @base.require(!frames.is_empty(), "empty continuation")
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
        _ => @base.error("internal CEK error: statement frame during return")
      }
      (next, "")
    }
    EvalStmt(s, env, k) =>
      match s {
        @s.Skip => (Done(env, k), "")
        @s.PrintStmt(n) => (Done(env, k), n.to_string())
        @s.Seq(a, b) => (EvalStmt(a, env, push(SeqFrame(b), k)), "")
        @s.Let(x, e) => (EvalExpr(e, env, push(LetFrame(x, env), k)), "")
        @s.IfStmt(c, s) => (EvalExpr(c, env, push(IfStmtFrame(s, env), k)), "")
        @s.While(c, s) => (EvalExpr(c, env, push(WhileFrame(c, s, env), k)), "")
      }
    Done(env, frames) => {
      @base.require(!frames.is_empty(), "empty continuation")
      let k = frames[1:].to_owned()
      (
        match frames[0] {
          SeqFrame(s) => EvalStmt(s, env, k)
          ScopeOut(e) => EvalExpr(e, env, k)
          Init => Done(env, push(Init, k))
          _ =>
            @base.error(
              "internal CEK error: unexpected frame after statement completion",
            )
        },
        "",
      )
    }
  }
}

///|
fn make(
  code : String,
  input : String,
  statements : Bool,
) -> State raise @base.Fault {
  let code = @s.parse(code, statements~)
  {
    code,
    state: EvalExpr(@s.apply_inputs(code, @s.inputs(input)), [], [Init]),
  }
}

///|
fn step(
  s : State,
  input : String,
  statements : Bool,
) -> @runtime.Transition[State] raise @base.Fault {
  @base.unit_input(input)
  match s.state {
    Return(v, k) if k.length() == 1 =>
      match k[0] {
        Init => @runtime.Halt(@s.as_nat(v).to_string())
        _ => {
          let (state, out) = transition(s.state, statements)
          @runtime.Continue({ ..s, state, }, out)
        }
      }
    _ => {
      let (state, out) = transition(s.state, statements)
      @runtime.Continue({ ..s, state, }, out)
    }
  }
}

///|
fn encode(s : State, statements : Bool) -> Json {
  @base.object([
    ("code", @s.expr_json(s.code)),
    ("state", phase_json(s.state, statements)),
  ])
}

///|
fn decode(j : Json, statements : Bool) -> State raise @base.Fault {
  {
    code: @s.decode_expr(@base.field(j, "code")),
    state: decode_phase(@base.field(j, "state"), statements),
  }
}

///|
fn render(s : State, statements : Bool) -> Json {
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
  @base.jarray([
    @base.titled("code", @s.print(s.code)),
    @base.table("state", ["field", "value"], [["mode", mode], ["focus", focus]]),
    @s.render_env(env),
    @base.table(
      "kont",
      ["depth", "frame"],
      k.mapi((i, f) => [i.to_string(), frame_json(f, statements).stringify()]),
    ),
  ])
}

///|
pub fn model(statements~ : Bool = false) -> @runtime.Model {
  @runtime.model(
    (c, i) => make(c, i, statements),
    (s, i) => step(s, i, statements),
    s => encode(s, statements),
    j => decode(j, statements),
    s => render(s, statements),
  )
}

///|
pub fn statement_model() -> @runtime.Model {
  model(statements=true)
}
