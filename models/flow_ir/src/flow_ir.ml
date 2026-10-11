module N = Utils.Number
module L = Utils.Lexer

let get = function Ok x -> x | Error e -> failwith e

type binop = Add | Sub [@@deriving yojson]

type value_expr =
  | VReg of Utils.Identifier.t
  | Imm of N.t
  | CodeLabel of Utils.Identifier.t
  | Ref of place_expr

and place_expr =
  | Label of Utils.Identifier.t
  | Deref of Utils.Identifier.t
  | SAcc of value_expr
  | HAcc of value_expr * value_expr
[@@deriving yojson]

type cond =
  | Lt of value_expr * value_expr
  | Eq of value_expr * value_expr
  | Gt of value_expr * value_expr
[@@deriving yojson]

type stmt =
  | Nop
  | Load of Utils.Identifier.t * place_expr
  | Assign of Utils.Identifier.t * value_expr
  | BinOp of Utils.Identifier.t * value_expr * binop * value_expr
  | Store of place_expr * value_expr
  | Pop of Utils.Identifier.t
  | Push of value_expr
  | LGet of Utils.Identifier.t
  | HAlloc of value_expr * Utils.Identifier.t
  | HFree of value_expr
  | Print of Utils.Identifier.t
  | Input of place_expr
[@@deriving yojson]

type jump_if = { cond : cond; target : value_expr } [@@deriving yojson]

type cont =
  | Go of jump_if list * value_expr
  | Enter of jump_if list * value_expr
  | Halt of jump_if list
[@@deriving yojson]

type block = { label : Utils.Identifier.t; stmts : stmt list; cont : cont }
[@@deriving yojson]

type region = { label : Utils.Identifier.t; blocks : block list }
[@@deriving yojson]

type static_def = { label : Utils.Identifier.t; value : N.t }
[@@deriving yojson]

type code = { statics : static_def list; regions : region list }
[@@deriving yojson]

type place_key =
  | Static of Utils.Identifier.t
  | Stack of int
  | Heap of int * int
[@@deriving yojson]

type flow_value =
  | Num of N.t
  | Code_label of Utils.Identifier.t
  | Key of place_key
[@@deriving yojson]

type ainput = (Utils.Identifier.t * flow_value) list [@@deriving yojson]
type foutput = ainput [@@deriving yojson]
type rinput = string
type routput = string

type state = {
  code : code;
  current_region : int;
  current_block : int;
  current_line : int;
  vregs : (Utils.Identifier.t * flow_value) list;
  static_mem : ainput;
  stack : flow_value list;
  heap : (int * flow_value list) list;
  next_handle : int;
  halted : bool;
}
[@@deriving yojson]

module Ainput = struct
  type t = ainput

  let print_value = function
    | Num n -> N.to_string n
    | Code_label l -> ":" ^ l
    | Key (Static l) -> "k:@" ^ l
    | Key (Stack i) -> Printf.sprintf "k:s[%d]" i
    | Key (Heap (h, i)) -> Printf.sprintf "k:h[%d][%d]" h i

  let parse_value text =
    L.protect (fun () ->
        let s = String.trim text in
        let prefix p = String.starts_with ~prefix:p s in
        if prefix ":" then
          let label = Utils.String_ext.strip_prefix ":" s in
          Code_label (get (Utils.Identifier.make label))
        else if prefix "k:@" then
          let label = Utils.String_ext.strip_prefix "k:@" s in
          Key (Static (get (Utils.Identifier.make label)))
        else if prefix "k:s[" then
          let body = Utils.String_ext.strip_prefix "k:s[" s in
          if not (String.ends_with ~suffix:"]" body) then
            failwith ("Invalid stack key: " ^ s)
          else
            let idx = Utils.String_ext.strip_suffix "]" body in
            Key (Stack (get (N.to_int (get (N.of_string idx)))))
        else if prefix "k:h[" then (
          let body = Utils.String_ext.strip_prefix "k:h[" s in
          let p = L.create body in
          let handle = L.integer p in
          L.expect p "][";
          let index = L.integer p in
          L.expect_symbol p ']';
          L.expect_eof p;
          Key (Heap (handle, index)))
        else Num (get (N.of_string s)))

  let parse text =
    Utils.Environment.parse
      ~invalid_line:(fun line -> "Invalid static input line: " ^ line)
      parse_value text
    |> Result.map Utils.Assoc.sorted

  let print env =
    Utils.Environment.print ~trailing_newline:true print_value
      (Utils.Assoc.sorted env)
end

