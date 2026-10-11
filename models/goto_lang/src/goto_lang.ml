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

let parse_code =
  Lexer.parse (fun p ->
      let rec loop acc =
        if Lexer.eof p then List.rev acc
        else
          let command =
            match Lexer.identifier p with
            | "clr" -> Clr (Lexer.identifier p)
            | "inc" -> Inc (Lexer.identifier p)
            | "dec" -> Dec (Lexer.identifier p)
            | "cpy" ->
                let dest = Lexer.identifier p in
                Lexer.expect p "<-";
                Cpy (dest, Lexer.identifier p)
            | "ifnz" ->
                let name = Lexer.identifier p in
                Lexer.expect_symbol p ':';
                Ifnz (name, Lexer.natural p)
            | _ -> Lexer.error p "Expected goto command"
          in
          Lexer.expect_symbol p ';';
          loop (command :: acc)
      in
      loop [])

let get env name =
  match List.assoc_opt name env with Some value -> value | None -> Number.zero

let set env name value = Utils.Assoc.set name value env
let make commands env = Ok { commands; pc = Number.zero; env }

let step state () =
  let* pc = Number.to_int state.pc in
  if pc >= List.length state.commands then Ok (Halt state.env)
  else
    let env, next_pc =
      match List.nth state.commands pc with
      | Clr name -> (set state.env name Number.zero, Number.succ state.pc)
      | Inc name ->
          ( set state.env name (Number.succ (get state.env name)),
            Number.succ state.pc )
      | Dec name ->
          ( set state.env name (Number.pred (get state.env name)),
            Number.succ state.pc )
      | Cpy (dest, source) ->
          (set state.env dest (get state.env source), Number.succ state.pc)
      | Ifnz (name, target) ->
          ( state.env,
            if Number.is_zero (get state.env name) then Number.succ state.pc
            else target )
    in
    Ok (Continue { next = { state with env; pc = next_pc }; output = () })

let snapshot = state_to_yojson
let restore = state_of_yojson

let print_command = function
  | Clr v -> "clr " ^ v ^ ";"
  | Inc v -> "inc " ^ v ^ ";"
  | Dec v -> "dec " ^ v ^ ";"
  | Cpy (d, s) -> "cpy " ^ d ^ " <- " ^ s ^ ";"
  | Ifnz (v, n) -> "ifnz " ^ v ^ " : " ^ n ^ ";"

let print_code code = String.concat "\n" (List.map print_command code)
let print_env = Utils.Environment.print Fun.id

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
                | Some command -> print_command command
                | None -> "halt")
            | Error _ -> "halt" );
        ];
      Render.assoc_table ~title:"env" Fun.id state.env;
    ]

module Ainput = struct
  type t = ainput

  let parse =
    Lexer.parse (fun p ->
        let rec loop acc =
          if Lexer.eof p then List.rev acc
          else
            let name = Lexer.identifier p in
            Lexer.expect_symbol p '=';
            let value = Lexer.natural p in
            ignore (Lexer.eat_symbol p ';');
            loop ((name, value) :: acc)
        in
        loop [])

  let print = print_env
end

module Rinput = Utils.Text.Unit
module Routput = Utils.Text.Unit
module Foutput = Ainput
