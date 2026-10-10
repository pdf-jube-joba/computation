open Common

type binop = Add | Sub [@@deriving yojson]

type place_expr = Var of string | Deref of value_expr

and value_expr =
  | Number of int64
  | NullPtr
  | BinOp of value_expr * binop * value_expr
  | Load of place_expr
  | Addr of place_expr
[@@deriving yojson]

type stmt =
  | Assign of place_expr * value_expr
  | Ifz of value_expr * stmt * stmt
  | Call of string * value_expr list
  | Return
[@@deriving yojson]

type fn_decl = { name : string; params : string list; body : stmt list }
[@@deriving yojson]

type code = { functions : fn_decl list } [@@deriving yojson]
type location = { frame_id : int; slot : int } [@@deriving yojson]

type value = Num of int64 | Location of location | Null_ptr
[@@deriving yojson]

type frame_store = { frame_id : int; values : (location * value) list }
[@@deriving yojson]

type call_frame = {
  env : (string * location) list;
  inner_k : stmt list;
  function_name : string;
}
[@@deriving yojson]

type state = {
  code : code;
  current_function : string;
  current_stmt : stmt option;
  env : (string * location) list;
  store : frame_store list;
  inner_k : stmt list;
  fn_k : call_frame list;
  next_frame_id : int;
}
[@@deriving yojson]

type ainput = unit
type rinput = unit
type routput = string
type foutput = string

let rec place_to_text = function
  | Var v -> v
  | Deref v -> "(" ^ value_to_text v ^ ") #loc"

and value_to_text = function
  | Number n -> Int64.to_string n
  | NullPtr -> "#null-ptr"
  | BinOp (a, op, b) ->
      "(" ^ value_to_text a
      ^ (if op = Add then " + " else " - ")
      ^ value_to_text b ^ ")"
  | Load p -> "ld " ^ place_to_text p
  | Addr p -> place_to_text p ^ " #addr"

let rec stmt_to_text = function
  | Assign (p, v) -> "assign " ^ place_to_text p ^ " := " ^ value_to_text v
  | Ifz (c, a, b) ->
      "ifz " ^ value_to_text c ^ " then " ^ stmt_to_text a ^ " else "
      ^ stmt_to_text b ^ " end"
  | Call (n, args) ->
      "call " ^ n ^ "(" ^ String.concat ", " (List.map value_to_text args) ^ ")"
  | Return -> "return"

let print c =
  String.concat "\n\n"
    (List.map
       (fun f ->
         "fn " ^ f.name ^ "("
         ^ String.concat ", " f.params
         ^ ") { "
         ^ String.concat "; " (List.map stmt_to_text f.body)
         ^ " }")
       c.functions)

let location_text (l : location) =
  Printf.sprintf "loc(%d, %d)" l.frame_id l.slot

let value_text = function
  | Num n -> Int64.to_string n
  | Location l -> location_text l
  | Null_ptr -> "#null-ptr"

let parse_runtime_value text =
  L.protect (fun () ->
      let text = String.trim text in
      if text = "#null-ptr" then Null_ptr
      else if
        String.starts_with ~prefix:"loc(" text
        && String.ends_with ~suffix:")" text
      then
        let body = String.sub text 4 (String.length text - 5) in
        match String.split_on_char ',' body with
        | [ frame; slot ] ->
            let frame_id = get (N.to_int (get (N.of_string frame))) in
            let slot = get (N.to_int (get (N.of_string slot))) in
            Location { frame_id; slot }
        | _ -> failwith "expected loc(frame,slot)"
      else Num (Int64.of_string text))

let print_runtime_value = value_text

let rec parse_place p =
  match L.peek p with
  | L.Ident _ -> Var (ident p)
  | _ ->
      L.expect_symbol p '(';
      let v = parse_value p in
      L.expect p ") #loc";
      Deref v

and parse_value p =
  let a = parse_atom p in
  let rec loop a =
    if L.eat_symbol p '+' then
      let b = parse_atom p in
      loop (BinOp (a, Add, b))
    else if L.eat_symbol p '-' then
      let b = parse_atom p in
      loop (BinOp (a, Sub, b))
    else a
  in
  loop a

and parse_atom p =
  if L.eat p "#null-ptr" then NullPtr
  else if L.eat_keyword p "ld" then Load (parse_place p)
  else if L.eat_symbol p '(' then (
    let v = parse_value p in
    L.expect_symbol p ')';
    v)
  else
    match L.peek p with
    | L.Number _ -> Number (Int64.of_string (L.natural p))
    | _ ->
        let v = parse_place p in
        L.expect p "#addr";
        Addr v

