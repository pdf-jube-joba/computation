module N = Utils.Number
module L = Utils.Lexer

let get = function Ok x -> x | Error e -> failwith e

let set k v xs =
  if List.mem_assoc k xs then
    List.map (fun (a, b) -> (a, if a = k then v else b)) xs
  else xs @ [ (k, v) ]

let sorted xs = List.sort (fun (a, _) (b, _) -> compare a b) xs

type binop = Add | Sub [@@deriving yojson]

type value_expr =
  | VReg of string
  | Imm of N.t
  | CodeLabel of string
  | Ref of place_expr

and place_expr =
  | Label of string
  | Deref of string
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
  | Load of string * place_expr
  | Assign of string * value_expr
  | BinOp of string * value_expr * binop * value_expr
  | Store of place_expr * value_expr
  | Pop of string
  | Push of value_expr
  | LGet of string
  | HAlloc of value_expr * string
  | HFree of value_expr
  | Print of string
  | Input of place_expr
[@@deriving yojson]

type jump_if = { cond : cond; target : value_expr } [@@deriving yojson]

type cont =
  | Go of jump_if list * value_expr
  | Enter of jump_if list * value_expr
  | Halt of jump_if list
[@@deriving yojson]

type block = { label : string; stmts : stmt list; cont : cont }
[@@deriving yojson]

type region = { label : string; blocks : block list } [@@deriving yojson]
type static_def = { label : string; value : N.t } [@@deriving yojson]

type code = { statics : static_def list; regions : region list }
[@@deriving yojson]

type place_key = Static of string | Stack of int | Heap of int * int
[@@deriving yojson]

type flow_value = Num of N.t | Code_label of string | Key of place_key
[@@deriving yojson]

type ainput = (string * flow_value) list [@@deriving yojson]
type foutput = ainput [@@deriving yojson]
type rinput = string
type routput = string

type state = {
  code : code;
  current_region : int;
  current_block : int;
  current_line : int;
  vregs : (string * flow_value) list;
  static_mem : ainput;
  stack : flow_value list;
  heap : (int * flow_value list) list;
  next_handle : int;
  halted : bool;
}
[@@deriving yojson]

let ifs = function Go (xs, _) | Enter (xs, _) | Halt xs -> xs

let value_print = function
  | Num n -> N.to_string n
  | Code_label l -> ":" ^ l
  | Key (Static l) -> "k:@" ^ l
  | Key (Stack i) -> Printf.sprintf "k:s[%d]" i
  | Key (Heap (h, i)) -> Printf.sprintf "k:h[%d][%d]" h i

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

let name p =
  match L.next p with
  | L.Ident s | L.Number s -> s
  | _ -> L.error p "Expected name"

let vreg p =
  L.expect_symbol p '%';
  name p

let number p = get (N.of_string (L.natural p))

let rec parse_value p =
  if L.eat_keyword p "ref" then Ref (parse_place p)
  else if L.peek p = L.Symbol '%' then VReg (vreg p)
  else if L.eat_symbol p ':' then CodeLabel (name p)
  else (
    ignore (L.eat_symbol p '#');
    Imm (number p))

and parse_place p =
  if L.eat_symbol p '@' then Label (name p)
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
  let label = name p in
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

let parse_code =
  L.parse (fun p ->
      let rec loop ss rs =
        if L.eof p then { statics = List.rev ss; regions = List.rev rs }
        else if L.eat_symbol p '@' then
          let label = name p in
          let value = number p in
          loop ({ label; value } :: ss) rs
        else (
          L.expect_symbol p ':';
          let label = name p in
          L.expect_symbol p '{';
          let rec blocks acc =
            if L.eat_symbol p '}' then List.rev acc
            else blocks (parse_block p :: acc)
          in
          let blocks = blocks [] in
          if blocks = [] then L.error p "Region has no blocks";
          loop ss ({ label; blocks } :: rs))
      in
      loop [] [])

