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

  val snapshot : state -> Yojson.Basic.t
  val restore : Yojson.Basic.t -> (state, string) result
  val render : state -> Yojson.Basic.t
  val routput_json : routput -> Yojson.Basic.t
  val foutput_json : foutput -> Yojson.Basic.t
end

let ( let* ) = Result.bind

let empty_input text =
  if String.trim text = "" then Ok () else Error "Expected empty input"

let natural text =
  match int_of_string_opt (String.trim text) with
  | Some n when n >= 0 -> Ok n
  | _ -> Error "Expected a nonnegative OCaml integer"

let text_render text =
  `List [ `Assoc [ ("kind", `String "text"); ("text", `String text) ] ]
