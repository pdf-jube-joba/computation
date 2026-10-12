open Utils.Machine
open Rec_tm_shared.Syntax
module T = Turing_machine.Main
module L = Utils.Parser

type lvalue = Rec_tm_shared.Syntax.lvalue = Var of Utils.Identifier.t | Head
[@@deriving yojson]

type rvalue = Rec_tm_shared.Syntax.rvalue =
  | Var of Utils.Identifier.t
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
  | Jump of Utils.Identifier.t * condition option
  | Return of condition option
  | Call of Utils.Identifier.t
[@@deriving yojson]

type block = { label : Utils.Identifier.t; body : stmt list }
[@@deriving yojson]

type func = { name : Utils.Identifier.t; blocks : block list }
[@@deriving yojson]

type code = { alphabet : T.sign list; functions : func list }
[@@deriving yojson]

type ainput = T.tape
type rinput = unit
type routput = unit
type foutput = T.tape

type frame = {
  function_ : Utils.Identifier.t;
  env : environment;
  blocks : block list;
  pc : int * int;
}
[@@deriving yojson]

type state = {
  program : code;
  frame : frame;
  stack : frame list;
  tape : T.tape;
}
[@@deriving yojson]

let stmt_vars = function
  | Assign (dst, src) -> vars_lvalue dst @ vars_rvalue src
  | Break c | Continue c | Jump (_, c) | Return c -> vars_condition c
  | _ -> []

let collect_vars blocks =
  List.sort_uniq String.compare
    (List.concat_map (fun b -> List.concat_map stmt_vars b.body) blocks)

let find_function p name =
  match List.find_opt (fun f -> f.name = name) p.functions with
  | Some f -> f
  | None -> failwith ("Undefined function '" ^ name ^ "'")

let validate_no_recursion p =
  L.protect (fun () ->
      ignore (find_function p "main");
      let visited = Hashtbl.create 32 in
      let rec visit path name =
        match Hashtbl.find_opt visited name with
        | Some 2 -> ()
        | Some 1 ->
            failwith
              ("Recursive call is not allowed: "
              ^ String.concat " -> " (path @ [ name ]))
        | _ ->
            Hashtbl.replace visited name 1;
            let f = find_function p name in
            List.iter
              (fun b ->
                List.iter
                  (function
                    | Call name -> visit (path @ [ f.name ]) name | _ -> ())
                  b.body)
              f.blocks;
            Hashtbl.replace visited name 2
      in
      List.iter (fun f -> visit [] f.name) p.functions)

let validate p =
  let* () = validate_no_recursion p in
  L.protect (fun () ->
      let allowed = normalized_alphabet p.alphabet in
      let names = Hashtbl.create 16 in
      List.iter
        (fun f ->
          if Hashtbl.mem names f.name then
            failwith ("Function '" ^ f.name ^ "' is defined twice");
          Hashtbl.add names f.name ();
          let labels = Hashtbl.create 16 in
          List.iter
            (fun b ->
              if Hashtbl.mem labels b.label then
                failwith ("label '" ^ b.label ^ "' is duplicated");
              Hashtbl.add labels b.label ())
            f.blocks;
          List.iter
            (fun b ->
              List.iter
                (function
                  | Assign (_, src) -> validate_rvalue allowed src
                  | Break c | Continue c | Return c -> validate_cond allowed c
                  | Jump (label, c) ->
                      validate_cond allowed c;
                      if not (Hashtbl.mem labels label) then
                        failwith ("jump target '" ^ label ^ "' not found")
                  | _ -> ())
                b.body)
            f.blocks)
        p.functions)

(* Calls are represented by names; print dependencies first so original Rust's parser also accepts compiled text. *)
let dependency_order p =
  let seen = Hashtbl.create 32 and result = ref [] in
  let rec visit f =
    if not (Hashtbl.mem seen f.name) then (
      Hashtbl.add seen f.name ();
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
            ignore (L.bump_group_if p ~opening:'(' L.expect_eof);
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
          let body = L.group p ~opening:'{' (fun p -> L.many p stmt) in
          { label; body }
        in
        let func p =
          L.expect_keyword p "fn";
          let name = identifier p in
          ignore (L.bump_group_if p ~opening:'(' L.expect_eof);
          let blocks = L.group p ~opening:'{' (fun p -> L.many p block) in
          { name; blocks }
        in
        let rec funcs acc =
          if L.eof p then List.rev acc
          else
            let f = func p in
            match List.find_opt (fun old -> old.name = f.name) acc with
            | Some old when old <> f ->
                failwith ("Function '" ^ f.name ^ "' is defined twice")
            | Some _ -> funcs acc
            | None -> funcs (f :: acc)
        in
        { alphabet; functions = funcs [] })

  let stmt_print = function
    | Lt -> "LT"
    | Rt -> "RT"
    | Assign (dst, src) -> lvalue_print dst ^ " := " ^ rvalue_print src
    | Break c -> "break" ^ condition_print c
    | Continue c -> "continue" ^ condition_print c
    | Return c -> "return" ^ condition_print c
    | Jump (label, c) -> "jump " ^ label ^ condition_print c
    | Call name -> "call " ^ name

  let print p =
    alphabet_print p.alphabet
    ^ String.concat "\n"
        (List.map
           (fun f ->
             "fn " ^ f.name ^ "() {\n"
             ^ String.concat ""
                 (List.map
                    (fun b ->
                      "  label " ^ b.label ^ ": {\n"
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
  L.protect (fun () ->
      validate_tape (normalized_alphabet program.alphabet) tape;
      {
        program;
        frame = build_frame (find_function program "main");
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
  L.protect (fun () ->
      let return s =
        match s.stack with
        | [] -> Halt s.tape
        | frame :: stack ->
            Utils.Machine.Continue
              { next = { s with frame; stack }; output = () }
      in
      let b, i = normalized_pc s.frame in
      if b >= List.length s.frame.blocks then return s
      else
        let stmt = List.nth (List.nth s.frame.blocks b).body i in
        let next = { s with frame = { s.frame with pc = (b, i + 1) } } in
        let cond c = eval_cond next.frame.env next.tape c in
        let continue next = Utils.Machine.Continue { next; output = () } in
        let pc pc = continue { next with frame = { next.frame with pc } } in
        match stmt with
        | Lt -> continue { next with tape = T.Tape.move_to next.tape T.Left }
        | Rt -> continue { next with tape = T.Tape.move_to next.tape T.Right }
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
              | [] -> failwith ("jump target '" ^ label ^ "' not found")
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

let snapshot = state_to_yojson

let restore json =
  let* s = state_of_yojson json in
  let* () = validate s.program in
  L.protect (fun () ->
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

let render s =
  let b, i = normalized_pc s.frame in
  Utils.Render.state
    [
      Utils.Render.text ~title:"function" s.frame.function_;
      Utils.Render.text ~title:"pc" (Printf.sprintf "%d:%d" b i);
      Utils.Render.text ~title:"next"
        (Option.fold ~none:"halt" ~some:Code.stmt_print (peek_stmt s));
      Utils.Render.container ~title:"stack" ~orientation:"horizontal"
        (List.map (fun f -> Utils.Render.text f.function_) (List.rev s.stack)
        @ [ Utils.Render.text ~class_name:"highlight" s.frame.function_ ]);
      Utils.Render.assoc_table ~title:"env" T.Sign.print (env_named s.frame.env);
      T.Tape.render s.tape;
    ]

module Ainput = Rec_tm_shared.Syntax.Tape
module Rinput = Utils.Text.Unit
module Routput = Utils.Text.Unit
module Foutput = Ainput
