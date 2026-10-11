(* parse.rs was empty. These parsers implement the checked-in continuation.pest
   lambda productions; grab/delim are dormant grammars without a Rust evaluator. *)
let lam_nat = Lambda.parse_nat_code
let lam_ext = Lambda.parse_code
let ctrl = Lambda.parse_ctrl_code
let parse_code = lam_ext
