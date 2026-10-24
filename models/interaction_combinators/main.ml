open Utils_runtime.Machine
module Id = Utils_names.Identifier
module Code = Net
module Ainput = Utils_data.Text.Unit
module Routput = Utils_data.Text.String
module Foutput = Net

module Rinput = struct
  type t = (Id.t * Id.t) option

  let parse text =
    if String.trim text = "" then Ok None
    else
      Utils_parsing.Parser.parse
        (fun p ->
          let a = Utils_parsing.Parser.identifier p in
          let b = Utils_parsing.Parser.identifier p in
          Some (a, b))
        text

  let print = function
    | None -> ""
    | Some (a, b) -> Id.to_string a ^ " " ^ Id.to_string b
end

type state = { net : Net.t; steps : int } [@@deriving yojson]

let make net () =
  let* net = Net.validate net in
  Ok { net; steps = 0 }

let step s selection =
  match (Net.active_pairs s.net, selection) with
  | [], None -> Ok (Halt s.net)
  | pairs, selection ->
      let pair =
        match selection with Some pair -> pair | None -> List.hd pairs
      in
      if s.steps = max_int then Error "Step count overflow"
      else
        let* net, output = Net.rewrite s.net pair in
        Ok (Continue { next = { net; steps = s.steps + 1 }; output })

module Snapshot = struct
  type t = state

  let to_yojson = state_to_yojson

  let of_yojson json =
    let* s = state_of_yojson json in
    if s.steps < 0 then Error "Negative step count" else Ok s
end

let render s =
  let open Utils_runtime.Render in
  let active = Net.active_pairs s.net in
  let next = match active with [] -> None | pair :: _ -> Some pair in
  let endpoint = function
    | Net.Agent (id, port) -> ("agent:" ^ Id.to_string id, port)
    | Net.Free id -> ("free:" ^ Id.to_string id, 0)
  in
  let free =
    List.fold_left
      (fun free (a, b) ->
        List.fold_left
          (fun free -> function Net.Free id -> Id.Set.add id free | _ -> free)
          free [ a; b ])
      Id.Set.empty s.net.wires
  in
  let nodes =
    List.map
      (fun (a : Net.agent) ->
        {
          id = "agent:" ^ Id.to_string a.id;
          label = Net.symbol a.kind ^ " " ^ Id.to_string a.id;
          ports = List.init (Net.arity a.kind + 1) string_of_int;
        })
      s.net.agents
    @ List.map
        (fun id ->
          {
            id = "free:" ^ Id.to_string id;
            label = "@" ^ Id.to_string id;
            ports = [ "" ];
          })
        (Id.Set.elements free)
  in
  let edges =
    List.map
      (fun (a, b) ->
        let highlighted =
          match (a, b, next) with
          | Net.Agent (a, 0), Net.Agent (b, 0), Some (x, y) ->
              Id.equal a x && Id.equal b y
          | _ -> false
        in
        { source = endpoint a; target = endpoint b; highlighted })
      s.net.wires
  in
  [
    text ~title:"steps" (string_of_int s.steps);
    text ~title:"next interaction"
      (match next with
      | None -> "No active pair"
      | Some (a, b) -> Id.to_string a ^ " ↔ " ^ Id.to_string b);
    graph ~title:"net (principal port = 0)" ~nodes ~edges ();
    text_table ~title:"active pairs" ~columns:[ "agent"; "agent" ]
      (List.map (fun (a, b) -> [ Id.to_string a; Id.to_string b ]) active);
    text_table ~title:"wires" ~columns:[ "port"; "port" ]
      (List.map
         (fun (a, b) -> [ Net.port_name a; Net.port_name b ])
         s.net.wires);
    text ~title:"cyclic wires without ports" (string_of_int s.net.loops);
  ]
