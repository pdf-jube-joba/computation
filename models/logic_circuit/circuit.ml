module Vertex_id = Utils_names.Id.Make ()
module Vertex_names = Utils_names.Naming.Make (Vertex_id)
module Vertices = Utils_names.Intern.Make (Utils_names.Identifier) (Vertex_id)

type pin = Utils_names.Identifier.t [@@deriving yojson]

(* Public references distinguish external ports, named local vertices, and
   the IDs of appended iterator instances. Names are resolved within a circuit. *)
type owner =
  | External of Utils_names.Identifier.t
  | Internal of Utils_names.Identifier.t
  | Instance of Vertex_id.t
[@@deriving yojson]

type named_pin = owner * pin [@@deriving yojson]
type internal_pin = Utils_names.Identifier.t * pin [@@deriving yojson]
type signal = (named_pin * bool) list [@@deriving yojson]
type gate_kind = Cst | Not | And | Or | Br | Delay | End [@@deriving yojson]
type gate = { gate_kind : gate_kind; state : bool } [@@deriving yojson]

type t = Gate of gate | Mix of mix | Iter of iter

and mix = {
  kind : Utils_names.Identifier.t;
  verts : (Utils_names.Identifier.t * t) list;
  edges : (internal_pin * internal_pin) list;
  inpin_maps : (pin * internal_pin) list;
  otpin_maps : (pin * internal_pin) list;
}

and iter = {
  kind : Utils_names.Identifier.t;
  init : t;
  used : t list;
  next_edges : (pin * pin) list;
  prev_edges : (pin * pin) list;
  inpin_maps : (pin * internal_pin) list;
  otpin_maps : (pin * internal_pin) list;
}
[@@deriving yojson]

type graph = {
  verts : (owner * t) list;
  edges : (named_pin * named_pin) list;
  inpins_map : (named_pin * named_pin) list;
  otpins_map : (named_pin * named_pin) list;
}
[@@deriving yojson]

let id n =
  match Vertex_id.of_int n with Ok id -> id | Error e -> invalid_arg e

let instance n pin = (Instance (id n), pin)
let internal (name, pin) = (Internal name, pin)

let owner_text = function
  | External name -> "@" ^ name
  | Internal name -> name
  | Instance id -> "$" ^ string_of_int (Vertex_id.to_int id)

let gate_name = function
  | Cst -> "CST"
  | Not -> "NOT"
  | And -> "AND"
  | Or -> "OR"
  | Br -> "BR"
  | Delay -> "DLY"
  | End -> "END"

let kind = function
  | Gate g -> gate_name g.gate_kind
  | Mix m -> m.kind
  | Iter i -> i.kind

let state_as_gate = function Gate g -> Some g.state | _ -> None
let new_gate gate_kind state = Gate { gate_kind; state }

let map_internal kind maps ((name, pin) as endpoint) =
  match List.find_opt (fun (_, target) -> target = endpoint) maps with
  | Some (exported, _) -> (External kind, exported)
  | None -> (Internal name, pin)

let rec get_inpins = function
  | Gate g ->
      List.map
        (fun p -> (External (gate_name g.gate_kind), p))
        (match g.gate_kind with
        | Cst -> []
        | And | Or -> [ "IN0"; "IN1" ]
        | _ -> [ "IN" ])
  | Mix m -> List.map (map_internal m.kind m.inpin_maps) (usable_mix_inpins m)
  | Iter i ->
      if i.used = [] then
        List.map (fun (_, p) -> (External i.kind, p)) (get_inpins i.init)
      else usable_iter_inpins i

and get_otpins = function
  | Gate g ->
      List.map
        (fun p -> (External (gate_name g.gate_kind), p))
        (match g.gate_kind with
        | End -> []
        | Br -> [ "OUT0"; "OUT1" ]
        | _ -> [ "OUT" ])
  | Mix m -> List.map (map_internal m.kind m.otpin_maps) (usable_mix_otpins m)
  | Iter i -> usable_iter_otpins i

and usable_mix_inpins (m : mix) =
  List.concat_map
    (fun (name, c) ->
      get_inpins c
      |> List.filter_map (fun (_, p) ->
          let endpoint = (name, p) in
          if List.exists (fun (_, target) -> target = endpoint) m.edges then
            None
          else Some endpoint))
    m.verts

