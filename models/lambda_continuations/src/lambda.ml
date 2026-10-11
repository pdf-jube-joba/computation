module N = Utils.Number
module L = Utils.Lexer
module Names = Set.Make (String)

(* Abort/Control extend the same structure, so the implemented variable
   operations can be shared with Ctrl. Its reduction is deliberately unsupported. *)
type term =
  | Var of string
  | Lam of string * term
  | App of term * term
  | Zero
  | Succ of term
  | Pred of term
  | IfZ of term * term * term
  | Let of string * term * term
  | Rec of string * string * term
  | Abort of term
  | Control of term
[@@deriving yojson]

type lam = term [@@deriving yojson]
type value = Fun of string * term | Num of N.t [@@deriving yojson]

let n_v x = Var x
let n_l x b = Lam (x, b)
let n_a f a = App (f, a)
let n_z () = Zero
let n_s e = Succ e
let n_p e = Pred e
let n_i c t f = IfZ (c, t, f)
let n_d x e b = Let (x, e, b)
let n_r f x b = Rec (f, x, b)

let rec num_to_exp n =
  if N.is_zero n then Zero else Succ (num_to_exp (N.pred n))

let rec exp_to_num = function
  | Zero -> Some N.zero
  | Succ n -> Option.map N.succ (exp_to_num n)
  | _ -> None

let lam_to_value = function
  | Lam (x, b) -> Some (Fun (x, b))
  | e -> Option.map (fun n -> Num n) (exp_to_num e)

let value_to_lam = function Fun (x, b) -> Lam (x, b) | Num n -> num_to_exp n
let is_value = lam_to_value

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

let fresh used =
  let rec loop i =
    let x = "_" ^ string_of_int i in
    if Names.mem x used then loop (i + 1) else x
  in
  loop 0

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
      let y = fresh avoid in
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
          let y = fresh used in
          (y, rename b x y)
        else (x, b)
      in
      Rec (f, x, subst b target replacement)
  | Abort e -> Abort (subst e target replacement)
  | Control e -> Control (subst e target replacement)

(* Bound variables are compared by lexical depth, never by a printed dummy name. *)
let alpha_eq a b =
  let rec index x i = function
    | [] -> None
    | y :: ys -> if x = y then Some i else index x (i + 1) ys
  in
  let rec eq left right a b =
    match (a, b) with
    | Var x, Var y -> (
        match (index x 0 left, index y 0 right) with
        | None, None -> x = y
        | i, j -> i = j)
    | Lam (x, a), Lam (y, b) -> eq (x :: left) (y :: right) a b
    | App (a, b), App (c, d) -> eq left right a c && eq left right b d
    | Zero, Zero -> true
    | Succ a, Succ b | Pred a, Pred b | Abort a, Abort b | Control a, Control b
      ->
        eq left right a b
    | IfZ (a, b, c), IfZ (d, e, f) ->
        eq left right a d && eq left right b e && eq left right c f
    | Let (x, a, b), Let (y, c, d) ->
        eq left right a c && eq (x :: left) (y :: right) b d
    | Rec (f, x, b), Rec (g, y, c) -> eq (x :: f :: left) (y :: g :: right) b c
    | _ -> false
  in
  eq [] [] a b

(* Preserve no_ext's display syntax; write_code emits the checked-in pest syntax. *)
let rec print = function
  | Var x -> x
  | Lam (x, b) -> "fun " ^ x ^ " => " ^ print b
  | App (a, b) -> "(" ^ print a ^ " " ^ print b ^ ")"
  | Zero -> "0"
  | Succ e -> "S " ^ print e
  | Pred e -> "P " ^ print e
  | IfZ (c, t, f) -> "if " ^ print c ^ " then " ^ print t ^ " else " ^ print f
  | Let (x, e, b) -> "let " ^ x ^ " = " ^ print e ^ " in \n " ^ print b
  | Rec (f, x, b) -> "rec " ^ f ^ " " ^ x ^ " = " ^ print b
  | Abort e -> "abort " ^ print e
  | Control e -> "control " ^ print e

let rec write_code = function
  | Var x -> x
  | Lam (x, b) -> "(fun " ^ x ^ " => " ^ write_code b ^ ")"
  | App (a, b) -> "@(" ^ write_code a ^ " " ^ write_code b ^ ")"
  | Zero -> "zero"
  | Succ e -> "(succ " ^ write_code e ^ ")"
  | Pred e -> "(pred " ^ write_code e ^ ")"
  | IfZ (c, t, f) ->
      "(ifz " ^ write_code c ^ " then " ^ write_code t ^ " else " ^ write_code f
      ^ ")"
  | Let (x, e, b) ->
      "(let " ^ x ^ " = " ^ write_code e ^ " in " ^ write_code b ^ ")"
  | Rec (f, x, b) -> "(rec " ^ f ^ " " ^ x ^ " = " ^ write_code b ^ ")"
  | Abort e -> "(abort " ^ write_code e ^ ")"
  | Control e -> "(control " ^ write_code e ^ ")"

let word p =
  let name = L.identifier p in
  (* The pest variable production permits a hyphen inside a name. Only join
     adjacent tokens, so spaced syntax remains unambiguous. *)
  let last_end = ref (snd p.L.tokens.(p.L.pos - 1) + String.length name) in
  let acc = Buffer.create 16 in
  Buffer.add_string acc name;
  let rec loop () =
    let tok, offset = p.L.tokens.(p.L.pos) in
    if offset = !last_end then
      match tok with
      | L.Symbol '-' ->
          ignore (L.next p);
          Buffer.add_char acc '-';
          incr last_end;
          loop ()
      | L.Ident s | L.Number s ->
          ignore (L.next p);
          Buffer.add_string acc s;
          last_end := !last_end + String.length s;
          loop ()
      | _ -> ()
  in
  loop ();
  Buffer.contents acc

let variable p =
  let name = word p in
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
  | L.Ident name when word { p with L.pos = p.L.pos } <> name ->
      Var (variable p)
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

let parse_code = L.parse (read_term ~control:false)
let parse_ctrl_code = L.parse (read_term ~control:true)

let parse_nat_code =
  (* The unextended grammar treats names such as zero and succ as variables. *)
  let rec read p =
    match L.peek p with
    | L.Ident "fun" when word { p with L.pos = p.L.pos } = "fun" -> (
        let saved = p.L.pos in
        match
          L.protect (fun () ->
              ignore (L.next p);
              let x = word p in
              L.expect p "=>";
              Lam (x, read p))
        with
        | Ok e -> e
        | Error _ ->
            p.L.pos <- saved;
            Var (word p))
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
    | _ -> Var (word p)
  in
  L.parse read
