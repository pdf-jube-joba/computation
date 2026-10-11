type ('state, 'routput, 'foutput) step_result =
  | Continue of { next : 'state; output : 'routput }
  | Halt of 'foutput

module type S = sig
  type code
  type ainput
  type rinput
  type routput
  type foutput
  type state

  val parse_code : string -> (code, string) result
  val parse_ainput : string -> (ainput, string) result
  val parse_rinput : string -> (rinput, string) result
  val make : code -> ainput -> (state, string) result

  val step :
    state -> rinput -> ((state, routput, foutput) step_result, string) result

  val snapshot : state -> Yojson.Safe.t
  val restore : Yojson.Safe.t -> (state, string) result
  val render : state -> Yojson.Safe.t
  val routput_json : routput -> Yojson.Safe.t
  val foutput_json : foutput -> Yojson.Safe.t
end

let ( let* ) = Result.bind

let empty_input text =
  Lexer.protect (fun () ->
      if String.trim (Lexer.without_comments text) = "" then ()
      else failwith "Expected empty input")

let natural text =
  Lexer.protect (fun () ->
      match int_of_string_opt (String.trim (Lexer.without_comments text)) with
      | Some n when n >= 0 -> n
      | _ -> failwith "Expected a nonnegative OCaml integer")

let text_render text =
  `List [ `Assoc [ ("kind", `String "text"); ("text", `String text) ] ]