and usable_mix_otpins (m : mix) =
  List.concat_map
    (fun (name, c) ->
      get_otpins c
      |> List.filter_map (fun (_, p) ->
          let endpoint = (name, p) in
          if List.mem_assoc endpoint m.edges then None else Some endpoint))
    m.verts

and usable_iter_inpins (i : iter) =
  List.concat
    (List.mapi
       (fun n c ->
         get_inpins c
         |> List.filter_map (fun (_, p) ->
             if
               (n = 0
               || not (List.exists (fun (_, target) -> target = p) i.next_edges)
               )
               && not (List.exists (fun (_, target) -> target = p) i.prev_edges)
             then Some (instance n p)
             else None))
       i.used)

and usable_iter_otpins (i : iter) =
  List.concat
    (List.mapi
       (fun n c ->
         get_otpins c
         |> List.filter_map (fun (_, p) ->
             if
               (n = 0 || not (List.mem_assoc p i.prev_edges))
               && not (List.mem_assoc p i.next_edges)
             then Some (instance n p)
             else None))
       i.used)

let rec get_otputs = function
  | Gate g as c -> List.map (fun pin -> (pin, g.state)) (get_otpins c)
  | Mix m ->
      List.concat_map
        (fun (name, c) ->
          get_otputs c
          |> List.filter_map (fun ((_, pin), value) ->
              let endpoint = (name, pin) in
              if List.mem_assoc endpoint m.edges then None
              else Some (map_internal m.kind m.otpin_maps endpoint, value)))
        m.verts
  | Iter i ->
      List.concat
        (List.mapi
           (fun n c ->
             get_otputs c
             |> List.filter_map (fun ((_, p), value) ->
                 if
                   (n = 0 || not (List.mem_assoc p i.prev_edges))
                   && not (List.mem_assoc p i.next_edges)
                 then Some (instance n p, value)
                 else None))
           i.used)

let check_mix (m : mix) =
  Utils_names.Identifier.check_unique
    ~message:(fun name -> "duplicate vertex name: " ^ name)
    (List.map fst m.verts);
  let inputs =
    List.concat_map
      (fun (name, c) -> List.map (fun (_, p) -> (name, p)) (get_inpins c))
      m.verts
  in
  let outputs =
    List.concat_map
      (fun (name, c) -> List.map (fun (_, p) -> (name, p)) (get_otpins c))
      m.verts
  in
  let available_in = ref inputs and available_out = ref outputs in
  let consume available endpoint =
    if not (List.mem endpoint !available) then
      failwith "edge endpoint not in vertices";
    available := List.filter (( <> ) endpoint) !available
  in
  List.iter
    (fun (a, b) ->
      consume available_out a;
      consume available_in b)
    m.edges;
  List.iter
    (fun (_, target) ->
      if not (List.mem target inputs) then
        failwith "input map endpoint not in vertices")
    m.inpin_maps;
  List.iter
    (fun (_, target) ->
      if not (List.mem target outputs) then
        failwith "output map endpoint not in vertices")
    m.otpin_maps

let validate_identifiers circuit =
  Utils_base.Error.protect (fun () ->
      let identifier name =
        match Utils_names.Identifier.make name with
        | Ok _ -> ()
        | Error e -> failwith e
      in
      let endpoint (name, pin) =
        identifier name;
        identifier pin
      in
      let maps entries =
        List.iter
          (fun (p, target) ->
            identifier p;
            endpoint target)
          entries
      in
      let rec check = function
        | Gate _ -> ()
        | Mix m ->
            identifier m.kind;
            List.iter
              (fun (name, c) ->
                identifier name;
                check c)
              m.verts;
            List.iter
              (fun (a, b) ->
                endpoint a;
                endpoint b)
              m.edges;
            maps m.inpin_maps;
            maps m.otpin_maps;
            check_mix m
        | Iter i ->
            identifier i.kind;
            check i.init;
            List.iter check i.used;
            List.iter
              (fun (a, b) ->
                identifier a;
                identifier b)
              (i.next_edges @ i.prev_edges);
            maps i.inpin_maps;
            maps i.otpin_maps
      in
      check circuit)

let new_mix kind verts edges inpin_maps otpin_maps =
  let circuit = Mix { kind; verts; edges; inpin_maps; otpin_maps } in
  Result.map (fun () -> circuit) (validate_identifiers circuit)

