module N = Utils.Number
module L = Utils.Lexer
module Names = Utils.Identifier.Set

(* Abort/Control extend the same structure, so the implemented variable
   operations can be shared with Ctrl. Its reduction is deliberately unsupported. *)
type term =
  | Var of Utils.Identifier.t
  | Lam of Utils.Identifier.t * term
  | App of term * term
  | Zero
  | Succ of term
  | Pred of term
  | IfZ of term * term * term
  | Let of Utils.Identifier.t * term * term
  | Rec of Utils.Identifier.t * Utils.Identifier.t * term
  | Abort of term
  | Control of term
[@@deriving yojson]

type value = Fun of Utils.Identifier.t * term | Num of N.t [@@deriving yojson]

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

let all_names e = Names.union (free_variables e) (bound_variables e)

let rec rename e from into =
  match e with
  | Var x -> if x = from then Var into else e
  | Lam (x, _) when x = from -> e
  | Lam (x, b) -> Lam (x, rename b from into)
  | App (a, b) -> App (rename a from into, rename b from into)
  | Zero -> Zero
  | Succ e -> Succ (rename e from into)
  | Pred e -> Pred (rename e from into)
  | IfZ (c, t, f) ->
      IfZ (rename c from into, rename t from into, rename f from into)
  | Let (x, a, b) ->
      Let (x, rename a from into, if x = from then b else rename b from into)
  | Rec (f, x, _) when f = from || x = from -> e
  | Rec (f, x, b) -> Rec (f, x, rename b from into)
  | Abort e -> Abort (rename e from into)
  | Control e -> Control (rename e from into)

let rec subst e target replacement =
  let avoid =
    Names.union (all_names e) (all_names replacement) |> Names.add target
  in
  let binder x b =
    if Names.mem x (free_variables replacement) then
      let y = Utils.Identifier.fresh avoid in
      (y, rename b x y)
    else (x, b)
  in
  match e with
  | Var x -> if x = target then replacement else e
  | Lam (x, _) when x = target -> e
  | Lam (x, b) ->
      let x, b = binder x b in
      Lam (x, subst b target replacement)
  | App (a, b) -> App (subst a target replacement, subst b target replacement)
  | Zero -> Zero
  | Succ e -> Succ (subst e target replacement)
  | Pred e -> Pred (subst e target replacement)
  | IfZ (c, t, f) ->
      IfZ
        ( subst c target replacement,
          subst t target replacement,
          subst f target replacement )
  | Let (x, a, b) ->
      let a = subst a target replacement in
      if x = target then Let (x, a, b)
      else
        let x, b = binder x b in
        Let (x, a, subst b target replacement)
  | Rec (f, x, _) when f = target || x = target -> e
  | Rec (f, x, b) ->
      let f, b = binder f b in
      let used = Names.add f (Names.union avoid (all_names b)) in
      let x, b =
        if Names.mem x (free_variables replacement) then
          let y = Utils.Identifier.fresh used in
          (y, rename b x y)
        else (x, b)
      in
      Rec (f, x, subst b target replacement)
  | Abort e -> Abort (subst e target replacement)
  | Control e -> Control (subst e target replacement)

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
        ignore (L.next p);
        let x = variable p in
        L.expect p "=>";
        Lam (x, read_term ~control p)
    | L.Symbol '@' ->
        ignore (L.next p);
        L.expect_symbol p '(';
        let a = read_term ~control p in
        let b = read_term ~control p in
        L.expect_symbol p ')';
        App (a, b)
    | L.Symbol '(' ->
        ignore (L.next p);
        let a = read_term ~control p in
        let a =
          if L.peek p = L.Symbol ')' then a
          else
            let b = read_term ~control p in
            App (a, b)
        in
        L.expect_symbol p ')';
        a
    | L.Ident "zero" | L.Number "0" ->
        ignore (L.next p);
        Zero
    | L.Ident "succ" | L.Ident "S" ->
        ignore (L.next p);
        Succ (read_term ~control p)
    | L.Ident "pred" | L.Ident "P" ->
        ignore (L.next p);
        Pred (read_term ~control p)
    | L.Ident "ifz" | L.Ident "if" ->
        ignore (L.next p);
        let c = read_term ~control p in
        L.expect_keyword p "then";
        let t = read_term ~control p in
        L.expect_keyword p "else";
        IfZ (c, t, read_term ~control p)
    | L.Ident "let" ->
        ignore (L.next p);
        let x = variable p in
        L.expect_symbol p '=';
        let e = read_term ~control p in
        L.expect_keyword p "in";
        Let (x, e, read_term ~control p)
    | L.Ident "rec" ->
        ignore (L.next p);
        let f = variable p in
        let x = variable p in
        L.expect_symbol p '=';
        Rec (f, x, read_term ~control p)
    | L.Ident "abort" when control ->
        ignore (L.next p);
        Abort (read_term ~control p)
    | L.Ident "control" when control ->
        ignore (L.next p);
        Control (read_term ~control p)
    | _ -> Var (variable p)

  let parse = L.parse (read_term ~control:false)
end

module Ctrl_code = struct
  include Code

  let parse = L.parse (read_term ~control:true)
end

module Nat_code = struct
  type t = term

  let print = Code.print

  let parse =
    (* The unextended grammar treats names such as zero and succ as variables. *)
    let rec read p =
      match L.peek p with
      | L.Ident "fun" -> (
          let saved = p.L.pos in
          match
            L.protect (fun () ->
                ignore (L.next p);
                let x = L.identifier p in
                L.expect p "=>";
                Lam (x, read p))
          with
          | Ok e -> e
          | Error _ ->
              p.L.pos <- saved;
              Var (L.identifier p))
      | L.Symbol '@' ->
          ignore (L.next p);
          L.expect_symbol p '(';
          let f = read p in
          let a = read p in
          L.expect_symbol p ')';
          App (f, a)
      | L.Symbol '(' ->
          ignore (L.next p);
          let e = read p in
          L.expect_symbol p ')';
          e
      | _ -> Var (L.identifier p)
    in
    L.parse read
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
