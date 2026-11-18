///|
priv struct Builder {
  functions : Array[@ir.FunctionDef]
  names : @names.Supply
}

///|
fn create() -> Builder {
  { functions: [], names: @names.supply(reserved=["main"]), }
}

///|
fn register(c : Builder, f : @ir.FunctionDef) -> @ir.FunctionDef {
  for old in c.functions {
    if old.blocks == f.blocks {
      return old
    }
  }
  let f = { ..f, name: c.names.fresh(f.name), }
  c.functions.push(f)
  f
}

///|
fn block(label : String, body : Array[@ir.Stmt]) -> @ir.Block {
  { label, body, }
}

///|
fn function_def(name : String, blocks : Array[@ir.Block]) -> @ir.FunctionDef {
  { name, blocks, }
}

///|
fn call(c : Builder, f : @ir.FunctionDef) -> @ir.Stmt {
  @ir.Call(register(c, f).name)
}

///|
fn put(sign : String) -> @ir.Stmt {
  @ir.Assign(@s.PutHead, @s.Const(sign))
}

///|
fn store(name : String, value : @s.Rvalue) -> @ir.Stmt {
  @ir.Assign(@s.PutVar(name), value)
}

///|
fn put_var(name : String) -> @ir.Stmt {
  @ir.Assign(@s.PutHead, @s.Var(name))
}

///|
fn head_is(sign : String) -> @s.Condition? {
  Some({ left: @s.Head, right: @s.Const(sign), })
}

///|
fn var_is(name : String, sign : String) -> @s.Condition? {
  Some({ left: @s.Var(name), right: @s.Const(sign), })
}

///|
fn swap() -> Array[@ir.Stmt] {
  [store("tmp", @s.Head), put_var("put"), store("put", @s.Var("tmp"))]
}

///|
fn scan_delimiters(c : Builder, n : Int32, right : Bool) -> @ir.Stmt {
  call(
    c,
    function_def(
      (if right { "move_right_till_x_" } else { "move_left_till_x_" }) +
      n.to_string(),
      Array::makei(n, i => {
        block("until_x_" + i.to_string(), [
          if right {
            @ir.Rt
          } else {
            @ir.Lt
          },
          @ir.Break(head_is("x")),
          @ir.Continue(None),
        ])
      }),
    ),
  )
}

///|
fn concat(c : Builder) -> @ir.FunctionDef {
  function_def("concat", [
    block("initially", [
      scan_delimiters(c, 2, true),
      store("put", @s.Const("x")),
      put("-"),
    ]),
    block(
      "loop",
      [@ir.Lt] + swap() + [@ir.Break(var_is("put", "x")), @ir.Continue(None)],
    ),
    block("finally", [scan_delimiters(c, 1, false)]),
  ])
}

///|
fn shift_left(sign : String) -> @ir.FunctionDef {
  function_def("shift_left_put_" + (if sign == "-" { "blank" } else { sign }), [
    block("initially", [store("put", @s.Head), put(sign), @ir.Lt]),
    block(
      "loop",
      swap() + [@ir.Return(var_is("put", "x")), @ir.Lt, @ir.Continue(None)],
    ),
  ])
}

///|
fn copy_to_end(c : Builder, n : Int32) -> @ir.FunctionDef {
  function_def("copy_to_end_" + n.to_string(), [
    block("initially", [
      scan_delimiters(c, n + 1, true),
      @ir.Rt,
      put("x"),
      scan_delimiters(c, n + 2, false),
    ]),
    block("main", [
      @ir.Rt,
      @ir.Break(head_is("x")),
      store("put", @s.Head),
      put("x"),
      scan_delimiters(c, n + 2, true),
      put_var("put"),
      @ir.Rt,
      put("x"),
      scan_delimiters(c, n + 2, false),
      put_var("put"),
      @ir.Continue(None),
    ]),
    block("finally", [scan_delimiters(c, 1, false)]),
  ])
}

///|
fn copy_n_times(c : Builder, n : Int32) -> @ir.FunctionDef {
  let cp = call(c, copy_to_end(c, 0))
  function_def(
    "copy_" + n.to_string(),
    Array::makei(n, i => {
      block("copy_head_" + i.to_string(), [cp, scan_delimiters(c, 1, true)])
    }) +
    Array::makei(n, i => {
      block("copy_tail_" + i.to_string(), [scan_delimiters(c, 1, false)])
    }),
  )
}

///|
fn swap_tuple(c : Builder) -> @ir.FunctionDef {
  function_def("swap_tuple", [
    block("initially", [
      store("put", @s.Const("x")),
      scan_delimiters(c, 2, true),
    ]),
    block(
      "loop_1",
      swap() + [@ir.Lt, @ir.Break(var_is("put", "x")), @ir.Continue(None)],
    ),
    block(
      "loop_2",
      swap() + [@ir.Lt, @ir.Break(var_is("put", "x")), @ir.Continue(None)],
    ),
    block(
      "loop_3",
      swap() + [@ir.Lt, @ir.Break(var_is("put", "x")), @ir.Continue(None)],
    ),
    block(
      "left_of_left",
      [@ir.Rt, @ir.Return(head_is("x"))] +
      swap() +
      [scan_delimiters(c, 3, true), @ir.Lt, @ir.Jump("loop_1", None)],
    ),
  ])
}

