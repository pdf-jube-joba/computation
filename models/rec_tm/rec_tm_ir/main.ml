open Utils_runtime.Machine
open Rec_tm_shared.Syntax
open Rec_tm_shared.Environment
open Rec_tm_shared.Eval
module T = Utils_data.Tape.Identifier
module L = Utils_parsing.Parser
module Identifier = Utils_names.Identifier

type lvalue = Rec_tm_shared.Syntax.lvalue =
  | Var of Utils_names.Identifier.t
  | Head
[@@deriving yojson]

type rvalue = Rec_tm_shared.Syntax.rvalue =
  | Var of Utils_names.Identifier.t
  | Head
  | Const of T.sign
[@@deriving yojson]

type condition = Rec_tm_shared.Syntax.condition = {
  left : rvalue;
  right : rvalue;
}
[@@deriving yojson]

type stmt =
  | Lt
  | Rt
  | Assign of lvalue * rvalue
  | Break of condition option
  | Continue of condition option
  | Jump of Utils_names.Identifier.t * condition option
  | Return of condition option
  | Call of Utils_names.Identifier.t
[@@deriving yojson]

type block = { label : Utils_names.Identifier.t; body : stmt list }
[@@deriving yojson]

type func = { name : Utils_names.Identifier.t; blocks : block list }
[@@deriving yojson]

type code = { alphabet : T.sign list; functions : func list }
[@@deriving yojson]

type ainput = T.t
type rinput = unit
type routput = unit
type foutput = T.t

type frame = {
  function_ : Utils_names.Identifier.t;
  env : environment;
  blocks : block list;
  pc : int * int;
}
[@@deriving yojson]

type state = { program : code; frame : frame; stack : frame list; tape : T.t }
[@@deriving yojson]

let stmt_vars = function
  | Assign (dst, src) -> vars_lvalue dst @ vars_rvalue src
  | Break c | Continue c | Jump (_, c) | Return c -> vars_condition c
  | _ -> []

let collect_vars blocks =
  List.sort_uniq Utils_names.Identifier.compare
    (List.concat_map (fun b -> List.concat_map stmt_vars b.body) blocks)

let find_function p name =
  match List.find_opt (fun f -> f.name = name) p.functions with
  | Some f -> f
  | None ->
      failwith
        ("Undefined function '" ^ Utils_names.Identifier.to_string name ^ "'")

let validate_no_recursion p =
  Utils_base.Error.protect (fun () ->
      ignore (find_function p (Utils_names.Identifier.make_exn "main"));
      let visited = ref Identifier.Map.empty in
      let rec visit (path : Utils_names.Identifier.t list)
          (name : Utils_names.Identifier.t) =
        match Identifier.Map.find_opt name !visited with
        | Some `Visited -> ()
        | Some `Visiting ->
            failwith
              ("Recursive call is not allowed: "
              ^ String.concat " -> "
                  (List.map Utils_names.Identifier.to_string (path @ [ name ]))
              )
        | None ->
            visited := Identifier.Map.add name `Visiting !visited;
            let f = find_function p name in
            List.iter
              (fun b ->
                List.iter
                  (function
                    | Call name -> visit (path @ [ f.name ]) name | _ -> ())
                  b.body)
              f.blocks;
            visited := Identifier.Map.add name `Visited !visited
      in
      List.iter (fun f -> visit [] f.name) p.functions)

let validate p =
  let* () = validate_no_recursion p in
  Utils_base.Error.protect (fun () ->
      let allowed = normalized_alphabet p.alphabet in
      Identifier.check_unique
        ~message:(fun name ->
          "Function '" ^ Identifier.to_string name ^ "' is defined twice")
        (List.map (fun f -> f.name) p.functions);
      List.iter
        (fun (f : func) ->
          let labels = List.map (fun b -> b.label) f.blocks in
          Identifier.check_unique
            ~message:(fun label ->
              "label '" ^ Identifier.to_string label ^ "' is duplicated")
            labels;
          let labels = Identifier.Set.of_list labels in
          List.iter
            (fun b ->
              List.iter
                (function
                  | Assign (_, src) -> validate_rvalue allowed src
                  | Break c | Continue c | Return c -> validate_cond allowed c
                  | Jump (label, c) ->
                      validate_cond allowed c;
                      if not (Identifier.Set.mem label labels) then
                        failwith
                          ("jump target '"
                          ^ Utils_names.Identifier.to_string label
                          ^ "' not found")
                  | _ -> ())
                b.body)
            f.blocks)
        p.functions)