let ifs = function Go (xs, _) | Enter (xs, _) | Halt xs -> xs

module Code = struct
  type t = code

  let rec value_to_text = function
    | VReg r -> "%" ^ r
    | Imm n -> "#" ^ N.to_string n
    | CodeLabel l -> ":" ^ l
    | Ref p -> "ref " ^ place_to_text p

  and place_to_text = function
    | Label l -> "@" ^ l
    | Deref r -> "deref %" ^ r
    | SAcc v -> "sacc " ^ value_to_text v
    | HAcc (h, i) -> "hacc(" ^ value_to_text h ^ ")[" ^ value_to_text i ^ "]"

  let cond_to_text c =
    let a, op, b =
      match c with
      | Lt (a, b) -> (a, "<", b)
      | Eq (a, b) -> (a, "=", b)
      | Gt (a, b) -> (a, ">", b)
    in
    value_to_text a ^ " " ^ op ^ " " ^ value_to_text b

  let stmt_to_text = function
    | Nop -> "nop;"
    | Load (r, p) -> "%" ^ r ^ " := ld " ^ place_to_text p ^ ";"
    | Assign (r, v) -> "%" ^ r ^ " := " ^ value_to_text v ^ ";"
    | BinOp (r, a, op, b) ->
        "%" ^ r ^ " := " ^ value_to_text a
        ^ (if op = Add then " + " else " - ")
        ^ value_to_text b ^ ";"
    | Store (p, v) -> place_to_text p ^ " := st " ^ value_to_text v ^ ";"
    | Pop r -> "pop %" ^ r ^ ";"
    | Push v -> "push " ^ value_to_text v ^ ";"
    | LGet r -> "lget %" ^ r ^ ";"
    | HAlloc (v, r) -> "halloc(" ^ value_to_text v ^ ") %" ^ r ^ ";"
    | HFree v -> "hfree " ^ value_to_text v ^ ";"
    | Print r -> "print %" ^ r ^ ";"
    | Input p -> "input " ^ place_to_text p ^ ";"

  let print (p : code) =
    let b = Buffer.create 512 in
    let line s = Buffer.add_string b (s ^ "\n") in
    List.iter
      (fun (s : static_def) -> line ("@" ^ s.label ^ " " ^ N.to_string s.value))
      p.statics;
    List.iter
      (fun (r : region) ->
        line (":" ^ r.label ^ " {");
        List.iter
          (fun (bb : block) ->
            line ("  :" ^ bb.label ^ " {");
            List.iter (fun s -> line ("    " ^ stmt_to_text s)) bb.stmts;
            List.iter
              (fun j ->
                line
                  ("    if " ^ cond_to_text j.cond ^ " then "
                 ^ value_to_text j.target ^ ";"))
              (ifs bb.cont);
            line
              ("    "
              ^
              match bb.cont with
              | Go (_, v) -> "goto " ^ value_to_text v ^ ";"
              | Enter (_, v) -> "enter " ^ value_to_text v ^ ";"
              | Halt _ -> "halt;");
            line "  }")
          r.blocks;
        line "}")
      p.regions;
    Buffer.contents b

  let vreg p =
    L.expect_symbol p '%';
    L.identifier p

  let rec parse_value p =
    if L.eat_keyword p "ref" then Ref (parse_place p)
    else if L.peek p = L.Symbol '%' then VReg (vreg p)
    else if L.eat_symbol p ':' then CodeLabel (L.identifier p)
    else (
      ignore (L.eat_symbol p '#');
      Imm (L.natural p))

  and parse_place p =
    if L.eat_symbol p '@' then Label (L.identifier p)
    else if L.eat_keyword p "deref" then Deref (vreg p)
    else if L.eat_keyword p "sacc" then SAcc (parse_value p)
    else if L.eat_keyword p "hacc" then (
      L.expect_symbol p '(';
      let h = parse_value p in
      L.expect_symbol p ')';
      L.expect_symbol p '[';
      let i = parse_value p in
      L.expect_symbol p ']';
      HAcc (h, i))
    else L.error p "Expected place expression"

  let parse_cond p =
    let a = parse_value p in
    let op = L.next p in
    let b = parse_value p in
    match op with
    | L.Symbol '<' -> Lt (a, b)
    | L.Symbol '=' -> Eq (a, b)
    | L.Symbol '>' -> Gt (a, b)
    | _ -> L.error p "Expected relation"

  let parse_stmt p =
    let s =
      if L.eat_keyword p "nop" then Nop
      else if L.eat_keyword p "pop" then Pop (vreg p)
      else if L.eat_keyword p "push" then Push (parse_value p)
      else if L.eat_keyword p "lget" then LGet (vreg p)
      else if L.eat_keyword p "hfree" then HFree (parse_value p)
      else if L.eat_keyword p "print" then Print (vreg p)
      else if L.eat_keyword p "input" then Input (parse_place p)
      else if L.eat_keyword p "halloc" then (
        L.expect_symbol p '(';
        let v = parse_value p in
        L.expect_symbol p ')';
        let r = vreg p in
        HAlloc (v, r))
      else if L.peek p = L.Symbol '%' then (
        let r = vreg p in
        L.expect p ":=";
        if L.eat_keyword p "ld" then Load (r, parse_place p)
        else
          let a = parse_value p in
          if L.eat_symbol p '+' then BinOp (r, a, Add, parse_value p)
          else if L.eat_symbol p '-' then BinOp (r, a, Sub, parse_value p)
          else Assign (r, a))
      else
        let place = parse_place p in
        L.expect p ":= st";
        Store (place, parse_value p)
    in
    L.expect_symbol p ';';
    s

  let parse_block p =
    L.expect_symbol p ':';
    let label = L.identifier p in
    L.expect_symbol p '{';
    let rec loop stmts jumps cont =
      if L.eat_symbol p '}' then
        {
          label;
          stmts = List.rev stmts;
          cont =
            (match cont with
            | Some c -> c
            | None -> L.error p "Missing terminator");
        }
      else if L.eat_keyword p "if" then (
        let cond = parse_cond p in
        L.expect_keyword p "then";
        let target = parse_value p in
        L.expect_symbol p ';';
        loop stmts (jumps @ [ { cond; target } ]) cont)
      else if cont <> None then L.error p "Line exists after terminator"
      else if L.eat_keyword p "goto" then (
        let v = parse_value p in
        L.expect_symbol p ';';
        loop stmts jumps (Some (Go (jumps, v))))
      else if L.eat_keyword p "enter" then (
        let v = parse_value p in
        L.expect_symbol p ';';
        loop stmts jumps (Some (Enter (jumps, v))))
      else if L.eat_keyword p "halt" then (
        L.expect_symbol p ';';
        loop stmts jumps (Some (Halt jumps)))
      else loop (parse_stmt p :: stmts) jumps cont
    in
    loop [] [] None

  let parse =
    L.parse (fun p ->
        let rec loop ss rs =
          if L.eof p then { statics = List.rev ss; regions = List.rev rs }
          else if L.eat_symbol p '@' then
            let label = L.identifier p in
            let value = L.natural p in
            loop ({ label; value } :: ss) rs
          else (
            L.expect_symbol p ':';
            let label = L.identifier p in
            L.expect_symbol p '{';
            let blocks = L.many_until p ~closing:'}' parse_block in
            if blocks = [] then L.error p "Region has no blocks";
            loop ss ({ label; blocks } :: rs))
        in
        loop [] [])