let rec parse_stmt p =
  if L.eat_keyword p "assign" then (
    let place = parse_place p in
    L.expect p ":=";
    Assign (place, parse_value p))
  else if L.eat_keyword p "ifz" then (
    let cond = parse_value p in
    L.expect_keyword p "then";
    let a = parse_stmt p in
    L.expect_keyword p "else";
    let b = parse_stmt p in
    L.expect_keyword p "end";
    Ifz (cond, a, b))
  else if L.eat_keyword p "call" then (
    let n = ident p in
    L.expect_symbol p '(';
    let rec args acc =
      let v = parse_value p in
      if L.eat_symbol p ',' then args (v :: acc) else List.rev (v :: acc)
    in
    let args = if L.peek p = L.Symbol ')' then [] else args [] in
    L.expect_symbol p ')';
    Call (n, args))
  else (
    L.expect_keyword p "return";
    Return)

let parse_code =
  L.parse (fun p ->
      let rec fs acc =
        if L.eof p then { functions = List.rev acc }
        else (
          L.expect_keyword p "fn";
          let name = ident p in
          L.expect_symbol p '(';
          let params = names_until p ')' in
          L.expect p ") {";
          let rec stmts acc =
            if L.peek p = L.Symbol '}' then List.rev acc
            else
              let s = parse_stmt p in
              if L.eat_symbol p ';' then stmts (s :: acc)
              else List.rev (s :: acc)
          in
          let body = stmts [] in
          L.expect_symbol p '}';
          fs ({ name; params; body } :: acc))
      in
      fs [])

let parse_ainput = Utils.Machine.empty_input
let parse_rinput = Utils.Machine.empty_input

let validate code =
  unique
    (fun s -> "duplicate function: " ^ s)
    (List.map (fun f -> f.name) code.functions)

let function_lookup c name =
  match List.find_opt (fun f -> f.name = name) c.functions with
  | Some f -> f
  | None -> failwith ("unknown function: " ^ name)

let split_body = function
  | [] -> failwith "function body must not be empty"
  | s :: ss -> (Some s, ss)

let bindings frame_id params values =
  List.fold_left
    (fun (env, store) (slot, (p, v)) ->
      let loc = { frame_id; slot } in
      (set p loc env, (loc, v) :: store))
    ([], [])
    (List.mapi (fun i pv -> (i, pv)) (List.combine params values))

let make code () =
  L.protect (fun () ->
      validate code;
      let f =
        match List.find_opt (fun f -> f.name = "main") code.functions with
        | Some f -> f
        | None -> failwith "function 'main' is missing"
      in
      let current_stmt, inner_k = split_body f.body in
      let env, values =
        bindings 0 f.params (List.map (fun _ -> Num 0L) f.params)
      in
      {
        code;
        current_function = f.name;
        current_stmt;
        inner_k;
        env = sorted env;
        store = [ { frame_id = 0; values } ];
        fn_k = [];
        next_frame_id = 1;
      })

let store_lookup s loc =
  let rec loop = function
    | [] -> failwith ("unknown location: " ^ location_text loc)
    | f :: fs -> (
        match List.assoc_opt loc f.values with Some v -> v | None -> loop fs)
  in
  loop s.store

let store_assign s loc value =
  let rec loop = function
    | [] -> failwith ("unknown location: " ^ location_text loc)
    | f :: fs ->
        if List.mem_assoc loc f.values then
          { f with values = set loc value f.values } :: fs
        else f :: loop fs
  in
  { s with store = loop s.store }

let expect_number = function
  | Num n -> n
  | v -> failwith ("expected a number, found " ^ value_text v)

let rec eval_place s = function
  | Var v -> lookup v s.env ("unknown variable: " ^ v)
  | Deref v -> (
      match eval_value s v with
      | Location loc -> loc
      | Null_ptr -> failwith "cannot dereference #null-ptr"
      | Num _ -> failwith "dereference expects a location value")

and eval_value s = function
  | Number n -> Num n
  | NullPtr -> Null_ptr
  | BinOp (a, op, b) ->
      let a = expect_number (eval_value s a)
      and b = expect_number (eval_value s b) in
      Num ((if op = Add then Int64.add else Int64.sub) a b)
  | Load p -> store_lookup s (eval_place s p)
  | Addr p -> Location (eval_place s p)