let parse_flow_value text =
  L.protect (fun () ->
      let s = String.trim text in
      let prefix p = String.starts_with ~prefix:p s in
      let rest p =
        String.sub s (String.length p) (String.length s - String.length p)
      in
      if prefix ":" then (
        let label = rest ":" in
        if label = "" then failwith "Empty code label";
        Code_label label)
      else if prefix "k:@" then (
        let label = rest "k:@" in
        if label = "" then failwith "Empty static key";
        Key (Static label))
      else if prefix "k:s[" then
        let body = rest "k:s[" in
        if not (String.ends_with ~suffix:"]" body) then
          failwith ("Invalid stack key: " ^ s)
        else
          let idx = String.sub body 0 (String.length body - 1) in
          Key (Stack (get (N.to_int (get (N.of_string idx)))))
      else if prefix "k:h[" then (
        let body = rest "k:h[" in
        let p = L.create body in
        let handle = L.integer p in
        L.expect p "][";
        let index = L.integer p in
        L.expect_symbol p ']';
        L.expect_eof p;
        Key (Heap (handle, index)))
      else Num (get (N.of_string s)))

let parse_ainput text =
  L.protect (fun () ->
      List.fold_left
        (fun acc line ->
          let line = String.trim line in
          if line = "" then acc
          else
            match String.index_opt line '=' with
            | None -> failwith ("Invalid static input line: " ^ line)
            | Some i ->
                let key = String.trim (String.sub line 0 i) in
                if key = "" then failwith "Empty static input key";
                set key
                  (get
                     (parse_flow_value
                        (String.sub line (i + 1) (String.length line - i - 1))))
                  acc)
        []
        (String.split_on_char '\n' text)
      |> sorted)

let print_env env =
  String.concat ""
    (List.map (fun (k, v) -> k ^ " = " ^ value_print v ^ "\n") (sorted env))

let parse_rinput s = Ok s

let unique kind names =
  let rec loop seen = function
    | [] -> ()
    | x :: xs ->
        if List.mem x seen then failwith ("Duplicate " ^ kind ^ ": " ^ x);
        loop (x :: seen) xs
  in
  loop [] names

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
            set k v m)
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
  | v -> failwith ("Expected number, got " ^ value_print v)

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
      | v -> failwith ("deref expects place key in vreg, got " ^ value_print v))
  | SAcc v -> Stack (index (eval_value s v))
  | HAcc (h, i) -> Heap (index (eval_value s h), index (eval_value s i))

let nth xs i err =
  match List.nth_opt xs i with Some v -> v | None -> failwith err

let read_place s = function
  | Static l -> (
      match List.assoc_opt l s.static_mem with
      | Some v -> v
      | None -> failwith ("Unknown data label: @" ^ l))
  | Stack i -> nth s.stack i (Printf.sprintf "Invalid stack key: s[%d]" i)
  | Heap (h, i) ->
      nth
        (Option.value (List.assoc_opt h s.heap) ~default:[])
        i
        (Printf.sprintf "Invalid heap key: h[%d][%d]" h i)

let replace xs i value err =
  ignore (nth xs i err);
  List.mapi (fun j v -> if i = j then value else v) xs

