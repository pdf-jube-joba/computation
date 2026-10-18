module Assoc = Utils_base.Assoc
module Number = Utils_base.Number
module Id = Utils_names.Id
module Identifier = Utils_names.Identifier
module Intern = Utils_names.Intern
module Parser = Utils_parsing.Parser
module Render = Utils_runtime.Render
open Utils_runtime.Machine
module Var_id = Id.Make ()
module Variables = Intern.Make (Identifier) (Var_id)

type command =
  | Clr of Utils_names.Identifier.t
  | Inc of Utils_names.Identifier.t
  | Dec of Utils_names.Identifier.t
  | Cpy of Utils_names.Identifier.t * Utils_names.Identifier.t
  | Ifnz of Utils_names.Identifier.t * Number.t
[@@deriving yojson]

type code = command list [@@deriving yojson]

type ainput = (Utils_names.Identifier.t * Utils_base.Number.t) list
[@@deriving yojson]

type rinput = unit
type routput = unit
type foutput = ainput

type state = {
  commands : code;
  pc : Number.t;
  variable_names : (Utils_names.Identifier.t * Var_id.t) list;
  env : (Var_id.t * Number.t) list;
}
[@@deriving yojson]

module Code = struct
  type t = code

  let parse =
    Parser.parse (fun p ->
        Parser.many p (fun p ->
            let command =
              match Identifier.to_string (Parser.identifier p) with
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
    | Clr v -> "clr " ^ Utils_names.Identifier.to_string v ^ ";"
    | Inc v -> "inc " ^ Utils_names.Identifier.to_string v ^ ";"
    | Dec v -> "dec " ^ Utils_names.Identifier.to_string v ^ ";"
    | Cpy (d, s) ->
        "cpy "
        ^ Utils_names.Identifier.to_string d
        ^ " <- "
        ^ Utils_names.Identifier.to_string s
        ^ ";"
    | Ifnz (v, n) ->
        "ifnz "
        ^ Utils_names.Identifier.to_string v
        ^ " : " ^ Number.to_string n ^ ";"

  let print code = String.concat "\n" (List.map print_command code)
end

let make commands input =
  let variables = ref Variables.empty and names = ref [] in
  let variable name =
    match Variables.find name !variables with
    | Some id -> id
    | None ->
        let id = Variables.take name variables in
        names := !names @ [ (name, id) ];
        id
  in
  List.iter
    (function
      | Clr name | Inc name | Dec name | Ifnz (name, _) ->
          ignore (variable name)
      | Cpy (a, b) ->
          ignore (variable a);
          ignore (variable b))
    commands;
  let env = List.map (fun (name, value) -> (variable name, value)) input in
  Ok { commands; pc = Number.zero; env; variable_names = !names }

let variable_id state name =
  Assoc.lookup name state.variable_names
    ("Unknown variable: " ^ Utils_names.Identifier.to_string name)

let named_env state =
  List.map
    (fun (id, value) ->
      let name =
        fst
          (List.find
             (fun (_, other) -> Var_id.equal id other)
             state.variable_names)
      in
      (name, value))
    state.env

let lookup state name =
  Option.value ~default:Number.zero
    (List.assoc_opt (variable_id state name) state.env)

let step state () =
  let* pc = Number.to_int state.pc in
  if pc >= List.length state.commands then Ok (Halt (named_env state))
  else
    let env, next_pc =
      match List.nth state.commands pc with
      | Clr name ->
          ( Assoc.set (variable_id state name) Number.zero state.env,
            Number.succ state.pc )
      | Inc name ->
          ( Assoc.set (variable_id state name)
              (Number.succ
                 (Option.value ~default:Number.zero
                    (List.assoc_opt (variable_id state name) state.env)))
              state.env,
            Number.succ state.pc )
      | Dec name ->
          ( Assoc.set (variable_id state name)
              (Number.pred
                 (Option.value ~default:Number.zero
                    (List.assoc_opt (variable_id state name) state.env)))
              state.env,
            Number.succ state.pc )
      | Cpy (dest, source) ->
          ( Assoc.set (variable_id state dest)
              (Option.value ~default:Number.zero
                 (List.assoc_opt (variable_id state source) state.env))
              state.env,
            Number.succ state.pc )
      | Ifnz (name, target) ->
          ( state.env,
            if
              Number.is_zero
                (Option.value ~default:Number.zero
                   (List.assoc_opt (variable_id state name) state.env))
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
          ("pc", Number.to_string state.pc);
          ( "next",
            match Number.to_int state.pc with
            | Ok pc -> (
                match List.nth_opt state.commands pc with
                | Some command -> Code.print_command command
                | None -> "halt")
            | Error _ -> "halt" );
        ];
      Render.keyed_table ~key_text:Utils_names.Identifier.to_string ~title:"env"
        Number.to_string (named_env state);
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

  let print = Utils_codec.Environment_text.print Number.to_string
end

module Rinput = Utils_codec.Text.Unit
module Routput = Utils_codec.Text.Unit
module Foutput = Ainput