let format_final_state s f =
  let values indent f =
    String.concat ""
      (List.map
         (fun (l, v) -> indent ^ location_text l ^ " = " ^ value_text v ^ "\n")
         (sorted f.values))
  in
  "halted\ncurrent_function: " ^ s.current_function ^ "\nfinished_frame: "
  ^ string_of_int f.frame_id ^ "\n" ^ values "  " f ^ "remaining_store:\n"
  ^
  if s.store = [] then "  <empty>\n"
  else
    String.concat ""
      (List.map
         (fun f ->
           "  frame " ^ string_of_int f.frame_id ^ ":\n" ^ values "    " f)
         (List.rev s.store))

let step s () =
  L.protect (fun () ->
      let continue next = Utils.Machine.Continue { next; output = "" } in
      match s.current_stmt with
      | None -> (
          match s.inner_k with
          | [] -> failwith "control reached end of function body without return"
          | st :: inner_k -> continue { s with current_stmt = Some st; inner_k }
          )
      | Some (Assign (p, v)) ->
          let loc = eval_place s p in
          let value = eval_value s v in
          continue { (store_assign s loc value) with current_stmt = None }
      | Some (Ifz (c, a, b)) ->
          let v = eval_value s c in
          let selected =
            match v with
            | Num 0L -> a
            | Num _ -> b
            | _ -> failwith ("ifz expects a number, found " ^ value_text v)
          in
          continue { s with current_stmt = Some selected }
      | Some (Call (name, args)) ->
          let values = List.map (eval_value s) args in
          let f = function_lookup s.code name in
          if List.length values <> List.length f.params then
            failwith
              (Printf.sprintf
                 "argument count mismatch for %s: expected %d, got %d" name
                 (List.length f.params) (List.length values));
          let current_stmt, inner_k = split_body f.body in
          let env, values = bindings s.next_frame_id f.params values in
          continue
            {
              s with
              current_function = f.name;
              current_stmt;
              inner_k;
              env = sorted env;
              store = { frame_id = s.next_frame_id; values } :: s.store;
              fn_k =
                {
                  env = s.env;
                  inner_k = s.inner_k;
                  function_name = s.current_function;
                }
                :: s.fn_k;
              next_frame_id = s.next_frame_id + 1;
            }
      | Some Return -> (
          match s.store with
          | [] -> failwith "return on empty store stack"
          | finished :: store -> (
              let s = { s with store } in
              match s.fn_k with
              | [] -> Utils.Machine.Halt (format_final_state s finished)
              | f :: fn_k ->
                  continue
                    {
                      s with
                      current_function = f.function_name;
                      env = f.env;
                      inner_k = f.inner_k;
                      fn_k;
                      current_stmt = None;
                    })))

let snapshot = state_to_yojson

let restore json =
  Result.bind (state_of_yojson json) (fun s ->
      L.protect (fun () ->
          validate s.code;
          s))

let render s =
  let text = Utils.Render.text in
  let table title columns rows =
    Utils.Render.table ~title ~columns:(List.map text columns)
      (List.map (List.map text) rows)
  in
  Utils.Render.state
    [
      table "status" [ "field"; "value" ]
        [
          [ "control"; (if s.current_stmt = None then "next" else "eval") ];
          [ "function"; s.current_function ];
          [
            "current";
            Option.fold ~none:"<none>" ~some:stmt_to_text s.current_stmt;
          ];
          [ "next_frame_id"; string_of_int s.next_frame_id ];
        ];
      table "env"
        [ "var"; "location"; "value" ]
        (List.map
           (fun (n, l) ->
             [
               n;
               location_text l;
               value_text (try store_lookup s l with Failure _ -> Null_ptr);
             ])
           (sorted s.env));
      Utils.Render.container ~title:"store"
        (List.map
           (fun f ->
             table
               ("frame " ^ string_of_int f.frame_id)
               [ "location"; "value" ]
               (List.map
                  (fun (l, v) -> [ location_text l; value_text v ])
                  (sorted f.values)))
           (List.rev s.store));
      Utils.Render.container ~title:"inner_k"
        (List.map (fun st -> text (stmt_to_text st)) s.inner_k);
      table "fn_k"
        [ "depth"; "function"; "resume_len" ]
        (List.mapi
           (fun i f ->
             [
               string_of_int i;
               f.function_name;
               string_of_int (List.length f.inner_k);
             ])
           s.fn_k);
    ]

let routput_json = string_json
let foutput_json = string_json

module Machine = struct
  type nonrec code = code
  type nonrec ainput = ainput
  type nonrec rinput = rinput
  type nonrec routput = routput
  type nonrec foutput = foutput
  type nonrec state = state

  let parse_code = parse_code
  let parse_ainput = parse_ainput
  let parse_rinput = parse_rinput
  let make = make
  let step = step
  let snapshot = snapshot
  let restore = restore
  let render = render
  let routput_json = routput_json
  let foutput_json = foutput_json
end
