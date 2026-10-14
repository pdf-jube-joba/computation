type t = Syntax.expr

let print = Print.print

let parse text =
  Result.bind (Parser.parse text) (fun expr ->
      Result.map
        (fun () -> Substitution.normalize expr)
        (Syntax.validate_direct expr))
