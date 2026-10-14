module N = Utils_base.Number
module L = Utils_parsing.Parser
module Names = Utils_names.Identifier.Set

(* Abort/Control extend the same structure, so the implemented variable
   operations can be shared with Ctrl. Its reduction is deliberately unsupported. *)
type term =
  | Var of Utils_names.Identifier.t
  | Lam of Utils_names.Identifier.t * term
  | App of term * term
  | Zero
  | Succ of term
  | Pred of term
  | IfZ of term * term * term
  | Let of Utils_names.Identifier.t * term * term
  | Rec of Utils_names.Identifier.t * Utils_names.Identifier.t * term
  | Abort of term
  | Control of term
[@@deriving yojson]

type value = Fun of Utils_names.Identifier.t * term | Num of N.t
[@@deriving yojson]

let rec num_to_exp n =
  if N.is_zero n then Zero else Succ (num_to_exp (N.pred n))

let rec exp_to_num = function
  | Zero -> Some N.zero
  | Succ n -> Option.map N.succ (exp_to_num n)
  | _ -> None

let is_value = function
  | Lam (x, b) -> Some (Fun (x, b))
  | e -> Option.map (fun n -> Num n) (exp_to_num e)

let rec free_variables = function
  | Var x -> Names.singleton x
  | Zero -> Names.empty
  | Lam (x, b) -> Names.remove x (free_variables b)
  | App (a, b) -> Names.union (free_variables a) (free_variables b)
  | Succ e | Pred e | Abort e | Control e -> free_variables e
  | IfZ (c, t, f) ->
      Names.union (free_variables c)
        (Names.union (free_variables t) (free_variables f))
  | Let (x, e, b) ->
      Names.union (free_variables e) (Names.remove x (free_variables b))
  | Rec (f, x, b) -> Names.remove f (Names.remove x (free_variables b))

let rec bound_variables = function
  | Var _ | Zero -> Names.empty
  | Lam (x, b) -> Names.add x (bound_variables b)
  | App (a, b) -> Names.union (bound_variables a) (bound_variables b)
  | Succ e | Pred e | Abort e | Control e -> bound_variables e
  | IfZ (c, t, f) ->
      Names.union (bound_variables c)
        (Names.union (bound_variables t) (bound_variables f))
  | Let (x, e, b) ->
      Names.add x (Names.union (bound_variables e) (bound_variables b))
  | Rec (f, x, b) -> Names.add f (Names.add x (bound_variables b))

module Substitution =
  Utils_names.Binding.Make
    (struct
      type t = term

      let variable name = Var name

      let view e =
        let open Utils_names.Binding in
        match e with
        | Var name -> Variable name
        | Lam (x, body) ->
            node ~binders:[ x ]
              [ ([ 0 ], body) ]
              (fun names children -> Lam (List.hd names, List.hd children))
        | Let (x, value, body) ->
            node ~binders:[ x ]
              [ ([], value); ([ 0 ], body) ]
              (fun names children ->
                Let (List.hd names, List.nth children 0, List.nth children 1))
        | Rec (f, x, body) ->
            node ~binders:[ f; x ]
              [ ([ 0; 1 ], body) ]
              (fun names children ->
                Rec (List.nth names 0, List.nth names 1, List.hd children))
        | App (a, b) ->
            node
              [ ([], a); ([], b) ]
              (fun _ children -> App (List.nth children 0, List.nth children 1))
        | IfZ (c, a, b) ->
            node
              [ ([], c); ([], a); ([], b) ]
              (fun _ children ->
                IfZ
                  (List.nth children 0, List.nth children 1, List.nth children 2))
        | Succ body ->
            node [ ([], body) ] (fun _ children -> Succ (List.hd children))
        | Pred body ->
            node [ ([], body) ] (fun _ children -> Pred (List.hd children))
        | Abort body ->
            node [ ([], body) ] (fun _ children -> Abort (List.hd children))
        | Control body ->
            node [ ([], body) ] (fun _ children -> Control (List.hd children))
        | Zero -> node [] (fun _ _ -> Zero)
    end)
    ()

let subst = Substitution.substitute

