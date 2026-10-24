module Id = Utils_names.Identifier
module P = Utils_parsing.Parser
open Utils_base.Error

type kind = Gamma | Delta | Epsilon [@@deriving yojson]
type agent = { id : Id.t; kind : kind } [@@deriving yojson]
type port = Agent of Id.t * int | Free of Id.t [@@deriving yojson]

type t = { agents : agent list; wires : (port * port) list; loops : int }
[@@deriving yojson]

module Ports = Map.Make (struct
  type t = port

  let compare = Stdlib.compare
end)

let arity = function Gamma | Delta -> 2 | Epsilon -> 0

let kind_name = function
  | Gamma -> "gamma"
  | Delta -> "delta"
  | Epsilon -> "epsilon"

let symbol = function Gamma -> "γ" | Delta -> "δ" | Epsilon -> "ε"

let port_name = function
  | Agent (id, i) -> Id.to_string id ^ "." ^ string_of_int i
  | Free id -> "@" ^ Id.to_string id

let wire a b = if compare a b <= 0 then (a, b) else (b, a)

let normalize net =
  {
    net with
    agents = List.sort (fun a b -> Id.compare a.id b.id) net.agents;
    wires = List.sort compare (List.map (fun (a, b) -> wire a b) net.wires);
  }

let validate net =
  protect (fun () ->
      if net.loops < 0 then failwith "Negative cyclic wire count";
      let agents =
        List.fold_left
          (fun agents a ->
            if Id.Map.mem a.id agents then
              failwith ("Duplicate agent: " ^ Id.to_string a.id);
            Id.Map.add a.id a.kind agents)
          Id.Map.empty net.agents
      in
      let use seen p =
        (match p with
        | Free _ -> ()
        | Agent (id, i) -> (
            match Id.Map.find_opt id agents with
            | None -> failwith ("Unknown agent: " ^ Id.to_string id)
            | Some kind ->
                if i < 0 || i > arity kind then
                  failwith ("Invalid port: " ^ port_name p)));
        if Ports.mem p seen then
          failwith ("Port used more than once: " ^ port_name p);
        Ports.add p () seen
      in
      let used =
        List.fold_left
          (fun seen (a, b) -> use (use seen a) b)
          Ports.empty net.wires
      in
      List.iter
        (fun a ->
          for i = 0 to arity a.kind do
            let p = Agent (a.id, i) in
            if not (Ports.mem p used) then
              failwith ("Unconnected port: " ^ port_name p)
          done)
        net.agents;
      normalize net)

let of_yojson json = Result.bind (of_yojson json) validate

let parse text =
  let* net =
    P.parse
      (fun p ->
        let agents = ref [] and wires = ref [] and loops = ref None in
        let port p =
          if P.bump_if_symbol p '@' then Free (P.identifier p)
          else
            let id = P.identifier p in
            P.expect_symbol p '.';
            Agent (id, P.integer p)
        in
        while not (P.eof p) do
          (match P.peek p with
          | P.Keyword _ ->
              let keyword = P.keyword p in
              if keyword = "loops" then (
                if !loops <> None then P.error p "Duplicate \\loops declaration";
                loops := Some (P.integer p))
              else
                let kind =
                  match keyword with
                  | "gamma" -> Gamma
                  | "delta" -> Delta
                  | "epsilon" -> Epsilon
                  | _ ->
                      P.error p
                        "Expected \\gamma, \\delta, \\epsilon or \\loops"
                in
                agents := { id = P.identifier p; kind } :: !agents
          | _ ->
              let a = port p in
              P.expect_symbol p '~';
              let b = port p in
              wires := (a, b) :: !wires);
          P.expect_symbol p ';'
        done;
        {
          agents = !agents;
          wires = !wires;
          loops = Option.value !loops ~default:0;
        })
      text
  in
  validate net

let print net =
  let net = normalize net in
  let agents =
    List.map
      (fun a -> "\\" ^ kind_name a.kind ^ " " ^ Id.to_string a.id ^ ";")
      net.agents
  in
  let wires =
    List.map (fun (a, b) -> port_name a ^ " ~ " ^ port_name b ^ ";") net.wires
  in
  String.concat "\n"
    (agents @ wires
    @
    if net.loops = 0 then [] else [ "\\loops " ^ string_of_int net.loops ^ ";" ]
    )