end

let unique kind names =
  Utils.Identifier.check_unique
    ~message:(fun name -> "Duplicate " ^ kind ^ ": " ^ name)
    names

let validate (code : code) =
  unique "static label"
    (List.map (fun (s : static_def) -> s.label) code.statics);
  unique "region label" (List.map (fun (r : region) -> r.label) code.regions);
  List.iter
    (fun (r : region) ->
      if r.blocks = [] then failwith ("Region :" ^ r.label ^ " has no blocks");
      unique
        ("block label in region :" ^ r.label)
        (List.map (fun (b : block) -> b.label) r.blocks))
    code.regions;
  if not (List.exists (fun (r : region) -> r.label = "main") code.regions) then
    failwith "Missing entry region: :main"

let make code input =
  L.protect (fun () ->
      validate code;
      let mem =
        List.map (fun (s : static_def) -> (s.label, Num s.value)) code.statics
      in
      let mem =
        List.fold_left
          (fun m (k, v) ->
            if not (List.mem_assoc k m) then
              failwith ("Unknown static input label: @" ^ k);
            Utils.Assoc.set k v m)
          mem input
      in
      let rec idx i = function
        | [] -> assert false
        | (r : region) :: rs -> if r.label = "main" then i else idx (i + 1) rs
      in
      {
        code;
        current_region = idx 0 code.regions;
        current_block = 0;
        current_line = 0;
        vregs = [];
        static_mem = mem;
        stack = [];
        heap = [];
        next_handle = 1;
        halted = false;
      })

let reg s r = Option.value (List.assoc_opt r s.vregs) ~default:(Num N.zero)