///|
fn rotate(c : Builder, n : Int32) -> @ir.FunctionDef {
  if n < 2 {
    return function_def("rotate_" + n.to_string(), [])
  }
  let sw = call(c, swap_tuple(c))
  function_def(
    "rotate_" + n.to_string(),
    Array::makei(n - 1, i => {
      block("rot_swap_" + i.to_string(), [sw, scan_delimiters(c, 1, true)])
    }) +
    [block("return_to_init", [scan_delimiters(c, n - 1, false)])],
  )
}

///|
fn projection_init(c : Builder, n : Int32, index : Int32) -> @ir.FunctionDef {
  function_def(
    "aux_proj_" + n.to_string() + "_" + index.to_string(),
    [block("initial", [@ir.Rt])] +
    Array::makei(index, i => {
      block("flat_" + i.to_string(), [
        put("-"),
        @ir.Rt,
        @ir.Break(head_is("-")),
        @ir.Continue(None),
      ])
    }) +
    [block("move_x", [scan_delimiters(c, 1, true)])] +
    Array::makei(n - index - 1, i => {
      block("flat_" + (n - 1 - i).to_string(), [
        put("-"),
        @ir.Lt,
        @ir.Break(head_is("-")),
        @ir.Continue(None),
      ])
    }) +
    [block("push_end_x", [put("x")])],
  )
}

///|
fn projection(c : Builder, n : Int32, index : Int32) -> @ir.FunctionDef {
  function_def("proj_" + n.to_string() + "_" + index.to_string(), [
    block("initial", [call(c, projection_init(c, n, index))]),
    block("loop2_init", [@ir.Lt, @ir.Break(head_is("-")), @ir.Continue(None)]),
    block("check_left", [@ir.Lt, @ir.Return(head_is("x")), @ir.Rt]),
    block("initial_B", [
      scan_delimiters(c, 1, true),
      put("-"),
      store("put", @s.Const("x")),
    ]),
    block(
      "shift_to_left_until_B",
      [@ir.Lt] + swap() + [@ir.Break(var_is("put", "-")), @ir.Continue(None)],
    ),
    block("finally_B", [@ir.Lt, @ir.Jump("check_left", None)]),
  ])
}

///|
fn composition(
  c : Builder,
  inner : Array[@ir.FunctionDef],
  outer : @ir.FunctionDef,
) -> @ir.FunctionDef {
  let n = inner.length()
  let rot = call(c, rotate(c, n))
  let blocks = [block("initially", [call(c, copy_n_times(c, n - 1))])]
  for i, f in inner {
    let body = [
      scan_delimiters(c, n - 1, true),
      call(c, f),
      scan_delimiters(c, n - 1, false),
    ]
    if i < n - 1 {
      body.push(rot)
    }
    blocks.push(block("call_" + i.to_string() + "_" + f.name, body))
  }
  let glue = call(c, concat(c))
  for i in 0..<(n - 1) {
    blocks.push(block("concat_" + i.to_string(), [glue]))
  }
  blocks.push(block("outer", [call(c, outer)]))
  function_def(
    "comp_" + outer.name + "_" + @base.join(inner.map(f => f.name), "_"),
    blocks,
  )
}

///|
/// Insert the marker x,l,x while preserving the two displaced symbols.
fn insert_marker(c : Builder) -> @ir.FunctionDef {
  function_def("insert_sig", [
    block("initially", [
      store("tmp1", @s.Const("l")),
      store("tmp2", @s.Const("x")),
      @ir.Rt,
    ]),
    block("main_loop", [
      @ir.Break(head_is("x")),
      store("tmp3", @s.Head),
      put_var("tmp1"),
      store("tmp1", @s.Var("tmp2")),
      store("tmp2", @s.Var("tmp3")),
      @ir.Rt,
      @ir.Continue(None),
    ]),
    block("finally", [
      put_var("tmp1"),
      @ir.Rt,
      put_var("tmp2"),
      @ir.Rt,
      put("x"),
      scan_delimiters(c, 1, false),
    ]),
  ])
}

///|
fn predecessor_tuple(c : Builder) -> @ir.FunctionDef {
  function_def("pred_tuple", [
    block("initially", [
      scan_delimiters(c, 1, true),
      call(c, shift_left("-")),
      put("x"),
      @ir.Rt,
      @ir.Break(head_is("x")),
      put("-"),
    ]),
    block("finally", [scan_delimiters(c, 1, false)]),
  ])
}

