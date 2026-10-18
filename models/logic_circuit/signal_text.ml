open Circuit

type t = signal

let value = Utils_base.Error.get
let parse_bool text = value (Utils_data.Text.Bool.parse text)
let print_bool = Utils_data.Text.Bool.print

module P = Utils_parsing.Parser

let read_pin p =
  let start = P.offset p in
  let owner, end_name =
    if P.bump_if_symbol p '@' then
      let name = P.identifier p in
      ( External name,
        start + 1 + String.length (Utils_names.Identifier.to_string name) )
    else if P.bump_if_symbol p '$' then (
      let start_id = P.offset p in
      if start_id <> start + 1 then P.error p "Expected contiguous $ID.PIN";
      let width =
        match P.peek p with P.Number text -> String.length text | _ -> 0
      in
      let value = P.integer p in
      (Instance (id value), start_id + width))
    else
      let name = P.identifier p in
      ( Internal name,
        start + String.length (Utils_names.Identifier.to_string name) )
  in
  let dot = P.offset p in
  if dot <> end_name then P.error p "Expected contiguous endpoint.PIN";
  P.expect_symbol p '.';
  if P.offset p <> dot + 1 then P.error p "Expected contiguous endpoint.PIN";
  let pin = P.identifier p in
  (owner, pin)

let read_item p =
  let pin = read_pin p in
  P.expect_symbol p '=';
  (pin, Utils_data.Text.Bool.read p)

let parse_pin text = value (P.parse read_pin text)
let parse_item text = value (P.parse read_item text)

let print_item ((name, pin), value) =
  owner_text name ^ "."
  ^ Utils_names.Identifier.to_string pin
  ^ "=" ^ print_bool value

let parse = P.parse (fun p -> P.separated_list p ~separator:',' read_item)
let print signal = String.concat ", " (List.map print_item signal)
