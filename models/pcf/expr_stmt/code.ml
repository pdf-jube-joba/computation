type t = Pcf_shared.Syntax.expr

let print = Pcf_shared.Print.print
let print_stmt = Pcf_shared.Print.print_stmt
let parse = Utils_parsing.Parser.parse (Pcf_shared.Parser.expression true)
