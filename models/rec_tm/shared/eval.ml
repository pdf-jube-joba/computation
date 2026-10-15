open Syntax
open Environment

let eval env tape : rvalue -> T.sign = function
  | Var s ->
      Option.value ~default:None (List.assoc_opt (variable env s) env.values)
  | Head -> tape.Utils_data.Tape.head
  | Const s -> s

let eval_cond env tape = function
  | None -> true
  | Some c -> eval env tape c.left = eval env tape c.right

let assign env tape (dst : lvalue) value =
  match dst with
  | Var s ->
      ( {
          env with
          values = Utils_base.Assoc.set_sorted (variable env s) value env.values;
        },
        tape )
  | Head -> (env, T.head_write tape value)
