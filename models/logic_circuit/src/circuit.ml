type pin = string [@@deriving yojson]
type named_pin = string * pin [@@deriving yojson]
type signal = (named_pin * bool) list [@@deriving yojson]
type gate_kind = Cst | Not | And | Or | Br | Delay | End [@@deriving yojson]
type gate = { gate_kind : gate_kind; state : bool } [@@deriving yojson]

type t = Gate of gate | Mix of mix | Iter of iter

and mix = {
  kind : string;
  verts : (string * t) list;
  edges : (named_pin * named_pin) list;
  inpin_maps : (pin * named_pin) list;
  otpin_maps : (pin * named_pin) list;
}

and iter = {
  kind : string;
  init : t;
  used : t list;
  next_edges : (pin * pin) list;
  prev_edges : (pin * pin) list;
  inpin_maps : (pin * named_pin) list;
  otpin_maps : (pin * named_pin) list;
}
[@@deriving yojson]

type graph = {
  verts : (string * t) list;
  edges : (named_pin * named_pin) list;
  inpins_map : (named_pin * named_pin) list;
  otpins_map : (named_pin * named_pin) list;
}
[@@deriving yojson]

let validate_identifiers circuit =
  Utils.Json.protect (fun () ->
      let identifier name =
        match Utils.Identifier.make name with
        | Ok _ -> ()
        | Error message -> failwith message
      in
      let named_pin (name, pin) =
        identifier name;
        identifier pin
      in
      let maps entries =
        List.iter
          (fun (pin, target) ->
            identifier pin;
            named_pin target)
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
              (fun (source, target) ->
                named_pin source;
                named_pin target)
              m.edges;
            maps m.inpin_maps;
            maps m.otpin_maps
        | Iter i ->
            identifier i.kind;
            check i.init;
            List.iter check i.used;
            List.iter
              (fun (o, p) ->
                identifier o;
                identifier p)
              (i.next_edges @ i.prev_edges);
            maps i.inpin_maps;
            maps i.otpin_maps
      in
      check circuit)

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
let num_to_ident n = Printf.sprintf "_%d_" n

let map_internal kind maps internal =
  match List.find_opt (fun (_, target) -> target = internal) maps with
  | Some (external_pin, _) -> (kind, external_pin)
  | None -> internal

let resolve_input kind maps ((name, pin) as input) =
  if name = kind then List.assoc_opt pin maps else Some input

let rec get_inpins = function
  | Gate g ->
      List.map
        (fun pin -> (gate_name g.gate_kind, pin))
        (match g.gate_kind with
        | Cst -> []
        | And | Or -> [ "IN0"; "IN1" ]
        | _ -> [ "IN" ])
  | Mix m -> List.map (map_internal m.kind m.inpin_maps) (usable_mix_inpins m)
  | Iter i -> List.map (map_internal i.kind i.inpin_maps) (usable_iter_inpins i)

and get_otpins = function
  | Gate g ->
      List.map
        (fun pin -> (gate_name g.gate_kind, pin))
        (match g.gate_kind with
        | End -> []
        | Br -> [ "OUT0"; "OUT1" ]
        | _ -> [ "OUT" ])
  | Mix m -> List.map (map_internal m.kind m.otpin_maps) (usable_mix_otpins m)
  | Iter i -> List.map (map_internal i.kind i.otpin_maps) (usable_iter_otpins i)

and usable_mix_inpins (m : mix) =
  List.concat_map
    (fun (name, c) ->
      List.filter
        (fun pin -> not (List.exists (fun (_, target) -> target = pin) m.edges))
        (List.map (fun (_, pin) -> (name, pin)) (get_inpins c)))
    m.verts

and usable_mix_otpins (m : mix) =
  List.concat_map
    (fun (name, c) ->
      List.filter
        (fun pin -> not (List.mem_assoc pin m.edges))
        (List.map (fun (_, pin) -> (name, pin)) (get_otpins c)))
    m.verts

and usable_iter_inpins (i : iter) =
  List.concat
    (List.mapi
       (fun n c ->
         get_inpins c
         |> List.filter_map (fun (_, pin) ->
             if
               (n = 0
               || not
                    (List.exists (fun (_, target) -> target = pin) i.next_edges)
               )
               && not
                    (List.exists (fun (_, target) -> target = pin) i.prev_edges)
             then Some (num_to_ident n, pin)
             else None))
       i.used)