///|
fn expand_argument(c : Builder) -> @ir.FunctionDef {
  let pred = call(c, predecessor_tuple(c))
  let cp = call(c, copy_n_times(c, 1))
  function_def("expand_arg", [
    block("initially", [call(c, insert_marker(c))]),
    block("check_non_zero", [
      @ir.Rt,
      @ir.Rt,
      @ir.Jump("pred_if_non_zero", head_is("l")),
      @ir.Jump("pred_if_zero", None),
    ]),
    block("pred_if_non_zero", [
      @ir.Lt,
      @ir.Lt,
      pred,
      cp,
      scan_delimiters(c, 1, true),
      @ir.Jump("check_non_zero", None),
    ]),
    block("pred_if_zero", [@ir.Lt, @ir.Lt, pred]),
  ])
}

///|
fn primitive(
  c : Builder,
  zero : @ir.FunctionDef,
  succ : @ir.FunctionDef,
) -> @ir.FunctionDef {
  let sw = call(c, swap_tuple(c))
  let glue = call(c, concat(c))
  let shift = call(c, shift_left("-"))
  function_def("prim_" + zero.name + "_" + succ.name, [
    block("call_zero", [call(c, expand_argument(c)), call(c, zero)]),
    block("loop", [
      scan_delimiters(c, 1, false),
      @ir.Rt,
      @ir.Break(head_is("l")),
      @ir.Lt,
      sw,
      glue,
      call(c, succ),
      @ir.Continue(None),
    ]),
    block("finally", [
      @ir.Rt,
      put("-"),
      scan_delimiters(c, 1, true),
      shift,
      put("x"),
      scan_delimiters(c, 1, true),
      shift,
      put("x"),
    ]),
  ])
}

///|
fn delete_tuple(c : Builder) -> @ir.FunctionDef {
  function_def("delete", [
    block("loop", [
      @ir.Rt,
      @ir.Break(head_is("x")),
      put("-"),
      @ir.Continue(None),
    ]),
    block("finally", [put("-"), scan_delimiters(c, 1, false)]),
  ])
}

///|
fn minimization(c : Builder, f : @ir.FunctionDef) -> @ir.FunctionDef {
  let c0 = call(c, copy_to_end(c, 0))
  let c2 = call(c, copy_to_end(c, 2))
  let glue = call(c, concat(c))
  let sw = call(c, swap_tuple(c))
  let del = call(c, delete_tuple(c))
  let fcall = call(c, f)
  function_def("mu_" + f.name, [
    block("initially", [
      scan_delimiters(c, 1, true),
      @ir.Rt,
      put("-"),
      @ir.Rt,
      put("x"),
      scan_delimiters(c, 1, false),
    ]),
    block("generate_and_call", [
      c0,
      scan_delimiters(c, 1, false),
      c2,
      scan_delimiters(c, 2, true),
      glue,
      fcall,
      @ir.Rt,
      @ir.Rt,
      @ir.Jump("is_zero", head_is("x")),
      scan_delimiters(c, 1, true),
    ]),
    block("is_non_zero", [
      put("-"),
      @ir.Lt,
      @ir.Break(head_is("x")),
      @ir.Continue(None),
    ]),
    block("next_num", [
      put("l"),
      @ir.Rt,
      put("x"),
      scan_delimiters(c, 1, false),
      @ir.Jump("generate_and_call", None),
    ]),
    block("is_zero", [
      put("-"),
      scan_delimiters(c, 3, false),
      sw,
      scan_delimiters(c, 1, true),
      del,
      scan_delimiters(c, 1, false),
    ]),
  ])
}

///|
fn compile_function(c : Builder, f : @source.Func) -> @ir.FunctionDef {
  match f {
    @source.Zero =>
      function_def("zero_const_function", [
        block("entry", [@ir.Rt, put("-"), @ir.Rt, put("x"), @ir.Lt, @ir.Lt]),
      ])
    @source.Succ =>
      function_def("succ_function", [
        block("entry", [
          scan_delimiters(c, 1, true),
          put("l"),
          @ir.Rt,
          put("x"),
          scan_delimiters(c, 1, false),
        ]),
      ])
    @source.Proj(n, i) => projection(c, n, i)
    @source.Comp(outer, inners) => {
      let outer = compile_function(c, outer)
      let inner = inners.map(f => compile_function(c, f))
      composition(c, inner, outer)
    }
    @source.Prim(zero, succ) => {
      let zero = compile_function(c, zero)
      let succ = compile_function(c, succ)
      primitive(c, zero, succ)
    }
    @source.Mu(f) => minimization(c, compile_function(c, f))
  }
}

///|
pub fn compile(f : @source.Func) -> @ir.Code raise @base.Fault {
  @source.validate(f)
  let c = create()
  let f = compile_function(c, f)
  let main = function_def("main", [block("main", [call(c, f)])])
  { alphabet: ["-", "l", "x"], functions: [main] + c.functions, }
}

///|
pub fn compiler() -> @runtime.Compiler {
  {
    compile_code: text => @ir.print_code(compile(@source.parse_code(text))),
    encode_ainput: text => @encoding.encode_tuple(@source.parse_input(text)),
    encode_rinput: @encoding.unit_text,
    decode_routput: @encoding.unit_text,
    decode_foutput: @encoding.decode_number,
  }
}
