///|
fn term_json(t : Term) -> Json {
  match t {
    Var(v0) => @base.tag("Var", [@base.jstr(v0)])
    Lam(v0, v1) => @base.tag("Lam", [@base.jstr(v0), term_json(v1)])
    App(v0, v1) => @base.tag("App", [term_json(v0), term_json(v1)])
    Zero => @base.tag("Zero", [])
    Succ(v0) => @base.tag("Succ", [term_json(v0)])
    Pred(v0) => @base.tag("Pred", [term_json(v0)])
    IfZ(v0, v1, v2) =>
      @base.tag("IfZ", [term_json(v0), term_json(v1), term_json(v2)])
    Let(v0, v1, v2) =>
      @base.tag("Let", [@base.jstr(v0), term_json(v1), term_json(v2)])
    Rec(v0, v1, v2) =>
      @base.tag("Rec", [@base.jstr(v0), @base.jstr(v1), term_json(v2)])
    Abort(v0) => @base.tag("Abort", [term_json(v0)])
    Control(v0) => @base.tag("Control", [term_json(v0)])
  }
}

///|
fn decode_term(j : Json) -> Term raise @base.Fault {
  match @base.tag_name(j) {
    "Var" => Var(@base.identifier(@base.string(@base.tag_arg(j, 0))))
    "Lam" =>
      Lam(
        @base.identifier(@base.string(@base.tag_arg(j, 0))),
        decode_term(@base.tag_arg(j, 1)),
      )
    "App" =>
      App(decode_term(@base.tag_arg(j, 0)), decode_term(@base.tag_arg(j, 1)))
    "Zero" => Zero
    "Succ" => Succ(decode_term(@base.tag_arg(j, 0)))
    "Pred" => Pred(decode_term(@base.tag_arg(j, 0)))
    "IfZ" =>
      IfZ(
        decode_term(@base.tag_arg(j, 0)),
        decode_term(@base.tag_arg(j, 1)),
        decode_term(@base.tag_arg(j, 2)),
      )
    "Let" =>
      Let(
        @base.identifier(@base.string(@base.tag_arg(j, 0))),
        decode_term(@base.tag_arg(j, 1)),
        decode_term(@base.tag_arg(j, 2)),
      )
    "Rec" =>
      Rec(
        @base.identifier(@base.string(@base.tag_arg(j, 0))),
        @base.identifier(@base.string(@base.tag_arg(j, 1))),
        decode_term(@base.tag_arg(j, 2)),
      )
    "Abort" => Abort(decode_term(@base.tag_arg(j, 0)))
    "Control" => Control(decode_term(@base.tag_arg(j, 0)))
    _ => @base.error("Invalid lambda continuation term")
  }
}
