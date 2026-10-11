open Utils.Machine
module Source = Recursive_function
module Target = Goto_lang

let output_var = "y"
let always_one_var = "__one"
let input_vars n = List.init n (fun i -> "x" ^ string_of_int i)

(* Labels refer to command positions, including the position just past the end.
   Resolve them only after every nested function has emitted its instructions. *)
type unresolved = Command of Target.command | Branch of string * int

type builder = {
  mutable commands : unresolved list;
  mutable position : int;
  labels : (int, int) Hashtbl.t;
  mutable next_label : int;
  mutable next_temp : int;
}

let builder () =
  {
    commands = [];
    position = 0;
    labels = Hashtbl.create 16;
    next_label = 0;
    next_temp = 0;
  }

let label b =
  let result = b.next_label in
  b.next_label <- result + 1;
  result

let mark b label = Hashtbl.replace b.labels label b.position

let temp b =
  let result = "__tmp" ^ string_of_int b.next_temp in
  b.next_temp <- b.next_temp + 1;
  result

let emit b command =
  b.commands <- command :: b.commands;
  b.position <- b.position + 1

let clr b var = emit b (Command (Target.Clr var))
let inc b var = emit b (Command (Target.Inc var))
let dec b var = emit b (Command (Target.Dec var))
let cpy b dst src = emit b (Command (Target.Cpy (dst, src)))
let ifnz b var label = emit b (Branch (var, label))
let goto b label = ifnz b always_one_var label

let rec compile_into b func args dst =
  match func with
  | Source.Zero -> clr b dst
  | Source.Succ ->
      cpy b dst (List.hd args);
      inc b dst
  | Source.Proj (_, i) -> cpy b dst (List.nth args i)
  | Source.Comp (outer, inners) ->
      let outer_args =
        List.map
          (fun inner ->
            let tmp = temp b in
            compile_into b inner args tmp;
            tmp)
          inners
      in
      compile_into b outer outer_args dst
  | Source.Prim (zero, succ) -> compile_primitive b zero succ args dst
  | Source.Mu func -> compile_mu b func args dst

and compile_primitive b zero succ args dst =
  let first = List.hd args and rest = List.tl args in
  let result = temp b in
  let counter = temp b in
  let index = temp b in
  let succ_value = temp b in
  compile_into b zero rest result;
  cpy b counter first;
  clr b index;
  let condition = label b in
  let body = label b in
  let finish = label b in
  mark b condition;
  ifnz b counter body;
  goto b finish;
  mark b body;
  (* The step takes (previous result, predecessor index, remaining inputs). *)
  compile_into b succ (result :: index :: rest) succ_value;
  cpy b result succ_value;
  inc b index;
  dec b counter;
  goto b condition;
  mark b finish;
  cpy b dst result

and compile_mu b func args dst =
  let index = temp b in
  let value = temp b in
  clr b index;
  let start = label b in
  let continue = label b in
  let finish = label b in
  mark b start;
  compile_into b func (index :: args) value;
  ifnz b value continue;
  cpy b dst index;
  goto b finish;
  mark b continue;
  inc b index;
  goto b start;
  mark b finish

let finalize b =
  let rec resolve = function
    | [] -> Ok []
    | command :: rest ->
        let* command =
          match command with
          | Command command -> Ok command
          | Branch (var, label) -> (
              match Hashtbl.find_opt b.labels label with
              | Some target ->
                  Ok (Target.Ifnz (var, Utils.Number.of_int target))
              | None -> Error ("undefined label: " ^ string_of_int label))
        in
        let* rest = resolve rest in
        Ok (command :: rest)
  in
  resolve (List.rev b.commands)

let compile func =
  let* () = Source.validate func in
  let b = builder () in
  clr b always_one_var;
  inc b always_one_var;
  compile_into b func (input_vars (Source.arity func)) output_var;
  finalize b

let encode_ainput input =
  Ok (List.mapi (fun i value -> ("x" ^ string_of_int i, value)) input)

let encode_rinput () = Ok ()
let decode_routput () = Ok ()

let decode_foutput output =
  Ok
    (Option.value ~default:Utils.Number.zero (List.assoc_opt output_var output))

let api =
  Utils.Compiler.make_api
    (module struct
      module Source = Source
      module Target = Target

      let compile = compile
      let encode_ainput = encode_ainput
      let encode_rinput = encode_rinput
      let decode_routput = decode_routput
      let decode_foutput = decode_foutput
    end)
