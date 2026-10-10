open Utils
open Machine

type command =
  | Clr of string
  | Inc of string
  | Dec of string
  | Cpy of string * string
  | Ifnz of string * Number.t
[@@deriving yojson]

type code = command list [@@deriving yojson]
type ainput = (string * Number.t) list [@@deriving yojson]
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

let parse_ainput =
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

let parse_rinput = empty_input

let get env name =
  match List.assoc_opt name env with Some value -> value | None -> Number.zero

let set env name value =
  if List.mem_assoc name env then
    List.map (fun (key, old) -> (key, if key = name then value else old)) env
  else env @ [ (name, value) ]

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

let print_env env =
  String.concat "\n" (List.map (fun (name, value) -> name ^ " = " ^ value) env)

let routput_json () = `String ""
let foutput_json env = `String (print_env env)

let render state =
  Render.state
    [
      Render.table ~title:"status"
        ~columns:[ Render.text "field"; Render.text "value" ]
        [
          [ Render.text "pc"; Render.text state.pc ];
          [
            Render.text "next";
            Render.text
              (match Number.to_int state.pc with
              | Ok pc -> (
                  match List.nth_opt state.commands pc with
                  | Some command -> print_command command
                  | None -> "halt")
              | Error _ -> "halt");
          ];
        ];
      Render.table ~title:"env"
        ~columns:[ Render.text "var"; Render.text "value" ]
        (List.map
           (fun (name, value) -> [ Render.text name; Render.text value ])
           state.env);
    ]
