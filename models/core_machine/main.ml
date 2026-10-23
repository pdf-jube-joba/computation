include Types
module Code = Code
module Ainput = Input_text
module Foutput = Input_text
module Rinput = Utils_data.Text.String
module Routput = Utils_data.Text.String
module E = Utils_base.Error

let fail s = failwith s
let rec origin = function Whole m -> m | Subslice (s, _, _) -> origin s

let rec base = function
  | Whole _ -> N.zero
  | Subslice (s, b, _) -> N.add (base s) b

let length = function Whole _ -> None | Subslice (_, b, e) -> Some (N.sub e b)

let within s i =
  match length s with None -> true | Some n -> N.compare i n <= 0

let readable s i =
  match length s with None -> true | Some n -> N.compare i n < 0

let rec validate_slice code = function
  | Whole m -> if m < 0 || m >= code.memories then fail "Unknown memory"
  | Subslice (s, b, e) ->
      validate_slice code s;
      if N.compare b e > 0 || not (within s e) then fail "Slice out of bounds"

let known_label code l = List.exists (fun b -> b.label = l) code.blocks

let validate_value code = function
  | Num _ -> ()
  | Code_label l -> if not (known_label code l) then fail "Unknown code label"
  | Slice_value s -> validate_slice code s
  | Address (s, i) ->
      validate_slice code s;
      if not (within s i) then fail "Address out of bounds"

let unique keys =
  if List.length (List.sort_uniq Stdlib.compare keys) <> List.length keys then
    fail "Duplicate key"

let validate_code code =
  if code.memories < 1 then fail "At least one memory is required";
  unique (List.map (fun b -> b.label) code.blocks);
  if not (known_label code (Id.make_exn "main")) then
    fail "Missing entry block :main";
  let value = function
    | Label l -> validate_value code (Code_label l)
    | Memory m -> validate_slice code (Whole m)
    | Reg _ | Imm _ -> ()
  in
  let place (Deref v) = value v in
  let target v =
    match v with Reg _ | Label _ -> value v | _ -> fail "Invalid jump target"
  in
  List.iter
    (fun b ->
      List.iter
        (function
          | Print v -> value v
          | Input p -> place p
          | Store (p, v) ->
              place p;
              value v
          | Assign (_, rhs) -> (
              match rhs with
              | Value v | Len v -> value v
              | Load p -> place p
              | Binop (a, _, b) | Addr (a, b) ->
                  value a;
                  value b
              | Slice (s, b, e) ->
                  value s;
                  value b;
                  value e))
        b.stmts;
      List.iter
        (fun j ->
          let a, _, b = j.cond in
          value a;
          value b;
          target j.target)
        b.ifs;
      match b.ending with Halt -> () | Goto v -> target v)
    code.blocks

let validate_memory code mem =
  unique (List.map fst mem);
  List.iter (fun (_, v) -> validate_value code v) mem

let normalize mem =
  List.filter (function _, Num n when N.is_zero n -> false | _ -> true) mem
  |> List.sort (fun (a, _) (b, _) -> N.compare a b)

let make code input =
  E.protect (fun () ->
      validate_code code;
      validate_memory code input;
      {
        code;
        block = Id.make_exn "main";
        line = 0;
        registers = [];
        memory =
          Array.init code.memories (fun m ->
              if m = 0 then normalize input else []);
      })

let eval s = function
  | Reg r -> Utils_base.Assoc.get ~default:(Num N.zero) r s.registers
  | Imm n -> Num n
  | Label l -> Code_label l
  | Memory m -> Slice_value (Whole m)

let num = function Num n -> n | _ -> fail "Expected natural number"
let slice = function Slice_value s -> s | _ -> fail "Expected slice"
let address = function Address (s, i) -> (s, i) | _ -> fail "Expected address"
let pos (s, i) = N.add (base s) i

let same_origin (s, _) (t, _) =
  if origin s <> origin t then fail "Different address origins"

let binop op a b =
  match (op, a, b) with
  | Add, Num x, Num y -> Num (N.add x y)
  | Sub, Num x, Num y -> Num (N.sub x y)
  | Add, Address (s, i), Num n ->
      let j = N.add i n in
      if not (within s j) then fail "Address out of bounds";
      Address (s, j)
  | Sub, Address (s, i), Num n ->
      if N.compare n i > 0 then fail "Address out of bounds";
      Address (s, N.sub i n)
  | Sub, Address (s, i), Address (t, j) ->
      same_origin (s, i) (t, j);
      Num (N.sub (pos (s, i)) (pos (t, j)))
  | _ -> fail "Invalid binary operand types"

let place s (Deref v) =
  let t, i = address (eval s v) in
  if not (readable t i) then fail "Cannot access slice end";
  (origin t, pos (t, i))

let memory s m = s.memory.(m)
let load s (m, i) = Utils_base.Assoc.get ~default:(Num N.zero) i (memory s m)