and usable_iter_otpins (i : iter) =
  List.concat
    (List.mapi
       (fun n c ->
         get_otpins c
         |> List.filter_map (fun (_, pin) ->
             if
               (n = 0 || not (List.mem_assoc pin i.prev_edges))
               && not (List.mem_assoc pin i.next_edges)
             then Some (num_to_ident n, pin)
             else None))
       i.used)

let rec get_otputs = function
  | Gate g as c -> List.map (fun pin -> (pin, g.state)) (get_otpins c)
  | Mix m ->
      List.concat_map
        (fun (name, c) ->
          get_otputs c
          |> List.filter_map (fun ((_, pin), value) ->
              let internal = (name, pin) in
              if List.mem_assoc internal m.edges then None
              else Some (map_internal m.kind m.otpin_maps internal, value)))
        m.verts
  | Iter i ->
      List.concat
        (List.mapi
           (fun n c ->
             get_otputs c
             |> List.filter_map (fun ((_, pin), value) ->
                 if
                   (n = 0 || not (List.mem_assoc pin i.prev_edges))
                   && not (List.mem_assoc pin i.next_edges)
                 then
                   Some
                     ( map_internal i.kind i.otpin_maps (num_to_ident n, pin),
                       value )
                 else None))
           i.used)

let new_mix kind verts edges inpin_maps otpin_maps =
  Utils.Json.protect (fun () ->
      let names = List.map fst verts in
      if List.length names <> List.length (List.sort_uniq String.compare names)
      then failwith "duplicate vertex name";
      let available_in =
        ref
          (List.concat_map
             (fun (n, c) -> List.map (fun (_, p) -> (n, p)) (get_inpins c))
             verts)
      in
      let available_out =
        ref
          (List.concat_map
             (fun (n, c) -> List.map (fun (_, p) -> (n, p)) (get_otpins c))
             verts)
      in
      let consume available ((n, p) as pin) =
        if not (List.mem pin !available) then
          failwith ("edge " ^ n ^ "." ^ p ^ " not in verts");
        available := List.filter (( <> ) pin) !available
      in
      List.iter
        (fun (source, target) ->
          consume available_out source;
          consume available_in target)
        edges;
      (* The Rust constructor deliberately did not validate the external maps. *)
      Mix { kind; verts; edges; inpin_maps; otpin_maps })

let new_iter kind init next_edges prev_edges =
  Ok
    (Iter
       {
         kind;
         init;
         used = [];
         next_edges;
         prev_edges;
         inpin_maps = [];
         otpin_maps = [];
       })

let input_value inputs pin =
  match List.find_opt (fun ((_, p), _) -> p = pin) inputs with
  | Some (_, b) -> b
  | None -> false

let add_external inputs (pin, value) =
  if List.mem_assoc pin inputs then inputs else inputs @ [ (pin, value) ]

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
      let pending =
        List.fold_left
          (fun pending (name, _) ->
            if List.mem_assoc name pending then pending
            else (name, ref []) :: pending)
          [] m.verts
      in
      (* Read all old outputs before changing any vertex. *)
      List.iter
        (fun (name, c) ->
          List.iter
            (fun ((_, pin), value) ->
              match List.assoc_opt (name, pin) m.edges with
              | Some ((target, _) as pin) ->
                  let signals = List.assoc target pending in
                  signals := !signals @ [ (pin, value) ]
              | None -> ())
            (get_otputs c))
        m.verts;
      List.iter
        (fun (pin, value) ->
          match resolve_input m.kind m.inpin_maps pin with
          | Some ((name, _) as internal) -> (
              match List.assoc_opt name pending with
              | Some signals ->
                  signals := add_external !signals (internal, value)
              | None -> ())
          | None -> ())
        inputs;
      (* The public Rust record/snapshot can contain duplicate names. Its
         HashMap has one pending entry per name and find_map steps only the
         first corresponding vertex, leaving later duplicates unchanged. *)
      let seen = ref [] in
      let verts =
        List.map
          (fun (name, c) ->
            if List.mem name !seen then (name, c)
            else (
              seen := name :: !seen;
              (name, step_exn c !(List.assoc name pending))))
          m.verts
      in
      Mix { m with verts }
  | Iter i ->
      let count = List.length i.used in
      let pending = Array.make (count + 1) [] in
      List.iteri
        (fun n c ->
          List.iter
            (fun ((_, pin), value) ->
              (match List.assoc_opt pin i.next_edges with
              | Some target ->
                  pending.(n + 1) <-
                    pending.(n + 1)
                    @ [ ((num_to_ident (n + 1), target), value) ]
              | None -> ());
              match List.assoc_opt pin i.prev_edges with
              | Some target ->
                  if n = 0 then
                    failwith
                      "Iterator predecessor of index zero is out of bounds";
                  pending.(n - 1) <-
                    pending.(n - 1)
                    @ [ ((num_to_ident (n - 1), target), value) ]
              | None -> ())
            (get_otputs c))
        i.used;
      List.iter
        (fun (pin, value) ->
          match resolve_input i.kind i.inpin_maps pin with
          | None -> ()
          | Some ((name, _) as internal) -> (
              if
                (* Rust parses a decimal usize here, although its generated labels
               are _n_. Keep those labels ignored rather than inventing routing. *)
                name <> ""
                && String.for_all
                     (function '0' .. '9' -> true | _ -> false)
                     name
              then
                match int_of_string_opt name with
                | Some n when n <= count ->
                    pending.(n) <- add_external pending.(n) (internal, value)
                | _ -> failwith "Iterator input index is out of bounds"))
        inputs;
      let used = List.mapi (fun n c -> step_exn c pending.(n)) i.used in
      let used =
        if List.exists snd pending.(count) then
          used @ [ step_exn i.init pending.(count) ]
        else used
      in
      Iter { i with used }