let new_iter kind init next_edges prev_edges =
  let circuit =
    Iter
      {
        kind;
        init;
        used = [];
        next_edges;
        prev_edges;
        inpin_maps = [];
        otpin_maps = [];
      }
  in
  Result.map (fun () -> circuit) (validate_identifiers circuit)

let input_value inputs pin =
  match List.find_opt (fun ((_, p), _) -> p = pin) inputs with
  | Some (_, value) -> value
  | None -> false

let add_input inputs (pin, value) =
  if List.mem_assoc pin inputs then inputs else inputs @ [ (pin, value) ]

(* A parent delivers an interface pin to the child's own endpoint. The parent
   vertex name is never reused as the child's external-circuit identity. *)
let deliver child inputs =
  List.filter_map
    (fun (pin, value) ->
      match List.find_opt (fun (_, p) -> p = pin) (get_inpins child) with
      | Some endpoint -> Some (endpoint, value)
      | None -> None)
    inputs

let rec step_exn circuit inputs =
  match circuit with
  | Gate g ->
      let state =
        match g.gate_kind with
        | Cst -> g.state
        | Not -> not (input_value inputs "IN")
        | And -> input_value inputs "IN0" && input_value inputs "IN1"
        | Or -> input_value inputs "IN0" || input_value inputs "IN1"
        | Br | Delay | End -> input_value inputs "IN"
      in
      Gate { g with state }
  | Mix m ->
      check_mix m;
      let ids = ref Vertices.empty in
      let vertices =
        List.map
          (fun (name, child) -> (Vertices.take name ids, name, child))
          m.verts
      in
      let find name = Vertices.find name !ids in
      let pending = List.map (fun (id, _, _) -> (id, ref [])) vertices in
      let send id pin value =
        let signals = List.assoc id pending in
        signals := add_input !signals (pin, value)
      in
      List.iter
        (fun (source, name, child) ->
          List.iter
            (fun ((_, pin), value) ->
              match List.assoc_opt (name, pin) m.edges with
              | Some (target, pin) -> (
                  match find target with
                  | Some id -> send id pin value
                  | None -> assert false)
              | None -> ())
            (get_otputs child);
          ignore source)
        vertices;
      List.iter
        (fun ((owner, pin), value) ->
          let target =
            match owner with
            | External name when name = m.kind ->
                List.assoc_opt pin m.inpin_maps
            | Internal name -> Some (name, pin)
            | _ -> None
          in
          match target with
          | Some (name, pin) -> (
              match find name with Some id -> send id pin value | None -> ())
          | None -> ())
        inputs;
      Mix
        {
          m with
          verts =
            List.map
              (fun (id, name, child) ->
                (name, step_exn child (deliver child !(List.assoc id pending))))
              vertices;
        }
  | Iter i ->
      let count = List.length i.used in
      let pending = Array.make (count + 1) [] in
      List.iteri
        (fun n c ->
          List.iter
            (fun ((_, p), value) ->
              (match List.assoc_opt p i.next_edges with
              | Some target ->
                  pending.(n + 1) <- add_input pending.(n + 1) (target, value)
              | None -> ());
              match List.assoc_opt p i.prev_edges with
              | Some target ->
                  if n = 0 then
                    failwith
                      "Iterator predecessor of index zero is out of bounds";
                  pending.(n - 1) <- add_input pending.(n - 1) (target, value)
              | None -> ())
            (get_otputs c))
        i.used;
      List.iter
        (fun ((owner, p), value) ->
          let n =
            match owner with
            | Instance id -> Some (Vertex_id.to_int id)
            | External name when name = i.kind -> Some count
            | _ -> None
          in
          match n with
          | Some n when n <= count ->
              pending.(n) <- add_input pending.(n) (p, value)
          | Some _ -> failwith "Iterator input ID is out of bounds"
          | None -> ())
        inputs;
      let used =
        List.mapi (fun n c -> step_exn c (deliver c pending.(n))) i.used
      in
      let used =
        if List.exists snd pending.(count) then
          used @ [ step_exn i.init (deliver i.init pending.(count)) ]
        else used
      in
      Iter { i with used }

let step c inputs = Utils_base.Error.protect (fun () -> step_exn c inputs)
