open Runtime_types
open Store

let expect_number = function
  | Num n -> n
  | v -> failwith ("expected a number, found " ^ Runtime_value.print v)

let rec eval_place s = function
  | Var v -> Utils_base.Assoc.lookup v s.env ("unknown variable: " ^ v)
  | Deref v -> (
      match eval_value s v with
      | Location loc -> loc
      | Null_ptr -> failwith "cannot dereference #null_ptr"
      | Num _ -> failwith "dereference expects a location value")

and eval_value s = function
  | Number n -> Num n
  | NullPtr -> Null_ptr
  | BinOp (a, op, b) ->
      let a = expect_number (eval_value s a)
      and b = expect_number (eval_value s b) in
      Num ((if op = Add then Int64.add else Int64.sub) a b)
  | Load p -> store_lookup s (eval_place s p)
  | Addr p -> Location (eval_place s p)
