open Structured_lang_shared.Arithmetic
open Runtime_types

type t = value

let location_text (l : location) =
  Printf.sprintf "\\loc(%d, %d)" (Frame_id.to_int l.frame_id) l.slot

let print = function
  | Num n -> Int64.to_string n
  | Location l -> location_text l
  | Null_ptr -> "\\null_ptr"

let parse text =
  L.protect (fun () ->
      let text = String.trim text in
      if text = "\\null_ptr" then Null_ptr
      else if
        String.starts_with ~prefix:"\\loc(" text
        && String.ends_with ~suffix:")" text
      then (
        let p = L.create text in
        L.expect_keyword p "loc";
        let location =
          L.group p L.Parentheses (fun p ->
              let frame_id = get (Frame_id.of_int (L.integer p)) in
              L.expect_symbol p ',';
              let slot = L.integer p in
              Location { frame_id; slot })
        in
        L.expect_eof p;
        location)
      else Num (Int64.of_string text))
