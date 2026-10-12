open Utils
open Machine

type command =
  | Clr of Utils.Identifier.t
  | Inc of Utils.Identifier.t
  | Dec of Utils.Identifier.t
  | Cpy of Utils.Identifier.t * Utils.Identifier.t
  | Ifnz of Utils.Identifier.t * Number.t
[@@deriving yojson]

type code = command list [@@deriving yojson]
type ainput = (Utils.Identifier.t * Number.t) list [@@deriving yojson]
type rinput = unit
type routput = unit
type foutput = ainput

type state = { commands : code; pc : Number.t; env : ainput }
[@@deriving yojson]

module Code = struct
  type t = code

  let parse =
    Parser.parse (fun p ->
        Parser.many p (fun p ->
            let command =
              match Parser.identifier p with
              | "clr" -> Clr (Parser.identifier p)
              | "inc" -> Inc (Parser.identifier p)
              | "dec" -> Dec (Parser.identifier p)
              | "cpy" ->
                  let dest = Parser.identifier p in
                  Parser.expect p "<-";
                  Cpy (dest, Parser.identifier p)
              | "ifnz" ->
                  let name = Parser.identifier p in
                  Parser.expect_symbol p ':';
                  Ifnz (name, Parser.natural p)
              | _ -> Parser.error p "Expected goto command"
            in
            Parser.expect_symbol p ';';
            command))

  let print_command = function
    | Clr v -> "clr " ^ v ^ ";"
    | Inc v -> "inc " ^ v ^ ";"
    | Dec v -> "dec " ^ v ^ ";"
    | Cpy (d, s) -> "cpy " ^ d ^ " <- " ^ s ^ ";"
    | Ifnz (v, n) -> "ifnz " ^ v ^ " : " ^ n ^ ";"

  let print code = String.concat "\n" (List.map print_command code)
end

let make commands env = Ok { commands; pc = Number.zero; env }

let step state () =
  let* pc = Number.to_int state.pc in
  if pc >= List.length state.commands then Ok (Halt state.env)
  else
    let env, next_pc =
      match List.nth state.commands pc with
      | Clr name -> (Assoc.set name Number.zero state.env, Number.succ state.pc)
      | Inc name ->
          ( Assoc.set name
              (Number.succ
                 (Option.value ~default:Number.zero
                    (List.assoc_opt name state.env)))
              state.env,
            Number.succ state.pc )
      | Dec name ->
          ( Assoc.set name
              (Number.pred
                 (Option.value ~default:Number.zero
                    (List.assoc_opt name state.env)))
              state.env,
            Number.succ state.pc )
      | Cpy (dest, source) ->
          ( Assoc.set dest
              (Option.value ~default:Number.zero
                 (List.assoc_opt source state.env))
              state.env,
            Number.succ state.pc )
      | Ifnz (name, target) ->
          ( state.env,
            if
              Number.is_zero
                (Option.value ~default:Number.zero
                   (List.assoc_opt name state.env))
            then Number.succ state.pc
            else target )
    in
    Ok (Continue { next = { state with env; pc = next_pc }; output = () })

let snapshot = state_to_yojson
let restore = state_of_yojson

let render state =
  Render.state
    [
      Render.assoc_table ~title:"status" ~columns:[ "field"; "value" ] Fun.id
        [
          ("pc", state.pc);
          ( "next",
            match Number.to_int state.pc with
            | Ok pc -> (
                match List.nth_opt state.commands pc with
                | Some command -> Code.print_command command
                | None -> "halt")
            | Error _ -> "halt" );
        ];
      Render.assoc_table ~title:"env" Fun.id state.env;
    ]

module Ainput = struct
  type t = ainput

  let parse =
    Parser.parse (fun p ->
        Parser.many p (fun p ->
            let name = Parser.identifier p in
            Parser.expect_symbol p '=';
            let value = Parser.natural p in
            ignore (Parser.bump_if_symbol p ';');
            (name, value)))

  let print = Utils.Environment.print Fun.id
end

module Rinput = Utils.Text.Unit
module Routput = Utils.Text.Unit
module Foutput = Ainput
