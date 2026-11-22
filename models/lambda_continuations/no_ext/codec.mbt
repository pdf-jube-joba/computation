///|
fn term_json(t : Term) -> Json {
  match t {
    Var(v0) => @utils.tag("Var", [@utils.jstr(v0)])
    Lam(v0, v1) => @utils.tag("Lam", [@utils.jstr(v0), term_json(v1)])
    App(v0, v1) => @utils.tag("App", [term_json(v0), term_json(v1)])
    Zero => @utils.tag("Zero", [])
    Succ(v0) => @utils.tag("Succ", [term_json(v0)])
    Pred(v0) => @utils.tag("Pred", [term_json(v0)])
    IfZ(v0, v1, v2) =>
      @utils.tag("IfZ", [term_json(v0), term_json(v1), term_json(v2)])
    Let(v0, v1, v2) =>
      @utils.tag("Let", [@utils.jstr(v0), term_json(v1), term_json(v2)])
    Rec(v0, v1, v2) =>
      @utils.tag("Rec", [@utils.jstr(v0), @utils.jstr(v1), term_json(v2)])
    Abort(v0) => @utils.tag("Abort", [term_json(v0)])
    Control(v0) => @utils.tag("Control", [term_json(v0)])
  }
}

///|
fn decode_term(j : Json) -> Term raise @utils.Fault {
  match @utils.tag_name(j) {
    "Var" => Var(@utils.identifier(@utils.string(@utils.tag_arg(j, 0))))
    "Lam" =>
      Lam(
        @utils.identifier(@utils.string(@utils.tag_arg(j, 0))),
        decode_term(@utils.tag_arg(j, 1)),
      )
    "App" =>
      App(decode_term(@utils.tag_arg(j, 0)), decode_term(@utils.tag_arg(j, 1)))
    "Zero" => Zero
    "Succ" => Succ(decode_term(@utils.tag_arg(j, 0)))
    "Pred" => Pred(decode_term(@utils.tag_arg(j, 0)))
    "IfZ" =>
      IfZ(
        decode_term(@utils.tag_arg(j, 0)),
        decode_term(@utils.tag_arg(j, 1)),
        decode_term(@utils.tag_arg(j, 2)),
      )
    "Let" =>
      Let(
        @utils.identifier(@utils.string(@utils.tag_arg(j, 0))),
        decode_term(@utils.tag_arg(j, 1)),
        decode_term(@utils.tag_arg(j, 2)),
      )
    "Rec" =>
      Rec(
        @utils.identifier(@utils.string(@utils.tag_arg(j, 0))),
        @utils.identifier(@utils.string(@utils.tag_arg(j, 1))),
        decode_term(@utils.tag_arg(j, 2)),
      )
    "Abort" => Abort(decode_term(@utils.tag_arg(j, 0)))
    "Control" => Control(decode_term(@utils.tag_arg(j, 0)))
    _ => @utils.error("Invalid lambda continuation term")
  }
}