let num = function
  | Num n -> n
  | v -> failwith ("Expected number, got " ^ Ainput.print_value v)

let index v = get (N.to_int (num v))

let rec eval_value s = function
  | VReg r -> reg s r
  | Imm n -> Num n
  | CodeLabel l -> Code_label l
  | Ref p -> Key (eval_place s p)

and eval_place s = function
  | Label l ->
      if List.mem_assoc l s.static_mem then Static l
      else failwith ("Place is not data label: @" ^ l)
  | Deref r -> (
      match reg s r with
      | Key k -> k
      | v ->
          failwith
            ("deref expects place key in vreg, got " ^ Ainput.print_value v))
  | SAcc v -> Stack (index (eval_value s v))
  | HAcc (h, i) -> Heap (index (eval_value s h), index (eval_value s i))

let read_place s = function
  | Static l -> (
      match List.assoc_opt l s.static_mem with
      | Some v -> v
      | None -> failwith ("Unknown data label: @" ^ l))
  | Stack i ->
      Utils.List_ext.nth s.stack i (Printf.sprintf "Invalid stack key: s[%d]" i)
  | Heap (h, i) ->
      Utils.List_ext.nth
        (Option.value (List.assoc_opt h s.heap) ~default:[])
        i
        (Printf.sprintf "Invalid heap key: h[%d][%d]" h i)

let write_place s key value =
  match key with
  | Static l ->
      ignore (read_place s key);
      { s with static_mem = Utils.Assoc.set l value s.static_mem }
  | Stack i ->
      {
        s with
        stack =
          Utils.List_ext.replace s.stack i value
            (Printf.sprintf "Invalid stack key: s[%d]" i);
      }
  | Heap (h, i) ->
      let area =
        match List.assoc_opt h s.heap with
        | Some a -> a
        | None -> failwith (Printf.sprintf "Unknown heap handle: %d" h)
      in
      {
        s with
        heap =
          Utils.Assoc.set h
            (Utils.List_ext.replace area i value
               (Printf.sprintf "Invalid heap key: h[%d][%d]" h i))
            s.heap;
      }

let eval_cond s c =
  let a, b, op =
    match c with
    | Lt (a, b) -> (a, b, -1)
    | Eq (a, b) -> (a, b, 0)
    | Gt (a, b) -> (a, b, 1)
  in
  let a = eval_value s a and b = eval_value s b in
  match (a, b) with
  | Num a, Num b ->
      let cmp = N.compare a b in
      if op = 0 then cmp = 0 else if op < 0 then cmp < 0 else cmp > 0
  | _ when op = 0 -> a = b
  | _ ->
      failwith
        ("Condition "
        ^ (if op < 0 then "lt" else "gt")
        ^ " supports only numbers, got " ^ Ainput.print_value a ^ " and "
        ^ Ainput.print_value b)

let find_idx f xs err =
  let rec loop i = function
    | [] -> failwith err
    | x :: xs -> if f x then i else loop (i + 1) xs
  in
  loop 0 xs

let target s v =
  match eval_value s v with
  | Code_label l -> l
  | x -> failwith ("jump target expects code label, got " ^ Ainput.print_value x)

let jump_go s v =
  let l = target s v in
  let r = List.nth s.code.regions s.current_region in
  {
    s with
    current_block =
      find_idx
        (fun (b : block) -> b.label = l)
        r.blocks
        ("goto target must be local block label, got :" ^ l);
    current_line = 0;
  }

let jump_enter s v =
  let l = target s v in
  {
    s with
    current_region =
      find_idx
        (fun (r : region) -> r.label = l)
        s.code.regions
        ("enter target must be region label, got :" ^ l);
    current_block = 0;
    current_line = 0;
    vregs = [];
  }

