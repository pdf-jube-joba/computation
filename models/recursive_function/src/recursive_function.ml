open Utils
open Machine

type func =
  | Zero
  | Succ
  | Proj of int * int
  | Comp of func * func list
  | Prim of func * func
  | Mu of func
[@@deriving yojson]

type process =
  | Call of func * process list
  | Search of Number.t * Number.t list * func * process
  | Result of Number.t
[@@deriving yojson]

type code = func [@@deriving yojson]
type ainput = Number.t list [@@deriving yojson]
type rinput = unit
type routput = unit
type foutput = Number.t

type state = { function_ : func; input : ainput; process : process }
[@@deriving yojson]

let rec arity = function
  | Zero -> 0
  | Succ -> 1
  | Proj (n, _) -> n
  | Comp (_, head :: _) -> arity head
  | Comp (_, []) -> 0
  | Prim (z, _) -> arity z + 1
  | Mu f -> arity f - 1

let rec validate = function
  | Zero | Succ -> Ok ()
  | Proj (n, i) ->
      if n > i && i >= 0 then Ok () else Error "Projection out of range"
  | Comp (f, args) ->
      let* () = validate f in
      let rec all = function
        | [] -> Ok ()
        | h :: t ->
            let* () = validate h in
            all t
      in
      let* () = all args in
      if
        args = []
        || List.length args <> arity f
        || not (List.for_all (fun a -> arity a = arity (List.hd args)) args)
      then Error "Composition arity mismatch"
      else Ok ()
  | Prim (z, s) ->
      let* () = validate z in
      let* () = validate s in
      if arity s = arity z + 2 then Ok ()
      else Error "Primitive recursion arity mismatch"
  | Mu f ->
      let* () = validate f in
      if arity f > 0 then Ok ()
      else Error "Mu requires a function of positive arity"

let parse_code text =
  let* f =
    Lexer.parse
      (fun p ->
        let bindings = ref [] in
        let rec func () =
          match Lexer.identifier p with
          | "ZERO" -> Zero
          | "SUCC" -> Succ
          | "PROJ" ->
              Lexer.expect_symbol p '[';
              let n = Lexer.integer p in
              Lexer.expect_symbol p ',';
              let i = Lexer.integer p in
              Lexer.expect_symbol p ']';
              Proj (n, i)
          | "COMP" ->
              Lexer.expect_symbol p '[';
              let outer = func () in
              Lexer.expect_symbol p ':';
              let rec args () =
                if Lexer.eat_symbol p '(' then (
                  let xs = items ')' in
                  Lexer.expect_symbol p ')';
                  ignore (Lexer.eat_symbol p ',');
                  xs @ items ']')
                else
                  let f = func () in
                  if Lexer.eat_symbol p ',' then f :: items ']' else [ f ]
              and items closing =
                while Lexer.eat_symbol p ',' do
                  ()
                done;
                if Lexer.peek p = Lexer.Symbol closing then []
                else
                  let f = func () in
                  if Lexer.eat_symbol p ',' then f :: items closing else [ f ]
              in
              let inner = args () in
              Lexer.expect_symbol p ']';
              Comp (outer, inner)
          | "PRIM" ->
              Lexer.expect_symbol p '[';
              Lexer.expect p "z:";
              let z = func () in
              Lexer.expect p "s:";
              let s = func () in
              Lexer.expect_symbol p ']';
              Prim (z, s)
          | "MUOP" ->
              Lexer.expect_symbol p '[';
              let f = func () in
              Lexer.expect_symbol p ']';
              Mu f
          | name -> (
              match List.assoc_opt name !bindings with
              | Some f -> f
              | None -> Lexer.error p ("Unknown function: " ^ name))
        in
        while Lexer.eat_keyword p "let" do
          let name = Lexer.identifier p in
          if List.mem_assoc name !bindings then
            Lexer.error p "Duplicate function name";
          Lexer.expect_symbol p '=';
          let f = func () in
          Lexer.expect_symbol p '.';
          bindings := (name, f) :: !bindings
        done;
        func ())
      text
  in
  let* () = validate f in
  Ok f

let parse_ainput =
  Lexer.parse (fun p ->
      Lexer.expect_symbol p '(';
      let rec items () =
        if Lexer.peek p = Lexer.Symbol ')' then []
        else
          let value = Lexer.natural p in
          if Lexer.eat_symbol p ',' then value :: items () else [ value ]
      in
      let xs = items () in
      Lexer.expect_symbol p ')';
      xs)

