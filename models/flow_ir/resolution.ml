open Types

module Target_names = Utils_names.Naming.Make (struct
  type t = code_target

  let compare = Stdlib.compare
end)

module Registers = Utils_names.Intern.Make (Utils_names.Identifier) (Vreg_id)

let unique kind names =
  Utils_names.Identifier.check_unique
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