module Code = struct
  type t = term

  let rec pretty = function
    | Var x -> x
    | Lam (x, b) -> "fun " ^ x ^ " => " ^ pretty b
    | App (a, b) -> "(" ^ pretty a ^ " " ^ pretty b ^ ")"
    | Zero -> "0"
    | Succ e -> "S " ^ pretty e
    | Pred e -> "P " ^ pretty e
    | IfZ (c, t, f) ->
        "if " ^ pretty c ^ " then " ^ pretty t ^ " else " ^ pretty f
    | Let (x, e, b) -> "let " ^ x ^ " = " ^ pretty e ^ " in \n " ^ pretty b
    | Rec (f, x, b) -> "rec " ^ f ^ " " ^ x ^ " = " ^ pretty b
    | Abort e -> "abort " ^ pretty e
    | Control e -> "control " ^ pretty e

  let rec print = function
    | Var x -> x
    | Lam (x, b) -> "(fun " ^ x ^ " => " ^ print b ^ ")"
    | App (a, b) -> "@(" ^ print a ^ " " ^ print b ^ ")"
    | Zero -> "zero"
    | Succ e -> "(succ " ^ print e ^ ")"
    | Pred e -> "(pred " ^ print e ^ ")"
    | IfZ (c, t, f) ->
        "(ifz " ^ print c ^ " then " ^ print t ^ " else " ^ print f ^ ")"
    | Let (x, e, b) -> "(let " ^ x ^ " = " ^ print e ^ " in " ^ print b ^ ")"
    | Rec (f, x, b) -> "(rec " ^ f ^ " " ^ x ^ " = " ^ print b ^ ")"
    | Abort e -> "(abort " ^ print e ^ ")"
    | Control e -> "(control " ^ print e ^ ")"

  let variable p =
    let name = L.identifier p in
    if
      List.mem name
        [
          "fun";
          "zero";
          "succ";
          "pred";
          "ifz";
          "then";
          "else";
          "let";
          "in";
          "rec";
          "abort";
          "control";
        ]
    then L.error p "expected variable";
    name

  let rec read_term ~control p =
    match L.peek p with
    | L.Ident "fun" ->
        ignore (L.bump p);
        let x = variable p in
        L.expect p "=>";
        Lam (x, read_term ~control p)
    | L.Symbol '@' ->
        ignore (L.bump p);
        L.group p ~opening:'(' (fun p ->
            let a = read_term ~control p in
            let b = read_term ~control p in
            App (a, b))
    | L.Symbol '(' ->
        L.group p ~opening:'(' (fun p ->
            let a = read_term ~control p in
            if L.eof p then a
            else
              let b = read_term ~control p in
              App (a, b))
    | L.Ident "zero" | L.Number "0" ->
        ignore (L.bump p);
        Zero
    | L.Ident "succ" | L.Ident "S" ->
        ignore (L.bump p);
        Succ (read_term ~control p)
    | L.Ident "pred" | L.Ident "P" ->
        ignore (L.bump p);
        Pred (read_term ~control p)
    | L.Ident "ifz" | L.Ident "if" ->
        ignore (L.bump p);
        let c = read_term ~control p in
        L.expect_keyword p "then";
        let t = read_term ~control p in
        L.expect_keyword p "else";
        IfZ (c, t, read_term ~control p)
    | L.Ident "let" ->
        ignore (L.bump p);
        let x = variable p in
        L.expect_symbol p '=';
        let e = read_term ~control p in
        L.expect_keyword p "in";
        Let (x, e, read_term ~control p)
    | L.Ident "rec" ->
        ignore (L.bump p);
        let f = variable p in
        let x = variable p in
        L.expect_symbol p '=';
        Rec (f, x, read_term ~control p)
    | L.Ident "abort" when control ->
        ignore (L.bump p);
        Abort (read_term ~control p)
    | L.Ident "control" when control ->
        ignore (L.bump p);
        Control (read_term ~control p)
    | _ -> Var (variable p)

  let parse text =
    Result.map Substitution.normalize (L.parse (read_term ~control:false) text)
end

module Ctrl_code = struct
  include Code

  let parse text =
    Result.map Substitution.normalize (L.parse (read_term ~control:true) text)
end

module Nat_code = struct
  type t = term

  let print = Code.print

  let parse =
    (* The unextended grammar treats names such as zero and succ as variables. *)
    let rec read p =
      match L.peek p with
      | L.Ident "fun" -> (
          match
            L.attempt p (fun p ->
                ignore (L.bump p);
                let x = L.identifier p in
                L.expect p "=>";
                Lam (x, read p))
          with
          | Ok e -> e
          | Error _ -> Var (L.identifier p))
      | L.Symbol '@' ->
          ignore (L.bump p);
          L.group p ~opening:'(' (fun p ->
              let f = read p in
              let a = read p in
              App (f, a))
      | L.Symbol '(' -> L.group p ~opening:'(' read
      | _ -> Var (L.identifier p)
    in
    fun text -> Result.map Substitution.normalize (L.parse read text)
end

module Value = struct
  type t = value

  let to_term = function
    | Fun (x, body) -> Lam (x, body)
    | Num n -> num_to_exp n

  let parse text =
    Result.bind (Ctrl_code.parse text) (fun term ->
        match is_value term with
        | Some value -> Ok value
        | None -> Error "Expected a value")

  let print value = Code.pretty (to_term value)
end