let step s input =
  L.protect (fun () ->
      let halt s = Utils.Machine.Halt (Utils.Assoc.sorted s.static_mem) in
      let continue s o = Utils.Machine.Continue { next = s; output = o } in
      if s.halted then halt s
      else
        let r =
          Utils.List_ext.nth s.code.regions s.current_region
            "Current region out of range"
        in
        let b =
          Utils.List_ext.nth r.blocks s.current_block
            "Current block out of range"
        in
        let sl = List.length b.stmts and il = List.length (ifs b.cont) in
        if s.current_line < sl then
          let st = List.nth b.stmts s.current_line in
          let s = { s with current_line = s.current_line + 1 } in
          let put r v = { s with vregs = Utils.Assoc.set r v s.vregs } in
          let s, o =
            match st with
            | Nop -> (s, "")
            | Assign (r, v) -> (put r (eval_value s v), "")
            | Load (r, p) -> (put r (read_place s (eval_place s p)), "")
            | BinOp (r, a, op, b) ->
                let a = num (eval_value s a) and b = num (eval_value s b) in
                (put r (Num ((if op = Add then N.add else N.sub) a b)), "")
            | Store (p, v) ->
                (write_place s (eval_place s p) (eval_value s v), "")
            | Push v -> ({ s with stack = s.stack @ [ eval_value s v ] }, "")
            | Pop r -> (
                match List.rev s.stack with
                | [] -> failwith "pop on empty stack"
                | v :: vs ->
                    ( {
                        s with
                        stack = List.rev vs;
                        vregs = Utils.Assoc.set r v s.vregs;
                      },
                      "" ))
            | LGet r -> (put r (Num (N.of_int (List.length s.stack))), "")
            | HAlloc (v, r) ->
                let size = index (eval_value s v) in
                ( {
                    s with
                    heap =
                      Utils.Assoc.set s.next_handle
                        (List.init size (fun _ -> Num N.zero))
                        s.heap;
                    next_handle = s.next_handle + 1;
                    vregs =
                      Utils.Assoc.set r (Num (N.of_int s.next_handle)) s.vregs;
                  },
                  "" )
            | HFree v ->
                let h = index (eval_value s v) in
                if not (List.mem_assoc h s.heap) then
                  failwith (Printf.sprintf "hfree unknown handle: %d" h);
                ({ s with heap = List.remove_assoc h s.heap }, "")
            | Print r ->
                let bytes = N.to_bytes (num (reg s r)) in
                if not (Utils.Utf8.valid bytes) then
                  failwith "print byte is not valid single-byte UTF-8";
                (s, bytes)
            | Input p ->
                (write_place s (eval_place s p) (Num (N.of_bytes input)), "")
          in
          continue s o
        else if s.current_line < sl + il then
          let j = List.nth (ifs b.cont) (s.current_line - sl) in
          let s = { s with current_line = s.current_line + 1 } in
          continue (if eval_cond s j.cond then jump_go s j.target else s) ""
        else if s.current_line = sl + il then
          match b.cont with
          | Go (_, v) -> continue (jump_go s v) ""
          | Enter (_, v) -> continue (jump_enter s v) ""
          | Halt _ -> halt s
        else failwith "Current line out of range")

let snapshot = state_to_yojson
let restore = state_of_yojson

let render s =
  let module R = Utils.Render in
  let r = List.nth_opt s.code.regions s.current_region in
  let b = Option.bind r (fun r -> List.nth_opt r.blocks s.current_block) in
  R.state
    [
      R.text ~title:"current_region"
        (":"
        ^ Option.fold ~none:"<invalid>" ~some:(fun (r : region) -> r.label) r);
      R.text ~title:"current_block"
        (":"
        ^ Option.fold ~none:"<invalid>" ~some:(fun (b : block) -> b.label) b);
      R.text ~title:"current_line" (string_of_int s.current_line);
      R.text ~title:"halted" (string_of_bool s.halted);
      R.assoc_table ~title:"vregs" ~columns:[ "vreg"; "value" ]
        Ainput.print_value
        (List.map (fun (k, v) -> ("%" ^ k, v)) (Utils.Assoc.sorted s.vregs));
      R.text_table ~title:"static" ~columns:[ "label"; "value" ]
        (List.map
           (fun (d : static_def) ->
             [
               "@" ^ d.label;
               Ainput.print_value
                 (Option.value
                    (List.assoc_opt d.label s.static_mem)
                    ~default:(Num N.zero));
             ])
           s.code.statics);
      R.indexed_table ~title:"stack" Ainput.print_value s.stack;
      R.text_table ~title:"heap"
        ~columns:[ "handle"; "index"; "value" ]
        (List.concat_map
           (fun (h, vs) ->
             List.mapi
               (fun i v ->
                 [ string_of_int h; string_of_int i; Ainput.print_value v ])
               vs)
           (Utils.Assoc.sorted s.heap));
    ]

module Rinput = Utils.Text.String
module Routput = Utils.Text.String
module Foutput = Ainput

module Machine = struct
  module Code = Code
  module Ainput = Ainput
  module Rinput = Rinput
  module Routput = Routput
  module Foutput = Foutput

  type nonrec state = state

  let make = make
  let step = step
  let snapshot = snapshot
  let restore = restore
  let render = render
end
