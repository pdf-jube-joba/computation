type token = Lexer.token =
  | Ident of string
  | Keyword of string
  | Number of string
  | String_lit of string
  | Character_lit of string
  | Symbol of char
  | Eof

type delimiter = Parentheses | Brackets | Braces

type node =
  | Atom of { token : token; offset : int }
  | Group of {
      delimiter : delimiter;
      children : node list;
      opening_offset : int;
      closing_offset : int;
    }

type t = {
  tokens : (token * int) array;
  groups : (int * node list) option array;
  tree : node list;
  mutable pos : int;
  limit : int;
}

type checkpoint = t * int

exception Error = Lexer.Error

let error_at offset message =
  raise (Error (Printf.sprintf "%s at byte %d" message offset))

let offset p = snd p.tokens.(p.pos)
let error p message = error_at (offset p) message

let delimiter = function
  | '(' -> (Parentheses, ')')
  | '[' -> (Brackets, ']')
  | '{' -> (Braces, '}')
  | _ -> invalid_arg "Expected one of '(', '[' or '{'"

let create ?(nested_comments = true) ?(character_literals = false) text =
  let tokens = Lexer.tokenize ~nested_comments ~character_literals text in
  let groups = Array.make (Array.length tokens) None in
  let rec sequence i opened acc =
    let token, offset = tokens.(i) in
    match token with
    | Eof -> (
        match opened with
        | None -> (List.rev acc, i)
        | Some (opening, opening_offset, expected) ->
            error_at offset
              (Printf.sprintf "Unclosed '%c' at byte %d; expected '%c'" opening
                 opening_offset expected))
    | Symbol (('(' | '[' | '{') as opening) ->
        let kind, expected = delimiter opening in
        let children, j =
          sequence (i + 1) (Some (opening, offset, expected)) []
        in
        groups.(i) <- Some (j, children);
        let group =
          Group
            {
              delimiter = kind;
              children;
              opening_offset = offset;
              closing_offset = snd tokens.(j);
            }
        in
        sequence (j + 1) opened (group :: acc)
    | Symbol ((')' | ']' | '}') as c) -> (
        match opened with
        | Some (_, _, expected) when c = expected -> (List.rev acc, i)
        | Some (opening, opening_offset, expected) ->
            error_at offset
              (Printf.sprintf "Expected '%c' for '%c' at byte %d, found '%c'"
                 expected opening opening_offset c)
        | None -> error_at offset (Printf.sprintf "Unexpected closing '%c'" c))
    | _ -> sequence (i + 1) opened (Atom { token; offset } :: acc)
  in
  let tree, limit = sequence 0 None [] in
  { tokens; groups; tree; pos = 0; limit }

let tree p = p.tree
let checkpoint p = (p, p.pos)

let restore p (owner, pos) =
  if owner != p then invalid_arg "Checkpoint belongs to another parser";
  p.pos <- pos

let peek p = if p.pos = p.limit then Eof else fst p.tokens.(p.pos)

let bump p =
  let token = peek p in
  if token <> Eof then p.pos <- p.pos + 1;
  token

let eof p = peek p = Eof
let expect_eof p = if not (eof p) then error p "Unexpected trailing tokens"

let bump_if_symbol p c =
  if peek p = Symbol c then (
    ignore (bump p);
    true)
  else false

let expect_symbol p c =
  if not (bump_if_symbol p c) then error p (Printf.sprintf "Expected '%c'" c)

let bump_if_keyword p word =
  if peek p = Keyword word then (
    ignore (bump p);
    true)
  else false

let expect_keyword p word =
  if not (bump_if_keyword p word) then error p ("Expected \\" ^ word)

let keyword p =
  match peek p with
  | Keyword word ->
      ignore (bump p);
      word
  | _ -> error p "Expected a keyword starting with '\\'"

let identifier p =
  match peek p with
  | Ident s ->
      ignore (bump p);
      Utils_base.Error.get (Utils_names.Identifier.make s)
  | _ -> error p "Expected identifier"

let natural p =
  match peek p with
  | Number s ->
      ignore (bump p);
      Utils_base.Error.get (Utils_base.Number.of_string s)
  | _ -> error p "Expected natural number"

let natural_with_plus p =
  let offset = snd p.tokens.(p.pos) in
  if bump_if_symbol p '+' && snd p.tokens.(p.pos) <> offset + 1 then
    error p "Expected digits immediately after '+'";
  natural p

let integer p =
  match peek p with
  | Number s -> (
      match
        int_of_string_opt
          (Utils_base.Number.to_string
             (Utils_base.Error.get (Utils_base.Number.of_string s)))
      with
      | Some n when n >= 0 ->
          ignore (bump p);
          n
      | _ -> error p "Integer out of range")
  | _ -> error p "Expected natural number"