let write_place s key value =
  match key with
  | Static l ->
      ignore (read_place s key);
      { s with static_mem = set l value s.static_mem }
  | Stack i ->
      {
        s with
        stack =
          replace s.stack i value (Printf.sprintf "Invalid stack key: s[%d]" i);
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
          set h
            (replace area i value
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
        ^ " supports only numbers, got " ^ value_print a ^ " and "
        ^ value_print b)

let find_idx f xs err =
  let rec loop i = function
    | [] -> failwith err
    | x :: xs -> if f x then i else loop (i + 1) xs
  in
  loop 0 xs

let target s v =
  match eval_value s v with
  | Code_label l -> l
  | x -> failwith ("jump target expects code label, got " ^ value_print x)

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

let number_of_bytes text =
  let n = ref N.zero in
  for i = String.length text - 1 downto 0 do
    n := N.add (N.mul !n "256") (N.of_int (Char.code text.[i]))
  done;
  !n

let bytes_of_number n =
  let b = Buffer.create 8 in
  let rec loop n =
    if not (N.is_zero n) then (
      let q, r = get (N.divmod n "256") in
      Buffer.add_char b (Char.chr (get (N.to_int r)));
      loop q)
  in
  loop n;
  Buffer.contents b

let valid_utf8 s =
  let n = String.length s in
  let byte i = Char.code s.[i] in
  let rec check i =
    if i = n then true
    else
      let a = byte i in
      if a < 128 then check (i + 1)
      else
        let count, lo, hi =
          if a >= 194 && a <= 223 then (1, 128, 191)
          else if a = 224 then (2, 160, 191)
          else if (a >= 225 && a <= 236) || (a >= 238 && a <= 239) then
            (2, 128, 191)
          else if a = 237 then (2, 128, 159)
          else if a = 240 then (3, 144, 191)
          else if a >= 241 && a <= 243 then (3, 128, 191)
          else if a = 244 then (3, 128, 143)
          else (0, 0, 0)
        in
        count > 0
        && i + count < n
        && byte (i + 1) >= lo
        && byte (i + 1) <= hi
        &&
        let rec rest j =
          if j > count then check (i + count + 1)
          else byte (i + j) >= 128 && byte (i + j) <= 191 && rest (j + 1)
        in
        rest 2
  in
  check 0

let step s input =
  L.protect (fun () ->
      let halt s = Utils.Machine.Halt (sorted s.static_mem) in
      let continue s o = Utils.Machine.Continue { next = s; output = o } in
      if s.halted then halt s
      else
        let r =
          nth s.code.regions s.current_region "Current region out of range"
        in
        let b = nth r.blocks s.current_block "Current block out of range" in
        let sl = List.length b.stmts and il = List.length (ifs b.cont) in
        if s.current_line < sl then
          let st = List.nth b.stmts s.current_line in
          let s = { s with current_line = s.current_line + 1 } in
          let put r v = { s with vregs = set r v s.vregs } in
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
                    ({ s with stack = List.rev vs; vregs = set r v s.vregs }, "")
                )
            | LGet r -> (put r (Num (N.of_int (List.length s.stack))), "")
            | HAlloc (v, r) ->
                let size = index (eval_value s v) in
                ( {
                    s with
                    heap =
                      set s.next_handle
                        (List.init size (fun _ -> Num N.zero))
                        s.heap;
                    next_handle = s.next_handle + 1;
                    vregs = set r (Num (N.of_int s.next_handle)) s.vregs;
                  },
                  "" )
            | HFree v ->
                let h = index (eval_value s v) in
                if not (List.mem_assoc h s.heap) then
                  failwith (Printf.sprintf "hfree unknown handle: %d" h);
                ({ s with heap = List.remove_assoc h s.heap }, "")
            | Print r ->
                let bytes = bytes_of_number (num (reg s r)) in
                if not (valid_utf8 bytes) then
                  failwith "print byte is not valid single-byte UTF-8";
                (s, bytes)
            | Input p ->
                ( write_place s (eval_place s p) (Num (number_of_bytes input)),
                  "" )
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
  let text = Utils.Render.text in
  let table title cols rows =
    Utils.Render.table ~title ~columns:(List.map text cols)
      (List.map (List.map text) rows)
  in
  let r = List.nth_opt s.code.regions s.current_region in
  let b = Option.bind r (fun r -> List.nth_opt r.blocks s.current_block) in
  Utils.Render.state
    [
      text ~title:"current_region"
        (":"
        ^ Option.fold ~none:"<invalid>" ~some:(fun (r : region) -> r.label) r);
      text ~title:"current_block"
        (":"
        ^ Option.fold ~none:"<invalid>" ~some:(fun (b : block) -> b.label) b);
      text ~title:"current_line" (string_of_int s.current_line);
      text ~title:"halted" (string_of_bool s.halted);
      table "vregs" [ "vreg"; "value" ]
        (List.map (fun (k, v) -> [ "%" ^ k; value_print v ]) (sorted s.vregs));
      table "static" [ "label"; "value" ]
        (List.map
           (fun (d : static_def) ->
             [
               "@" ^ d.label;
               value_print
                 (Option.value
                    (List.assoc_opt d.label s.static_mem)
                    ~default:(Num N.zero));
             ])
           s.code.statics);
      table "stack" [ "index"; "value" ]
        (List.mapi (fun i v -> [ string_of_int i; value_print v ]) s.stack);
      table "heap"
        [ "handle"; "index"; "value" ]
        (List.concat_map
           (fun (h, vs) ->
             List.mapi
               (fun i v -> [ string_of_int h; string_of_int i; value_print v ])
               vs)
           (sorted s.heap));
    ]

let routput_json s = `String s
let foutput_json e = `String (print_env e)

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