let parse_rinput = empty_input

let make function_ input =
  let* () = validate function_ in
  if List.length input <> arity function_ then
    Error "Function argument count mismatch"
  else
    Ok
      {
        function_;
        input;
        process = Call (function_, List.map (fun n -> Result n) input);
      }

let rec evaluate_one = function
  | Result _ -> None
  | Call (f, args) -> (
      let rec first prefix = function
        | [] -> None
        | head :: tail -> (
            match evaluate_one head with
            | Some next -> Some (List.rev_append prefix (next :: tail))
            | None -> first (head :: prefix) tail)
      in
      match first [] args with
      | Some args -> Some (Call (f, args))
      | None ->
          let numbers =
            List.map (function Result n -> n | _ -> assert false) args
          in
          let results xs = List.map (fun n -> Result n) xs in
          Some
            (match (f, numbers) with
            | Zero, [] -> Result Number.zero
            | Succ, [ n ] -> Result (Number.succ n)
            | Proj (_, i), _ -> Result (List.nth numbers i)
            | Comp (outer, inners), _ ->
                Call (outer, List.map (fun inner -> Call (inner, args)) inners)
            | Prim (z, s), n :: tail ->
                if Number.is_zero n then Call (z, results tail)
                else
                  let predecessor = Number.pred n in
                  Call
                    ( s,
                      Call (f, results (predecessor :: tail))
                      :: results (predecessor :: tail) )
            | Mu f, _ ->
                Search
                  ( Number.zero,
                    numbers,
                    f,
                    Call (f, results (Number.zero :: numbers)) )
            | _ -> failwith "Invalid recursive-function process"))
  | Search (index, args, f, process) -> (
      match evaluate_one process with
      | Some next -> Some (Search (index, args, f, next))
      | None -> (
          match process with
          | Result n when Number.is_zero n -> Some (Result index)
          | Result _ ->
              let index = Number.succ index in
              Some
                (Search
                   ( index,
                     args,
                     f,
                     Call (f, List.map (fun n -> Result n) (index :: args)) ))
          | _ -> assert false))

let step state () =
  match evaluate_one state.process with
  | Some process -> Ok (Continue { next = { state with process }; output = () })
  | None -> (
      match state.process with
      | Result output -> Ok (Halt output)
      | _ -> Error "Invalid recursive-function process")

let snapshot = state_to_yojson

let restore json =
  let* state = state_of_yojson json in
  let* () = validate state.function_ in
  if List.length state.input <> arity state.function_ then
    Error "Invalid snapshot input arity"
  else Ok state

let rec print = function
  | Zero -> "ZERO"
  | Succ -> "SUCC"
  | Proj (n, i) -> Printf.sprintf "PROJ[%d,%d]" n i
  | Comp (f, args) ->
      "COMP[" ^ print f ^ ": " ^ String.concat ", " (List.map print args) ^ "]"
  | Prim (z, s) -> "PRIM[z:" ^ print z ^ " s:" ^ print s ^ "]"
  | Mu f -> "MUOP[" ^ print f ^ "]"

let rec print_process = function
  | Result n -> n
  | Call (f, args) ->
      print f ^ "(" ^ String.concat ", " (List.map print_process args) ^ ")"
  | Search (n, _, _, p) -> "Mu[" ^ n ^ "](" ^ print_process p ^ ")"

let render state =
  Render.state
    [
      Render.text ~title:"function" (print state.function_);
      Render.table ~title:"input"
        ~columns:[ Render.text "arg"; Render.text "value" ]
        (List.mapi
           (fun index value ->
             [ Render.text (string_of_int index); Render.text value ])
           state.input);
      Render.text ~title:"process" (print_process state.process);
    ]

let routput_json () = `String ""
let foutput_json n = `String n

let rec evaluate ?(fuel = 100000) process =
  if fuel = 0 then Error "Evaluation step limit reached"
  else
    match process with
    | Result n -> Ok n
    | _ -> (
        match evaluate_one process with
        | Some p -> evaluate ~fuel:(fuel - 1) p
        | None -> Error "Invalid process")

(* Direct evaluation, like the native interpreter in the former implementation.
   The step API and [evaluate ~fuel] remain available for bounded execution. *)
let interpreter function_ =
  let* () = validate function_ in
  Ok
    (fun input ->
      let* state = make function_ input in
      let rec run = function
        | Result n -> Ok n
        | process -> (
            match evaluate_one process with
            | Some next -> run next
            | None -> Error "Invalid recursive-function process")
      in
      run state.process)