(* Calls are represented by names; print dependencies first so original Rust's parser also accepts compiled text. *)
let dependency_order p =
  let seen = ref Identifier.Set.empty and result = ref [] in
  let rec visit f =
    if not (Identifier.Set.mem f.name !seen) then (
      seen := Identifier.Set.add f.name !seen;
      List.iter
        (fun b ->
          List.iter
            (function Call name -> visit (find_function p name) | _ -> ())
            b.body)
        f.blocks;
      result := f :: !result)
  in
  List.iter visit p.functions;
  List.rev !result

module Code = struct
  type t = code

  let parse =
    L.parse (fun p ->
        let alphabet = alphabet p in
        let stmt p =
          if L.bump_if_keyword p "LT" then Lt
          else if L.bump_if_keyword p "RT" then Rt
          else if L.bump_if_keyword p "break" then Break (condition p)
          else if L.bump_if_keyword p "continue" then Continue (condition p)
          else if L.bump_if_keyword p "return" then Return (condition p)
          else if L.bump_if_keyword p "jump" then
            let label = identifier p in
            Jump (label, condition p)
          else if L.bump_if_keyword p "call" then (
            let name = identifier p in
            ignore (L.bump_group_if p L.Parentheses L.expect_eof);
            Call name)
          else
            let dst = lvalue p in
            L.expect p ":=";
            let src = rvalue p in
            Assign (dst, src)
        in
        let block p =
          L.expect_keyword p "label";
          let label = identifier p in
          L.expect_symbol p ':';
          let body = L.group p L.Braces (fun p -> L.many p stmt) in
          { label; body }
        in
        let func p =
          L.expect_keyword p "fn";
          let name = identifier p in
          ignore (L.bump_group_if p L.Parentheses L.expect_eof);
          let blocks = L.group p L.Braces (fun p -> L.many p block) in
          { name; blocks }
        in
        let rec funcs acc =
          if L.eof p then List.rev acc
          else
            let f = func p in
            match List.find_opt (fun old -> old.name = f.name) acc with
            | Some old when old <> f ->
                failwith
                  ("Function '"
                  ^ Utils_names.Identifier.to_string f.name
                  ^ "' is defined twice")
            | Some _ -> funcs acc
            | None -> funcs (f :: acc)
        in
        { alphabet; functions = funcs [] })

  let stmt_print = function
    | Lt -> "\\LT"
    | Rt -> "\\RT"
    | Assign (dst, src) -> lvalue_print dst ^ " := " ^ rvalue_print src
    | Break c -> "\\break" ^ condition_print c
    | Continue c -> "\\continue" ^ condition_print c
    | Return c -> "\\return" ^ condition_print c
    | Jump (label, c) ->
        "\\jump " ^ Utils_names.Identifier.to_string label ^ condition_print c
    | Call name -> "\\call " ^ Utils_names.Identifier.to_string name

  let print p =
    alphabet_print p.alphabet
    ^ String.concat "\n"
        (List.map
           (fun f ->
             "\\fn "
             ^ Utils_names.Identifier.to_string f.name
             ^ "() {\n"
             ^ String.concat ""
                 (List.map
                    (fun b ->
                      "  \\label "
                      ^ Utils_names.Identifier.to_string b.label
                      ^ ": {\n"
                      ^ String.concat ""
                          (List.map
                             (fun s -> "    " ^ stmt_print s ^ "\n")
                             b.body)
                      ^ "  }\n")
                    f.blocks)
             ^ "}\n")
           (dependency_order p))
end

let build_frame f =
  {
    function_ = f.name;
    env = env_create (collect_vars f.blocks);
    blocks = f.blocks;
    pc = (0, 0);
  }

let make program tape =
  let* () = validate program in
  Utils_base.Error.protect (fun () ->
      validate_tape (normalized_alphabet program.alphabet) tape;
      {
        program;
        frame =
          build_frame
            (find_function program (Utils_names.Identifier.make_exn "main"));
        stack = [];
        tape;
      })

let normalized_pc frame =
  let rec loop (b, i) =
    if b >= List.length frame.blocks then (b, i)
    else if i >= List.length (List.nth frame.blocks b).body then loop (b + 1, 0)
    else (b, i)
  in
  loop frame.pc

let peek_stmt s =
  let b, i = normalized_pc s.frame in
  if b >= List.length s.frame.blocks then None
  else Some (List.nth (List.nth s.frame.blocks b).body i)

let step s () =
  Utils_base.Error.protect (fun () ->
      let return s =
        match s.stack with
        | [] -> Halt s.tape
        | frame :: stack ->
            Utils_runtime.Machine.Continue
              { next = { s with frame; stack }; output = () }
      in
      let b, i = normalized_pc s.frame in
      if b >= List.length s.frame.blocks then return s
      else
        let stmt = List.nth (List.nth s.frame.blocks b).body i in
        let next = { s with frame = { s.frame with pc = (b, i + 1) } } in
        let cond c = eval_cond next.frame.env next.tape c in
        let continue next =
          Utils_runtime.Machine.Continue { next; output = () }
        in
        let pc pc = continue { next with frame = { next.frame with pc } } in
        match stmt with
        | Lt -> continue { next with tape = T.move_to next.tape T.Left }
        | Rt -> continue { next with tape = T.move_to next.tape T.Right }
        | Assign (dst, src) ->
            let env, tape =
              assign next.frame.env next.tape dst
                (eval next.frame.env next.tape src)
            in
            continue { next with tape; frame = { next.frame with env } }
        | Break c when cond c -> pc (b + 1, 0)
        | Continue c when cond c -> pc (b, 0)
        | Jump (label, c) when cond c ->
            let rec find idx = function
              | [] ->
                  failwith
                    ("jump target '"
                    ^ Utils_names.Identifier.to_string label
                    ^ "' not found")
              | b :: xs -> if b.label = label then idx else find (idx + 1) xs
            in
            pc (find 0 next.frame.blocks, 0)
        | Return c when cond c -> return next
        | Call name ->
            continue
              {
                next with
                frame = build_frame (find_function next.program name);
                stack = next.frame :: next.stack;
              }
        | _ -> continue next)

module Snapshot = struct
  type t = state

  let to_yojson = state_to_yojson

  let of_yojson json =
    let* s = state_of_yojson json in
    let* () = validate s.program in
    Utils_base.Error.protect (fun () ->
        validate_tape (normalized_alphabet s.program.alphabet) s.tape;
        List.iter
          (fun f ->
            let b, i = f.pc in
            if b < 0 || i < 0 then failwith "Invalid frame pc";
            let definition = find_function s.program f.function_ in
            if f.blocks <> definition.blocks then
              failwith "Frame differs from function definition";
            validate_environment (collect_vars f.blocks) f.env)
          (s.frame :: s.stack);
        s)
end

let render s =
  let b, i = normalized_pc s.frame in
  let signs, head = T.into_vec s.tape in
  [
    Utils_runtime.Render.text ~title:"function"
      (Utils_names.Identifier.to_string s.frame.function_);
    Utils_runtime.Render.text ~title:"pc" (Printf.sprintf "%d:%d" b i);
    Utils_runtime.Render.text ~title:"next"
      (Option.fold ~none:"halt" ~some:Code.stmt_print (peek_stmt s));
    Utils_runtime.Render.highlight ~title:"stack" ~index:(List.length s.stack)
      (fun f -> Utils_names.Identifier.to_string f.function_)
      (List.rev s.stack @ [ s.frame ]);
    Utils_runtime.Render.keyed_table ~key_text:Utils_names.Identifier.to_string
      ~title:"env" T.Sign.print (env_named s.frame.env);
    Utils_runtime.Render.highlight ~title:"tape" ~index:head T.Sign.print signs;
  ]

module Ainput = Rec_tm_shared.Syntax.Tape
module Rinput = Utils_data.Text.Unit
module Routput = Utils_data.Text.Unit
module Foutput = Ainput