let store s (m, i) v =
  let cells = normalize (Utils_base.Assoc.set i v (memory s m)) in
  let memory = Array.copy s.memory in
  memory.(m) <- cells;
  { s with memory }

let rhs s = function
  | Value v -> eval s v
  | Binop (a, op, b) -> binop op (eval s a) (eval s b)
  | Load p -> load s (place s p)
  | Slice (v, b, e) ->
      let t = slice (eval s v) in
      let t = Subslice (t, num (eval s b), num (eval s e)) in
      validate_slice s.code t;
      Slice_value t
  | Addr (v, i) ->
      let t = slice (eval s v) in
      let i = num (eval s i) in
      if not (within t i) then fail "Address out of bounds";
      Address (t, i)
  | Len v -> (
      match length (slice (eval s v)) with
      | Some n -> Num n
      | None -> fail "Infinite slice has no natural length")

let cond s (a, op, b) =
  let a = eval s a and b = eval s b in
  let cmp =
    match (a, b) with
    | Num x, Num y -> N.compare x y
    | Code_label x, Code_label y when op = Eq -> if x = y then 0 else 1
    | Slice_value x, Slice_value y when op = Eq -> if x = y then 0 else 1
    | Address (t, i), Address (u, j) ->
        if op = Eq && origin t <> origin u then 1
        else (
          same_origin (t, i) (u, j);
          N.compare (pos (t, i)) (pos (u, j)))
    | _ -> fail "Invalid comparison operand types"
  in
  match op with Eq -> cmp = 0 | Lt -> cmp < 0 | Gt -> cmp > 0

let jump s v =
  match eval s v with
  | Code_label l when known_label s.code l -> { s with block = l; line = 0 }
  | _ -> fail "Jump target is not a known code label"

let step s input =
  E.protect (fun () ->
      let b =
        match List.find_opt (fun b -> b.label = s.block) s.code.blocks with
        | Some b -> b
        | None -> fail "Unknown current block"
      in
      let ns = List.length b.stmts and ni = List.length b.ifs in
      let continue next =
        Utils_runtime.Machine.Continue { next; output = "" }
      in
      if s.line < 0 then fail "Invalid instruction position"
      else if s.line < ns then
        let next, output =
          match List.nth b.stmts s.line with
          | Print v ->
              let bytes = N.to_bytes (num (eval s v)) in
              if not (Utils_base.Utf8.valid bytes) then
                fail "print expects valid UTF-8";
              (s, bytes)
          | Input p -> (store s (place s p) (Num (N.of_bytes input)), "")
          | Assign (r, e) ->
              let v = rhs s e in
              ({ s with registers = Utils_base.Assoc.set r v s.registers }, "")
          | Store (p, v) ->
              let dst = place s p in
              let v = eval s v in
              (store s dst v, "")
        in
        Utils_runtime.Machine.Continue
          { next = { next with line = s.line + 1 }; output }
      else if s.line < ns + ni then
        let j = List.nth b.ifs (s.line - ns) in
        continue
          (if cond s j.cond then jump s j.target
           else { s with line = s.line + 1 })
      else if s.line = ns + ni then
        match b.ending with
        | Halt -> Utils_runtime.Machine.Halt (normalize (memory s 0))
        | Goto v -> continue (jump s v)
      else fail "Invalid instruction position")

module Snapshot = struct
  type t = state

  let to_yojson = state_to_yojson

  let of_yojson json =
    Result.bind (state_of_yojson json) (fun s ->
        E.protect (fun () ->
            validate_code s.code;
            let b =
              match
                List.find_opt (fun b -> b.label = s.block) s.code.blocks
              with
              | Some b -> b
              | None -> fail "Unknown current block"
            in
            if s.line < 0 || s.line > List.length b.stmts + List.length b.ifs
            then fail "Invalid instruction position";
            unique (List.map fst s.registers);
            List.iter (fun (_, v) -> validate_value s.code v) s.registers;
            if Array.length s.memory <> s.code.memories then
              fail "Memory count does not match declaration";
            Array.iter (validate_memory s.code) s.memory;
            s))
end

let render s =
  let text = Utils_runtime.Render.text_table in
  [
    text ~title:"status" ~columns:[ "field"; "value" ]
      [
        [ "block"; ":" ^ Id.to_string s.block ];
        [ "line"; string_of_int s.line ];
      ];
    text ~title:"registers" ~columns:[ "register"; "value" ]
      (List.map
         (fun (r, v) -> [ "%" ^ Id.to_string r; Input_text.value_text v ])
         s.registers);
    text ~title:"memory (nonzero cells)"
      ~columns:[ "memory"; "address"; "value" ]
      (Array.mapi
         (fun m cells ->
           List.map
             (fun (i, v) ->
               [
                 "MEM_" ^ string_of_int m;
                 N.to_string i;
                 Input_text.value_text v;
               ])
             cells)
         s.memory
      |> Array.to_list |> List.concat);
  ]
