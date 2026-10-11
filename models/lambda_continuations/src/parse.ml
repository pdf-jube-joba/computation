(* parse.rs was empty. These parsers implement the checked-in continuation.pest
   lambda productions; grab/delim are dormant grammars without a Rust evaluator. *)
let lam_nat = Lambda.Nat_code.parse
let lam_ext = Lambda.Code.parse
let ctrl = Lambda.Ctrl_code.parse

module Code = Lambda.Code
