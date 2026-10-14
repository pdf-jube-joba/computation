type t = Syntax.expr

let print = Print.print
let print_stmt = Print.print_stmt
let parse = Utils_parsing.Parser.parse (Parser.expression true)
