module Vertex_id = Utils.Id.Make ()
module Vertex_names = Utils.Naming.Make (Vertex_id)
module Vertices = Utils.Intern.Make (Utils.Identifier) (Vertex_id)

type pin = Utils.Identifier.t [@@deriving yojson]

(* Public references distinguish external ports, named local vertices, and
   the IDs of appended iterator instances. Names are resolved within a circuit. *)
type owner =
  | External of Utils.Identifier.t
  | Internal of Utils.Identifier.t
  | Instance of Vertex_id.t
[@@deriving yojson]

type named_pin = owner * pin [@@deriving yojson]
type internal_pin = Utils.Identifier.t * pin [@@deriving yojson]
type signal = (named_pin * bool) list [@@deriving yojson]
type gate_kind = Cst | Not | And | Or | Br | Delay | End [@@deriving yojson]
type gate = { gate_kind : gate_kind; state : bool } [@@deriving yojson]

type t = Gate of gate | Mix of mix | Iter of iter

and mix = {
  kind : Utils.Identifier.t;
  verts : (Utils.Identifier.t * t) list;
  edges : (internal_pin * internal_pin) list;
  inpin_maps : (pin * internal_pin) list;
  otpin_maps : (pin * internal_pin) list;
}

and iter = {
  kind : Utils.Identifier.t;
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
  Utils.Identifier.check_unique
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
  Utils.Json.protect (fun () ->
      let identifier name =
        match Utils.Identifier.make name with
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

let step c inputs = Utils.Json.protect (fun () -> step_exn c inputs)

let as_graph_group = function
  | Gate _ as c ->
      {
        verts = [ (External (kind c), c) ];
        edges = [];
        inpins_map = List.map (fun p -> (p, p)) (get_inpins c);
        otpins_map = List.map (fun p -> (p, p)) (get_otpins c);
      }
  | Mix m ->
      {
        verts = List.map (fun (name, c) -> (Internal name, c)) m.verts;
        edges = List.map (fun (a, b) -> (internal a, internal b)) m.edges;
        inpins_map =
          List.map
            (fun p -> (map_internal m.kind m.inpin_maps p, internal p))
            (usable_mix_inpins m);
        otpins_map =
          List.map
            (fun p -> (map_internal m.kind m.otpin_maps p, internal p))
            (usable_mix_otpins m);
      }
  | Iter i ->
      let edges =
        List.concat
          (List.mapi
             (fun n _ ->
               (if n < List.length i.used - 1 then
                  List.map
                    (fun (a, b) -> (instance n a, instance (n + 1) b))
                    i.next_edges
                else [])
               @
               if n > 0 then
                 List.map
                   (fun (a, b) -> (instance n a, instance (n - 1) b))
                   i.prev_edges
               else [])
             i.used)
      in
      {
        verts = List.mapi (fun n c -> (Instance (id n), c)) i.used;
        edges;
        inpins_map = List.map (fun p -> (p, p)) (usable_iter_inpins i);
        otpins_map = List.map (fun p -> (p, p)) (usable_iter_otpins i);
      }

module Signal = struct
  type t = signal

  let value = function Ok value -> value | Error error -> failwith error
  let parse_bool text = Utils.Bool.to_bool (value (Utils.Bool.parse text))
  let print_bool value = Utils.Bool.print (Utils.Bool.of_bool value)

  module P = Utils.Parser

  let read_pin p =
    let start = P.offset p in
    let owner, end_name =
      if P.bump_if_symbol p '@' then
        let name = P.identifier p in
        (External name, start + 1 + String.length name)
      else if P.bump_if_symbol p '$' then (
        let start_id = P.offset p in
        if start_id <> start + 1 then P.error p "Expected contiguous $ID.PIN";
        let width =
          match P.peek p with P.Number text -> String.length text | _ -> 0
        in
        let value = P.integer p in
        (Instance (id value), start_id + width))
      else
        let name = P.identifier p in
        (Internal name, start + String.length name)
    in
    let dot = P.offset p in
    if dot <> end_name then P.error p "Expected contiguous endpoint.PIN";
    P.expect_symbol p '.';
    if P.offset p <> dot + 1 then P.error p "Expected contiguous endpoint.PIN";
    let pin = P.identifier p in
    (owner, pin)

  let read_item p =
    let pin = read_pin p in
    P.expect_symbol p '=';
    let start = P.offset p in
    P.expect_symbol p '#';
    if P.offset p <> start + 1 then P.error p "Expected #true or #false";
    let boolean = P.identifier p in
    (pin, parse_bool ("#" ^ boolean))

  let parse_pin text = value (P.parse read_pin text)
  let parse_item text = value (P.parse read_item text)

  let print_item ((name, pin), value) =
    owner_text name ^ "." ^ pin ^ "=" ^ print_bool value

  let parse = P.parse (fun p -> P.separated_list p ~separator:',' read_item)
  let print signal = String.concat ", " (List.map print_item signal)
end

let render circuit =
  let graph = as_graph_group circuit in
  let names = ref Vertex_names.empty in
  List.iter
    (fun (owner, _) ->
      match owner with
      | Internal name | External name ->
          names := Vertex_names.reserve name !names
      | Instance _ -> ())
    graph.verts;
  let display = function
    | Instance id ->
        let name, next = Vertex_names.assign ~hint:"instance" id !names in
        names := next;
        name
    | owner -> owner_text owner
  in
  let inputs =
    List.map (fun ((owner, p), _) -> [ display owner; p ]) graph.inpins_map
  in
  let outputs =
    List.map
      (fun ((owner, p), (vert, pin)) ->
        let value =
          match List.assoc_opt vert graph.verts with
          | None -> "?"
          | Some c -> (
              match
                List.find_opt (fun ((_, p), _) -> p = pin) (get_otputs c)
              with
              | Some (_, b) -> Signal.print_bool b
              | None -> "?")
        in
        [ display owner; p; value ])
      graph.otpins_map
  in
  let verts =
    List.concat_map
      (fun (owner, c) ->
        List.map
          (fun ((_, p), v) -> [ display owner; p; Signal.print_bool v ])
          (get_otputs c))
      graph.verts
  in
  Utils.Render.state
    [
      Utils.Render.text ~title:"kind" (kind circuit);
      Utils.Render.text_table ~title:"inputs" ~columns:[ "name"; "pin" ] inputs;
      Utils.Render.text_table ~title:"outputs"
        ~columns:[ "name"; "pin"; "value" ] outputs;
      Utils.Render.text_table ~title:"verts" ~columns:[ "vert"; "pin"; "value" ]
        verts;
    ]
