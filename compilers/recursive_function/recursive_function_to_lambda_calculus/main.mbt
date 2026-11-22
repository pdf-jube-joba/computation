///|
fn abstractions(
  vars : Array[@target.Variable],
  body : @target.Term,
) -> @target.Term {
  let mut term = body
  for v in vars.rev() {
    term = @target.Abs(v, term)
  }
  term
}

///|
fn applications(
  first : @target.Term,
  rest : Array[@target.Term],
) -> @target.Term {
  let mut term = first
  for argument in rest {
    term = @target.App(term, argument)
  }
  term
}

///|
fn variables(
  next : Ref[Int32],
  prefix : String,
  count : Int32,
) -> Array[@target.Variable] {
  Array::makei(count, i => @target.fresh(next, prefix + i.to_string()))
}

///|
fn church(next : Ref[Int32], n : Int32) -> @target.Term {
  let s = @target.fresh(next, "s")
  let z = @target.fresh(next, "z")
  let mut body = @target.Var(z)
  for _ in 0..<n {
    body = @target.App(@target.Var(s), body)
  }
  abstractions([s, z], body)
}

///|
fn fixpoint(next : Ref[Int32]) -> @target.Term {
  let f = @target.fresh(next, "x")
  let x = @target.fresh(next, "f")
  let inner = @target.Abs(
    x,
    @target.App(@target.Var(f), @target.App(@target.Var(x), @target.Var(x))),
  )
  @target.Abs(f, @target.App(inner, @target.freshen_bound(inner, next)))
}

///|
fn boolean(next : Ref[Int32], value : Bool) -> @target.Term {
  let x = @target.fresh(next, "x")
  let y = @target.fresh(next, "y")
  abstractions([x, y], @target.Var(if value { x } else { y }))
}

///|
fn successor(next : Ref[Int32]) -> @target.Term {
  let x = @target.fresh(next, "x_succ")
  let y = @target.fresh(next, "y_succ")
  let z = @target.fresh(next, "z_succ")
  abstractions(
    [x, y, z],
    @target.App(
      @target.Var(y),
      applications(@target.Var(x), [@target.Var(y), @target.Var(z)]),
    ),
  )
}

///|
fn predecessor(next : Ref[Int32]) -> @target.Term {
  let x = @target.fresh(next, "x_pred")
  let y = @target.fresh(next, "y_pred")
  let z = @target.fresh(next, "z_pred")
  let p = @target.fresh(next, "p_pred")
  let q = @target.fresh(next, "q_pred")
  let ignored = @target.fresh(next, "v_pred")
  let identity = @target.fresh(next, "v_pred")
  let m = abstractions(
    [p, q],
    @target.App(@target.Var(q), @target.App(@target.Var(p), @target.Var(y))),
  )
  let n = @target.Abs(ignored, @target.Var(z))
  let identity = @target.Abs(identity, @target.Var(identity))
  abstractions([x, y, z], applications(@target.Var(x), [m, n, identity]))
}

///|
fn is_zero(next : Ref[Int32]) -> @target.Term {
  let n = @target.fresh(next, "n")
  let ignored = @target.fresh(next, "_")
  let falsity = boolean(next, false)
  let truth = boolean(next, true)
  @target.Abs(
    n,
    applications(@target.Var(n), [@target.Abs(ignored, falsity), truth]),
  )
}

///|
fn compile_function(
  next : Ref[Int32],
  f : @source.Func,
) -> @target.Term raise @utils.Fault {
  match f {
    @source.Zero => church(next, 0)
    @source.Succ => successor(next)
    @source.Proj(n, i) => {
      let vs = variables(next, "x", n)
      abstractions(vs, @target.Var(vs[i]))
    }
    @source.Comp(outer, inner) => {
      let outer = compile_function(next, outer)
      let inner_terms = inner.map(f => compile_function(next, f))
      let vs = variables(next, "x", @source.arity(inner[0]))
      let args = vs.map(v => @target.Var(v))
      abstractions(
        vs,
        applications(outer, inner_terms.map(t => applications(t, args))),
      )
    }
    @source.Prim(zero, succ) => {
      let zero_term = compile_function(next, zero)
      let succ_term = compile_function(next, succ)
      let vs = variables(next, "x", @source.arity(zero) + 1)
      let this = @target.fresh(next, "THIS")
      let first = @target.Var(vs[0])
      let rest = vs[1:].to_owned().map(v => @target.Var(v))
      let zero_test = @target.App(is_zero(next), first)
      let base = applications(zero_term, rest)
      let pred = @target.App(predecessor(next), first)
      let recursive_call = applications(@target.Var(this), [pred] + rest)
      let pred_copy = @target.freshen_bound(pred, next)
      let successor = applications(
        succ_term,
        [recursive_call, pred_copy] + rest,
      )
      let body = abstractions(
        [this] + vs,
        applications(zero_test, [base, successor]),
      )
      @target.App(fixpoint(next), body)
    }
    @source.Mu(predicate) => {
      let predicate_term = compile_function(next, predicate)
      let vs = variables(next, "v", @source.arity(predicate))
      let inc = @target.fresh(next, "INC")
      let index = @target.Var(vs[0])
      let rest = vs[1:].to_owned().map(v => @target.Var(v))
      let value = applications(predicate_term, vs.map(v => @target.Var(v)))
      let zero_test = @target.App(is_zero(next), value)
      let next_index = @target.App(successor(next), index)
      let otherwise = applications(@target.Var(inc), [next_index] + rest)
      let body = abstractions(
        [inc] + vs,
        applications(zero_test, [index, otherwise]),
      )
      let search = @target.App(fixpoint(next), body)
      let zero = church(next, 0)
      abstractions(vs[1:].to_owned(), applications(search, [zero] + rest))
    }
  }
}

///|
fn decode_number(t : @target.Term) -> @utils.Nat raise @utils.Fault {
  match t {
    @target.Abs(s, @target.Abs(z, body)) if s.id != z.id => {
      let mut count = @utils.zero()
      let mut current = body
      while true {
        match current {
          @target.Var(v) if v.id == z.id => return count
          @target.App(@target.Var(v), rest) if v.id == s.id => {
            count += @utils.one()
            current = rest
          }
          _ =>
            @utils.error(
              "failed to decode Church numeral: " + @target.print_code(t),
            )
        }
      } nobreak {
        @utils.error(
          "failed to decode Church numeral: " + @target.print_code(t),
        )
      }
    }
    _ =>
      @utils.error("failed to decode Church numeral: " + @target.print_code(t))
  }
}

///|
pub fn compiler() -> @definition.Compiler[
  @source.Code,
  @source.Ainput,
  @source.Rinput,
  @source.Routput,
  @source.Foutput,
  @target.Code,
  @target.Ainput,
  @target.Rinput,
  @target.Routput,
  @target.Foutput,
] {
  @definition.Compiler(
    compile_code=f => {
      @source.validate(f)
      compile_function({ val: 0, }, f)
    },
    encode_ainput=input => {
      let next : Ref[Int32] = { val: 0, }
      { values: input.values.map(n => church(next, @utils.index(n))), }
    },
    encode_rinput=_ => 0,
    decode_routput=output => output,
    decode_foutput=decode_number,
  )
}