let active_pairs net =
  List.filter_map
    (function Agent (a, 0), Agent (b, 0) -> Some (a, b) | _ -> None)
    (normalize net).wires

(* Glue the old context to the rule's replacement. Removed auxiliary ports
   have degree two, so each component is either a path with two surviving
   endpoints or a closed wire. This also handles connections inside the redex. *)
let splice ~removed ~loops edges =
  let add a b map =
    Ports.update a
      (function None -> Some [ b ] | Some xs -> Some (b :: xs))
      map
  in
  let adjacency =
    List.fold_left (fun map (a, b) -> add b a (add a b map)) Ports.empty edges
  in
  let seen = ref Ports.empty and wires = ref [] and loops = ref loops in
  let rec visit ends = function
    | [] -> ends
    | p :: todo when Ports.mem p !seen -> visit ends todo
    | p :: todo ->
        seen := Ports.add p () !seen;
        let ends = if removed p then ends else p :: ends in
        visit ends (Ports.find p adjacency @ todo)
  in
  Ports.iter
    (fun p _ ->
      if not (Ports.mem p !seen) then
        match visit [] [ p ] with
        | [ a; b ] -> wires := wire a b :: !wires
        | [] ->
            if !loops = max_int then failwith "Cyclic wire count overflow";
            incr loops
        | _ -> failwith "Invalid rewrite wiring")
    adjacency;
  (!wires, !loops)

let rewrite net (a, b) =
  protect (fun () ->
      let a, b = if Id.compare a b <= 0 then (a, b) else (b, a) in
      if not (List.mem (a, b) (active_pairs net)) then
        failwith "Selected agents are not an active pair";
      let kind id =
        (List.find (fun node -> Id.equal node.id id) net.agents).kind
      in
      let ka = kind a and kb = kind b in
      let deleted id = Id.equal id a || Id.equal id b in
      let removed = function Agent (id, _) -> deleted id | Free _ -> false in
      let agents =
        ref (List.filter (fun node -> not (deleted node.id)) net.agents)
      in
      let used =
        ref (Id.Set.of_list (List.map (fun node -> node.id) net.agents))
      in
      let fresh kind =
        let id = Id.fresh ~prefix:"n" !used in
        used := Id.Set.add id !used;
        agents := { id; kind } :: !agents;
        id
      in
      let p id i = Agent (id, i) in
      let replacement =
        match (ka, kb) with
        | Epsilon, Epsilon -> []
        | Gamma, Gamma -> [ (p a 1, p b 2); (p a 2, p b 1) ]
        | Delta, Delta -> [ (p a 1, p b 1); (p a 2, p b 2) ]
        | Epsilon, _ | _, Epsilon ->
            let node = if ka = Epsilon then b else a in
            List.init 2 (fun i -> (p node (i + 1), p (fresh Epsilon) 0))
        | Gamma, Delta | Delta, Gamma ->
            let g, d = if ka = Gamma then (a, b) else (b, a) in
            let d1 = fresh Delta in
            let d2 = fresh Delta in
            let g1 = fresh Gamma in
            let g2 = fresh Gamma in
            [
              (p g 1, p d1 0);
              (p g 2, p d2 0);
              (p d 1, p g1 0);
              (p d 2, p g2 0);
              (p d1 1, p g1 1);
              (p d1 2, p g2 1);
              (p d2 1, p g1 2);
              (p d2 2, p g2 2);
            ]
      in
      let principal = wire (p a 0) (p b 0) in
      let edges =
        List.filter (fun (x, y) -> wire x y <> principal) net.wires
        @ replacement
      in
      let wires, loops = splice ~removed ~loops:net.loops edges in
      let next = get (validate { agents = !agents; wires; loops }) in
      let left, right = if compare ka kb <= 0 then (ka, kb) else (kb, ka) in
      (next, kind_name left ^ "-" ^ kind_name right))
