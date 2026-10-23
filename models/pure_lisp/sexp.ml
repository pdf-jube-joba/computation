(* Code and values share the same representation, including function values. *)
type t = Nil | Symbol of string | Cons of t * t [@@deriving yojson]

let list xs = List.fold_right (fun x tail -> Cons (x, tail)) xs Nil

let to_list value =
  let rec loop acc = function
    | Nil -> Ok (List.rev acc)
    | Cons (head, tail) -> loop (head :: acc) tail
    | _ -> Error "Expected a proper list"
  in
  loop [] value

let rec print = function
  | Nil -> "nil"
  | Symbol name -> name
  | Cons (head, tail) -> "(" ^ print head ^ print_tail tail ^ ")"

and print_tail = function
  | Nil -> ""
  | Cons (head, tail) -> " " ^ print head ^ print_tail tail
  | tail -> " . " ^ print tail

(* Lisp symbols, semicolon comments and quote abbreviations have a different
   lexical grammar from the shared infix-language parser. *)
exception Parse_error of string

let parse source =
  let pos = ref 0 and length = String.length source in
  let error message =
    raise (Parse_error (Printf.sprintf "%s at byte %d" message !pos))
  in
  let delimiter = function
    | ' ' | '\t' | '\r' | '\n' | '(' | ')' | '\'' | ';' -> true
    | _ -> false
  in
  let rec skip () =
    if !pos < length then
      match source.[!pos] with
      | ' ' | '\t' | '\r' | '\n' ->
          incr pos;
          skip ()
      | ';' ->
          while !pos < length && source.[!pos] <> '\n' do
            incr pos
          done;
          skip ()
      | _ -> ()
  in
  let dot () =
    !pos < length
    && source.[!pos] = '.'
    && (!pos + 1 = length || delimiter source.[!pos + 1])
  in
  let rec read () =
    skip ();
    if !pos = length then error "Expected an S-expression";
    match source.[!pos] with
    | '(' ->
        incr pos;
        read_list []
    | ')' -> error "Unexpected ')'"
    | '\'' ->
        incr pos;
        list [ Symbol "quote"; read () ]
    | _ when dot () -> error "Unexpected dot"
    | _ ->
        let start = !pos in
        while !pos < length && not (delimiter source.[!pos]) do
          incr pos
        done;
        let name = String.sub source start (!pos - start) in
        if name = "nil" then Nil else Symbol name
  and read_list acc =
    skip ();
    if !pos = length then error "Unclosed list";
    if source.[!pos] = ')' then (
      incr pos;
      list (List.rev acc))
    else if dot () then (
      if acc = [] then error "Dot requires a preceding element";
      incr pos;
      let tail = read () in
      skip ();
      if !pos = length || source.[!pos] <> ')' then
        error "Expected ')' after dotted tail";
      incr pos;
      List.fold_left (fun tail head -> Cons (head, tail)) tail acc)
    else
      let head = read () in
      read_list (head :: acc)
  in
  try
    let value = read () in
    skip ();
    if !pos <> length then error "Expected one S-expression";
    Ok value
  with Parse_error message -> Error message
