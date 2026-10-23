open Types
module P = Utils_parsing.Parser

type t = memory

let rec read_slice p =
  if P.bump_if_keyword p "slice" then
    let s = read_slice p in
    let b = P.natural p in
    let e = P.natural p in
    Subslice (s, b, e)
  else
    match Code.value p with
    | Memory m -> Whole m
    | _ -> P.error p "Expected memory or slice"

let read_value p =
  if P.bump_if_keyword p "addr" then
    let s = read_slice p in
    let i = P.natural p in
    Address (s, i)
  else if P.peek p = P.Keyword "slice" then Slice_value (read_slice p)
  else
    match Code.value p with
    | Imm n -> Num n
    | Label l -> Code_label l
    | Memory m -> Slice_value (Whole m)
    | Reg _ -> P.error p "Registers are not input values"

let rec slice_text = function
  | Whole m -> "MEM_" ^ string_of_int m
  | Subslice (s, b, e) ->
      "\\slice " ^ slice_text s ^ " " ^ N.to_string b ^ " " ^ N.to_string e

let value_text = function
  | Num n -> N.to_string n
  | Code_label l -> ":" ^ Id.to_string l
  | Slice_value s -> slice_text s
  | Address (s, i) -> "\\addr " ^ slice_text s ^ " " ^ N.to_string i

let parse =
  P.parse (fun p ->
      P.many p (fun p ->
          let i = P.natural p in
          P.expect p ":=";
          let v = read_value p in
          P.expect_symbol p ';';
          (i, v)))

let print xs =
  String.concat ""
    (List.map (fun (i, v) -> N.to_string i ^ " := " ^ value_text v ^ ";\n") xs)
