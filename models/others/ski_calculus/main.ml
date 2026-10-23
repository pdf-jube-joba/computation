open Utils_runtime.Machine
module Identifier = Utils_names.Identifier

type term = S | K | I | Var of Identifier.t | App of term * term
[@@deriving yojson]

let combinators word =
  String.for_all (fun c -> c = 'S' || c = 'K' || c = 'I') word

let rec validate = function
  | Var name when combinators (Identifier.to_string name) ->
      Error "S, K and I are combinators, not variable names"
  | App (left, right) ->
      let* () = validate left in
      validate right
  | _ -> Ok ()

module Code = struct
  type t = term

  module P = Utils_parsing.Parser

  let read p =
    let rec expression p =
      let first = atom p in
      let append = List.fold_left (fun left right -> App (left, right)) in
      let rec rest left =
        if P.eof p then left else rest (append left (atom p))
      in
      match first with
      | first :: tail -> rest (append first tail)
      | [] -> P.error p "Expected a term"
    and atom p =
      if P.is_group p P.Parentheses then [ P.group p P.Parentheses expression ]
      else
        match P.peek p with
        | P.Ident word ->
            ignore (P.bump p);
            if combinators word then
              let combinator = function 'S' -> S | 'K' -> K | _ -> I in
              List.init (String.length word) (fun i -> combinator word.[i])
            else [ Var (Identifier.make_exn word) ]
        | _ -> P.error p "Expected S, K, I, a variable or a parenthesized term"
    in
    expression p

  let parse = P.parse read

  let rec print = function
    | S -> "S"
    | K -> "K"
    | I -> "I"
    | Var name -> Identifier.to_string name
    | App (left, right) -> (
        print left ^ " "
        ^ match right with App _ -> "(" ^ print right ^ ")" | _ -> print right)
end

(* Normal order: contract the outermost redex first, then search left to right. *)
let rec reduce = function
  | App (I, x) -> Some x
  | App (App (K, x), _) -> Some x
  | App (App (App (S, x), y), z) -> Some (App (App (x, z), App (y, z)))
  | App (left, right) -> (
      match reduce left with
      | Some left -> Some (App (left, right))
      | None -> Option.map (fun right -> App (left, right)) (reduce right))
  | S | K | I | Var _ -> None

type state = term

let make code () =
  let* () = validate code in
  Ok code

let step term () =
  match reduce term with
  | None -> Ok (Halt term)
  | Some next -> Ok (Continue { next; output = () })

module Snapshot = struct
  type t = state

  let to_yojson = term_to_yojson

  let of_yojson json =
    let* term = term_of_yojson json in
    let* () = validate term in
    Ok term
end

let render term = [ Utils_runtime.Render.text ~title:"term" (Code.print term) ]

module Ainput = Utils_data.Text.Unit
module Rinput = Utils_data.Text.Unit
module Routput = Utils_data.Text.Unit
module Foutput = Code
