module N = Utils.Number
module L = Utils.Parser
module Vreg_id = Utils.Id.Make ()
module Registers = Utils.Intern.Make (Utils.Identifier) (Vreg_id)
module Static_id = Utils.Id.Make ()
module Region_id = Utils.Id.Make ()
module Block_id = Utils.Id.Make ()
module Heap_id = Utils.Id.Make ()

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

type code_target = Region_target of Region_id.t | Block_target of Block_id.t
[@@deriving yojson]

module Target_names = Utils.Naming.Make (struct
  type t = code_target

  let compare = Stdlib.compare
end)

type 'data location = Static of 'data | Stack of int | Heap of Heap_id.t * int
[@@deriving yojson]

type ('label, 'data) value =
  | Num of N.t
  | Code_label of 'label
  | Key of 'data location
[@@deriving yojson]

type place_key = Static_id.t location [@@deriving yojson]

type flow_value = (Utils.Identifier.t, Utils.Identifier.t) value
[@@deriving yojson]

type runtime_value = (code_target, Static_id.t) value [@@deriving yojson]

type resolution = {
  registers : (Utils.Identifier.t * Vreg_id.t) list;
  data : (Utils.Identifier.t * Static_id.t) list;
  regions : (Utils.Identifier.t * Region_id.t) list;
  blocks : (Utils.Identifier.t * Block_id.t) list list;
  label_names : (code_target * Utils.Identifier.t) list;
}
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
  resolution : resolution;
  vregs : (Vreg_id.t * runtime_value) list;
  static_mem : (Static_id.t * runtime_value) list;
  stack : runtime_value list;
  heap : (Heap_id.t * runtime_value list) list;
  next_handle : Heap_id.Supply.t;
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
    | Key (Heap (h, i)) -> Printf.sprintf "k:h[%d][%d]" (Heap_id.to_int h) i

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
        else if prefix "k:s[" then (
          let p = L.create s in
          L.expect p "k:s";
          let index = L.group p ~opening:'[' L.integer in
          L.expect_eof p;
          Key (Stack index))
        else if prefix "k:h[" then (
          let p = L.create s in
          L.expect p "k:h";
          let handle = L.group p ~opening:'[' L.integer in
          let index = L.group p ~opening:'[' L.integer in
          L.expect_eof p;
          Key (Heap (get (Heap_id.of_int handle), index)))
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
    if L.bump_if_keyword p "ref" then Ref (parse_place p)
    else if L.peek p = L.Symbol '%' then VReg (vreg p)
    else if L.bump_if_symbol p ':' then CodeLabel (L.identifier p)
    else (
      ignore (L.bump_if_symbol p '#');
      Imm (L.natural p))

  and parse_place p =
    if L.bump_if_symbol p '@' then Label (L.identifier p)
    else if L.bump_if_keyword p "deref" then Deref (vreg p)
    else if L.bump_if_keyword p "sacc" then SAcc (parse_value p)
    else if L.bump_if_keyword p "hacc" then
      let h = L.group p ~opening:'(' parse_value in
      let i = L.group p ~opening:'[' parse_value in
      HAcc (h, i)
    else L.error p "Expected place expression"

  let parse_cond p =
    L.binary p
      ~operators:
        [
          ("<", fun a b -> Lt (a, b));
          ("=", fun a b -> Eq (a, b));
          (">", fun a b -> Gt (a, b));
        ]
      ~operand:parse_value
      ~make:(fun a op b -> op a b)

  let parse_stmt p =
    let s =
      if L.bump_if_keyword p "nop" then Nop
      else if L.bump_if_keyword p "pop" then Pop (vreg p)
      else if L.bump_if_keyword p "push" then Push (parse_value p)
      else if L.bump_if_keyword p "lget" then LGet (vreg p)
      else if L.bump_if_keyword p "hfree" then HFree (parse_value p)
      else if L.bump_if_keyword p "print" then Print (vreg p)
      else if L.bump_if_keyword p "input" then Input (parse_place p)
      else if L.bump_if_keyword p "halloc" then
        let v = L.group p ~opening:'(' parse_value in
        let r = vreg p in
        HAlloc (v, r)
      else if L.peek p = L.Symbol '%' then (
        let r = vreg p in
        L.expect p ":=";
        if L.bump_if_keyword p "ld" then Load (r, parse_place p)
        else
          let a = parse_value p in
          if L.bump_if_symbol p '+' then BinOp (r, a, Add, parse_value p)
          else if L.bump_if_symbol p '-' then BinOp (r, a, Sub, parse_value p)
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
    L.group p ~opening:'{' (fun p ->
        let rec loop stmts jumps cont =
          if L.eof p then
            {
              label;
              stmts = List.rev stmts;
              cont =
                (match cont with
                | Some c -> c
                | None -> L.error p "Missing terminator");
            }
          else if L.bump_if_keyword p "if" then (
            let cond = parse_cond p in
            L.expect_keyword p "then";
            let target = parse_value p in
            L.expect_symbol p ';';
            loop stmts (jumps @ [ { cond; target } ]) cont)
          else if cont <> None then L.error p "Line exists after terminator"
          else if L.bump_if_keyword p "goto" then (
            let v = parse_value p in
            L.expect_symbol p ';';
            loop stmts jumps (Some (Go (jumps, v))))
          else if L.bump_if_keyword p "enter" then (
            let v = parse_value p in
            L.expect_symbol p ';';
            loop stmts jumps (Some (Enter (jumps, v))))
          else if L.bump_if_keyword p "halt" then (
            L.expect_symbol p ';';
            loop stmts jumps (Some (Halt jumps)))
          else loop (parse_stmt p :: stmts) jumps cont
        in
        loop [] [] None)

  let parse =
    L.parse (fun p ->
        let rec loop ss rs =
          if L.eof p then { statics = List.rev ss; regions = List.rev rs }
          else if L.bump_if_symbol p '@' then
            let label = L.identifier p in
            let value = L.natural p in
            loop ({ label; value } :: ss) rs
          else (
            L.expect_symbol p ':';
            let label = L.identifier p in
            let blocks =
              L.group p ~opening:'{' (fun p -> L.many p parse_block)
            in
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

let name_of_id entries id =
  match List.find_opt (fun (_, value) -> value = id) entries with
  | Some (name, _) -> name
  | None -> failwith "Unknown resolved ID"

let id_of_name entries name =
  match List.assoc_opt name entries with
  | Some id -> id
  | None -> failwith ("Unknown name: " ^ name)

let build_resolution (code : code) =
  let registers = ref [] and registry = ref Registers.empty in
  let register name =
    match Registers.find name !registry with
    | Some _ -> ()
    | None -> registers := (name, Registers.take name registry) :: !registers
  in
  let rec value = function
    | VReg r -> register r
    | Ref p -> place p
    | Imm _ | CodeLabel _ -> ()
  and place = function
    | Deref r -> register r
    | SAcc v -> value v
    | HAcc (a, b) ->
        value a;
        value b
    | Label _ -> ()
  in
  let condition = function
    | Lt (a, b) | Eq (a, b) | Gt (a, b) ->
        value a;
        value b
  in
  let statement = function
    | Load (r, p) ->
        register r;
        place p
    | Assign (r, v) | HAlloc (v, r) ->
        register r;
        value v
    | BinOp (r, a, _, b) ->
        register r;
        value a;
        value b
    | Store (p, v) ->
        place p;
        value v
    | Pop r | LGet r | Print r -> register r
    | Push v | HFree v -> value v
    | Input p -> place p
    | Nop -> ()
  in
  let region_supply = ref Region_id.Supply.empty
  and block_supply = ref Block_id.Supply.empty in
  let labels = ref Target_names.empty and label_names = ref [] in
  let label target hint =
    let name, next = Target_names.assign ~hint target !labels in
    labels := next;
    label_names := !label_names @ [ (target, name) ]
  in
  let regions =
    List.map
      (fun (r : region) ->
        let id = Region_id.Supply.take region_supply in
        label (Region_target id) r.label;
        (r.label, id))
      code.regions
  in
  let blocks =
    List.map
      (fun (r : region) ->
        List.map
          (fun (b : block) ->
            List.iter statement b.stmts;
            List.iter
              (fun j ->
                condition j.cond;
                value j.target)
              (ifs b.cont);
            (match b.cont with
            | Go (_, v) | Enter (_, v) -> value v
            | Halt _ -> ());
            let id = Block_id.Supply.take block_supply in
            label (Block_target id) b.label;
            (b.label, id))
          r.blocks)
      code.regions
  in
  let data_supply = ref Static_id.Supply.empty in
  let data =
    List.map
      (fun (d : static_def) -> (d.label, Static_id.Supply.take data_supply))
      code.statics
  in
  {
    registers = List.rev !registers;
    data;
    regions;
    blocks;
    label_names = !label_names;
  }

let local_target resolution region name =
  Block_target (id_of_name (List.nth resolution.blocks region) name)

let region_target resolution name =
  Region_target (id_of_name resolution.regions name)

let resolve_target resolution region name =
  match List.assoc_opt name resolution.regions with
  | Some id -> Region_target id
  | None -> (
      match List.assoc_opt name (List.nth resolution.blocks region) with
      | Some id -> Block_target id
      | None -> (
          match
            List.find_opt
              (fun (_, display) -> display = name)
              resolution.label_names
          with
          | Some (target, _) -> target
          | None -> local_target resolution region name))

let target_name resolution target = List.assoc target resolution.label_names

let resolve_input resolution region = function
  | Num n -> Num n
  | Code_label name -> Code_label (resolve_target resolution region name)
  | Key (Static name) -> Key (Static (id_of_name resolution.data name))
  | Key (Stack i) -> Key (Stack i)
  | Key (Heap (h, i)) -> Key (Heap (h, i))

let named_value resolution = function
  | Num n -> Num n
  | Code_label target -> Code_label (target_name resolution target)
  | Key (Static id) -> Key (Static (name_of_id resolution.data id))
  | Key (Stack i) -> Key (Stack i)
  | Key (Heap (h, i)) -> Key (Heap (h, i))

let runtime_text s v = Ainput.print_value (named_value s.resolution v)
let register_id s name = id_of_name s.resolution.registers name

let make code input =
  L.protect (fun () ->
      validate code;
      let resolution = build_resolution code in
      let rec main_index i = function
        | [] -> assert false
        | (r : region) :: rest ->
            if r.label = "main" then i else main_index (i + 1) rest
      in
      let main = main_index 0 code.regions in
      let mem =
        List.map
          (fun (s : static_def) ->
            (id_of_name resolution.data s.label, Num s.value))
          code.statics
      in
      let mem =
        List.fold_left
          (fun m (k, v) ->
            let key = id_of_name resolution.data k in
            Utils.Assoc.set key (resolve_input resolution main v) m)
          mem input
      in
      let rec idx i = function
        | [] -> assert false
        | (r : region) :: rs -> if r.label = "main" then i else idx (i + 1) rs
      in
      {
        code;
        resolution;
        current_region = idx 0 code.regions;
        current_block = 0;
        current_line = 0;
        vregs = [];
        static_mem = mem;
        stack = [];
        heap = [];
        next_handle = Heap_id.Supply.after (get (Heap_id.of_int 0));
        halted = false;
      })

let reg s r =
  Option.value (List.assoc_opt (register_id s r) s.vregs) ~default:(Num N.zero)

let num s = function
  | Num n -> n
  | v -> failwith ("Expected number, got " ^ runtime_text s v)

let index s v = get (N.to_int (num s v))

let rec eval_value s = function
  | VReg r -> reg s r
  | Imm n -> Num n
  | CodeLabel l -> Code_label (resolve_target s.resolution s.current_region l)
  | Ref p -> Key (eval_place s p)

and eval_place s = function
  | Label l ->
      let id = id_of_name s.resolution.data l in
      if List.mem_assoc id s.static_mem then Static id
      else failwith ("Place is not data label: @" ^ l)
  | Deref r -> (
      match reg s r with
      | Key k -> k
      | v ->
          failwith ("deref expects place key in vreg, got " ^ runtime_text s v))
  | SAcc v -> Stack (index s (eval_value s v))
  | HAcc (h, i) ->
      Heap
        ( get (Heap_id.of_int (index s (eval_value s h))),
          index s (eval_value s i) )

let read_place s = function
  | Static l -> (
      match List.assoc_opt l s.static_mem with
      | Some v -> v
      | None ->
          failwith ("Unknown data label: @" ^ name_of_id s.resolution.data l))
  | Stack i ->
      Utils.List_ext.nth s.stack i (Printf.sprintf "Invalid stack key: s[%d]" i)
  | Heap (h, i) ->
      Utils.List_ext.nth
        (Option.value (List.assoc_opt h s.heap) ~default:[])
        i
        (Printf.sprintf "Invalid heap key: h[%d][%d]" (Heap_id.to_int h) i)

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
        | None ->
            failwith
              (Printf.sprintf "Unknown heap handle: %d" (Heap_id.to_int h))
      in
      {
        s with
        heap =
          Utils.Assoc.set h
            (Utils.List_ext.replace area i value
               (Printf.sprintf "Invalid heap key: h[%d][%d]" (Heap_id.to_int h)
                  i))
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
        ^ " supports only numbers, got " ^ runtime_text s a ^ " and "
        ^ runtime_text s b)

let find_idx f xs err =
  let rec loop i = function
    | [] -> failwith err
    | x :: xs -> if f x then i else loop (i + 1) xs
  in
  loop 0 xs

let target s v =
  match eval_value s v with
  | Code_label l -> l
  | x -> failwith ("jump target expects code label, got " ^ runtime_text s x)

let jump_go s v =
  let label =
    match v with
    | CodeLabel name -> local_target s.resolution s.current_region name
    | _ -> target s v
  in
  let id =
    match label with
    | Block_target id -> id
    | _ -> failwith "goto expects a local block ID"
  in
  {
    s with
    current_block =
      find_idx
        (fun (_, value) -> Block_id.equal id value)
        (List.nth s.resolution.blocks s.current_region)
        "goto target is not in the current region";
    current_line = 0;
  }

let jump_enter s v =
  let label =
    match v with
    | CodeLabel name -> region_target s.resolution name
    | _ -> target s v
  in
  let id =
    match label with
    | Region_target id -> id
    | _ -> failwith "enter expects a region ID"
  in
  {
    s with
    current_region =
      find_idx
        (fun (_, value) -> Region_id.equal id value)
        s.resolution.regions "unknown region ID";
    current_block = 0;
    current_line = 0;
    vregs = [];
  }

let step s input =
  L.protect (fun () ->
      let halt s =
        Utils.Machine.Halt
          (Utils.Assoc.sorted
             (List.map
                (fun (id, v) ->
                  (name_of_id s.resolution.data id, named_value s.resolution v))
                s.static_mem))
      in
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
          let put r v =
            { s with vregs = Utils.Assoc.set (register_id s r) v s.vregs }
          in
          let s, o =
            match st with
            | Nop -> (s, "")
            | Assign (r, v) -> (put r (eval_value s v), "")
            | Load (r, p) -> (put r (read_place s (eval_place s p)), "")
            | BinOp (r, a, op, b) ->
                let a = num s (eval_value s a) and b = num s (eval_value s b) in
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
                        vregs = Utils.Assoc.set (register_id s r) v s.vregs;
                      },
                      "" ))
            | LGet r -> (put r (Num (N.of_int (List.length s.stack))), "")
            | HAlloc (v, r) ->
                let size = index s (eval_value s v) in
                let handle, next_handle = Heap_id.Supply.fresh s.next_handle in
                ( {
                    s with
                    heap =
                      Utils.Assoc.set handle
                        (List.init size (fun _ -> Num N.zero))
                        s.heap;
                    next_handle;
                    vregs =
                      Utils.Assoc.set (register_id s r)
                        (Num (N.of_int (Heap_id.to_int handle)))
                        s.vregs;
                  },
                  "" )
            | HFree v ->
                let h = get (Heap_id.of_int (index s (eval_value s v))) in
                if not (List.mem_assoc h s.heap) then
                  failwith
                    (Printf.sprintf "hfree unknown handle: %d"
                       (Heap_id.to_int h));
                ({ s with heap = List.remove_assoc h s.heap }, "")
            | Print r ->
                let bytes = N.to_bytes (num s (reg s r)) in
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

let restore json =
  Result.bind (state_of_yojson json) (fun s ->
      L.protect (fun () ->
          validate s.code;
          if s.resolution <> build_resolution s.code then
            failwith "Inconsistent name resolution in snapshot";
          let known_data id = ignore (name_of_id s.resolution.data id) in
          let heap_ids = ref (Heap_id.Set.of_list (List.map fst s.heap)) in
          let check_value = function
            | Num _ -> ()
            | Code_label target -> ignore (target_name s.resolution target)
            | Key (Static id) -> known_data id
            | Key (Heap (id, _)) -> heap_ids := Heap_id.Set.add id !heap_ids
            | Key (Stack _) -> ()
          in
          List.iter
            (fun (id, v) ->
              ignore (name_of_id s.resolution.registers id);
              check_value v)
            s.vregs;
          List.iter
            (fun (id, v) ->
              known_data id;
              check_value v)
            s.static_mem;
          List.iter check_value s.stack;
          List.iter (fun (_, values) -> List.iter check_value values) s.heap;
          get (Heap_id.Supply.validate_used !heap_ids s.next_handle);
          s))

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
      R.assoc_table ~title:"vregs" ~columns:[ "vreg"; "value" ] (runtime_text s)
        (List.map
           (fun (k, v) -> ("%" ^ name_of_id s.resolution.registers k, v))
           (Utils.Assoc.sorted s.vregs));
      R.text_table ~title:"static" ~columns:[ "label"; "value" ]
        (List.map
           (fun (d : static_def) ->
             [
               "@" ^ d.label;
               runtime_text s
                 (Option.value
                    (List.assoc_opt
                       (id_of_name s.resolution.data d.label)
                       s.static_mem)
                    ~default:(Num N.zero));
             ])
           s.code.statics);
      R.indexed_table ~title:"stack" (runtime_text s) s.stack;
      R.text_table ~title:"heap"
        ~columns:[ "handle"; "index"; "value" ]
        (List.concat_map
           (fun (h, vs) ->
             List.mapi
               (fun i v ->
                 [
                   string_of_int (Heap_id.to_int h);
                   string_of_int i;
                   runtime_text s v;
                 ])
               vs)
           (Utils.Assoc.sorted s.heap));
    ]

module Rinput = Utils.Text.String
module Routput = Utils.Text.String
module Foutput = Ainput