let string_literal p =
  match peek p with
  | String_lit s ->
      ignore (bump p);
      s
  | _ -> error p "Expected string literal"

let character_literal p =
  match peek p with
  | Character_lit c ->
      ignore (bump p);
      c
  | _ -> error p "Expected character literal"

let bump_if p text =
  let saved = p.pos in
  let wanted = Lexer.tokenize text in
  let rec loop i =
    if i = Array.length wanted - 1 then true
    else if peek p = fst wanted.(i) then (
      ignore (bump p);
      loop (i + 1))
    else false
  in
  if loop 0 then true
  else (
    p.pos <- saved;
    false)

let expect p text = if not (bump_if p text) then error p ("Expected " ^ text)

let opening_symbol = function
  | Parentheses -> '('
  | Brackets -> '['
  | Braces -> '{'

let is_group p delimiter = peek p = Symbol (opening_symbol delimiter)

let group p delimiter f =
  if not (is_group p delimiter) then
    error p (Printf.sprintf "Expected '%c'" (opening_symbol delimiter));
  (* Reuse the already constructed subtree; never re-tokenize a group. *)
  let limit, tree =
    match p.groups.(p.pos) with Some group -> group | None -> assert false
  in
  let inner = { p with tree; pos = p.pos + 1; limit } in
  let value = f inner in
  expect_eof inner;
  p.pos <- limit + 1;
  value

let bump_group_if p delimiter f =
  if is_group p delimiter then Some (group p delimiter f) else None

let protect f =
  try Ok (f ())
  with Error message | Failure message | Invalid_argument message ->
    Result.Error message

let parse ?(nested_comments = true) ?(character_literals = false) f text =
  protect (fun () ->
      let p = create ~nested_comments ~character_literals text in
      let value = f p in
      expect_eof p;
      value)

let item_with_progress p item =
  let before = p.pos in
  let value = item p in
  if p.pos <= before then error p "Parser item consumed no tokens";
  value

let attempt p f =
  let saved = checkpoint p in
  match protect (fun () -> f p) with
  | Ok _ as result -> result
  | Error _ as result ->
      restore p saved;
      result

let chain_left ?(allow_trailing = false) p ~separator ~combine item =
  let rec loop lhs =
    if bump_if_symbol p separator then
      if allow_trailing && eof p then lhs
      else
        let rhs = item_with_progress p item in
        loop (combine lhs rhs)
    else lhs
  in
  loop (item_with_progress p item)

let binary p ~operators ~operand ~make =
  let lhs = item_with_progress p operand in
  let rec operator = function
    | (text, op) :: rest ->
        let before = p.pos in
        if bump_if p text then (
          if p.pos = before then error p "Empty binary operator";
          op)
        else operator rest
    | [] -> error p "Expected binary operator"
  in
  let op = operator operators in
  let rhs = item_with_progress p operand in
  make lhs op rhs

let expression ?(prefix = []) ~infix ~atom p =
  let rec read min_bp p =
    let rec head = function
      | (text, rbp, apply) :: rest ->
          let before = p.pos in
          if bump_if p text then (
            if p.pos = before then error p "Empty prefix operator";
            apply (read rbp p))
          else head rest
      | [] -> item_with_progress p (atom (read 0))
    in
    let rec tail lhs operators =
      match operators with
      | (text, lbp, rbp, apply) :: rest ->
          let before = p.pos in
          if lbp >= min_bp && bump_if p text then (
            if p.pos = before then error p "Empty infix operator";
            let rhs = read rbp p in
            tail (apply lhs rhs) infix)
          else tail lhs rest
      | [] -> lhs
    in
    tail (head prefix) infix
  in
  read 0 p

let many ?skip_separator p item =
  let rec loop acc =
    match skip_separator with
    | Some c when bump_if_symbol p c -> loop acc
    | _ when eof p -> List.rev acc
    | _ ->
        let value = item_with_progress p item in
        loop (value :: acc)
  in
  loop []

let rec separated p ~separator item =
  let first = item_with_progress p item in
  if bump_if_symbol p separator then first :: separated p ~separator item
  else [ first ]

let separated_list ?(allow_trailing = false) ?(skip_empty = false) p ~separator
    item =
  let rec skip () = if skip_empty && bump_if_symbol p separator then skip () in
  let rec loop acc =
    skip ();
    if skip_empty && eof p then List.rev acc
    else
      let value = item_with_progress p item in
      let acc = value :: acc in
      if bump_if_symbol p separator then
        if allow_trailing && eof p then List.rev acc else loop acc
      else (
        expect_eof p;
        List.rev acc)
  in
  if eof p then [] else loop []