let step circuit inputs = Utils.Json.protect (fun () -> step_exn circuit inputs)

let as_graph_group = function
  | Gate _ as c ->
      {
        verts = [ (kind c, c) ];
        edges = [];
        inpins_map = List.map (fun p -> (p, p)) (get_inpins c);
        otpins_map = List.map (fun p -> (p, p)) (get_otpins c);
      }
  | Mix m ->
      {
        verts = m.verts;
        edges = m.edges;
        inpins_map =
          List.map
            (fun p -> (map_internal m.kind m.inpin_maps p, p))
            (usable_mix_inpins m);
        otpins_map =
          List.map
            (fun p -> (map_internal m.kind m.otpin_maps p, p))
            (usable_mix_otpins m);
      }
  | Iter i ->
      let edges =
        List.concat
          (List.mapi
             (fun n _ ->
               (if n < List.length i.used - 1 then
                  List.map
                    (fun (o, p) ->
                      ((num_to_ident n, o), (num_to_ident (n + 1), p)))
                    i.next_edges
                else [])
               @
               if n > 0 then
                 List.map
                   (fun (o, p) ->
                     ((num_to_ident n, o), (num_to_ident (n - 1), p)))
                   i.prev_edges
               else [])
             i.used)
      in
      {
        verts = List.mapi (fun n c -> (num_to_ident n, c)) i.used;
        edges;
        inpins_map =
          List.map
            (fun p -> (map_internal i.kind i.inpin_maps p, p))
            (usable_iter_inpins i);
        otpins_map =
          List.map
            (fun p -> (map_internal i.kind i.otpin_maps p, p))
            (usable_iter_otpins i);
      }

let bool_text b = if b then "\\true" else "\\false"

let print_signal signal =
  String.concat ", "
    (List.map (fun ((n, p), b) -> n ^ "." ^ p ^ "=" ^ bool_text b) signal)

let render circuit =
  let graph = as_graph_group circuit in
  let row xs = List.map Utils.Render.text xs in
  let inputs = List.map (fun ((n, p), _) -> row [ n; p ]) graph.inpins_map in
  let outputs =
    List.map
      (fun ((n, p), (vert, pin)) ->
        let value =
          match List.assoc_opt vert graph.verts with
          | None -> "?"
          | Some c -> (
              match
                List.find_opt (fun ((_, p), _) -> p = pin) (get_otputs c)
              with
              | Some (_, b) -> bool_text b
              | None -> "?")
        in
        row [ n; p; value ])
      graph.otpins_map
  in
  let verts =
    List.concat_map
      (fun (name, c) ->
        List.map
          (fun ((_, pin), b) -> row [ name; pin; bool_text b ])
          (get_otputs c))
      graph.verts
  in
  Utils.Render.state
    [
      Utils.Render.text ~title:"kind" (kind circuit);
      Utils.Render.table ~title:"inputs" ~columns:(row [ "name"; "pin" ]) inputs;
      Utils.Render.table ~title:"outputs"
        ~columns:(row [ "name"; "pin"; "value" ])
        outputs;
      Utils.Render.table ~title:"verts"
        ~columns:(row [ "vert"; "pin"; "value" ])
        verts;
    ]
