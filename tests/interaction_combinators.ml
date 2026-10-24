module M = Interaction_combinators.Main
module N = Interaction_combinators.Net
module Id = Utils_names.Identifier
module Machine = Utils_runtime.Machine

let get = Utils_base.Error.get
let check message b = if not b then failwith message
let id = Id.make_exn
let start text = get (M.make (get (M.Code.parse text)) ())

let same_net message actual expected =
  check message (N.print actual = N.print (get (N.parse expected)))

let tick ?(input = "") state =
  let snapshot = M.Snapshot.to_yojson state in
  let restored =
    get
      (M.Snapshot.of_yojson
         (Yojson.Safe.from_string (Yojson.Safe.to_string snapshot)))
  in
  check "snapshot roundtrip" (M.Snapshot.to_yojson restored = snapshot);
  check "render roundtrip" (M.render state = M.render restored);
  let result = get (M.step restored (get (M.Rinput.parse input))) in
  check "step mutated snapshot" (M.Snapshot.to_yojson restored = snapshot);
  check "restored transition differs"
    (result = get (M.step state (get (M.Rinput.parse input))));
  match result with
  | Machine.Halt _ -> failwith "Unexpected halt"
  | Machine.Continue { next; output } ->
      check "rewrite violates net invariants"
        (Result.is_ok (N.validate next.net));
      check "code roundtrip" (get (N.parse (N.print next.net)) = next.net);
      (next, output)

let halt state =
  match get (M.step state None) with
  | Machine.Halt net -> net
  | _ -> failwith "Expected halt"

let pair left right =
  let aux name kind x y =
    if kind = "epsilon" then ""
    else Printf.sprintf "%s.1 ~ @%s; %s.2 ~ @%s;" name x name y
  in
  Printf.sprintf "\\%s a; \\%s b; a.0 ~ b.0; %s %s" left right
    (aux "a" left "x" "y") (aux "b" right "u" "v")

let rules () =
  List.iter
    (fun (left, right, expected, label) ->
      let next, output = tick (start (pair left right)) in
      check "rule output" (output = label);
      same_net "six interaction rules" (halt next) expected)
    [
      ("gamma", "gamma", "@x ~ @v; @y ~ @u;", "gamma-gamma");
      ("delta", "delta", "@x ~ @u; @y ~ @v;", "delta-delta");
      ("epsilon", "epsilon", "", "epsilon-epsilon");
      ( "gamma",
        "epsilon",
        {|\epsilon n0; \epsilon n1; n0.0 ~ @x; n1.0 ~ @y;|},
        "gamma-epsilon" );
      ( "epsilon",
        "gamma",
        {|\epsilon n0; \epsilon n1; n0.0 ~ @u; n1.0 ~ @v;|},
        "gamma-epsilon" );
      ( "delta",
        "epsilon",
        {|\epsilon n0; \epsilon n1; n0.0 ~ @x; n1.0 ~ @y;|},
        "delta-epsilon" );
      ( "epsilon",
        "delta",
        {|\epsilon n0; \epsilon n1; n0.0 ~ @u; n1.0 ~ @v;|},
        "delta-epsilon" );
    ];
  let next, output = tick (start (pair "gamma" "delta")) in
  check "commutation label" (output = "gamma-delta");
  same_net "commutation ports" (halt next)
    {|\delta n0; \delta n1; \gamma n2; \gamma n3;
      n0.0 ~ @x; n1.0 ~ @y; n2.0 ~ @u; n3.0 ~ @v;
      n0.1 ~ n2.1; n0.2 ~ n3.1; n1.1 ~ n2.2; n1.2 ~ n3.2;|}

(* A separate oracle: Lafont's two-stack path semantics (§3.1).
   Gamma flips the popped bit; delta does not. Every rewrite must preserve
   paths between interface ports, for any initial pair of stacks. *)
let trace net start gamma delta =
  let rec walk fuel port gamma delta =
    if fuel = 0 then failwith "Path oracle exhausted fuel";
    let other =
      List.find_map
        (fun (a, b) ->
          if a = port then Some b else if b = port then Some a else None)
        net.N.wires
      |> Option.get
    in
    match other with
    | N.Free name -> Some (name, gamma, delta)
    | N.Agent (name, port) -> (
        let kind =
          (List.find (fun (a : N.agent) -> Id.equal a.id name) net.agents).kind
        in
        let stack = if kind = N.Gamma then gamma else delta in
        let continue port stack =
          let gamma, delta =
            if kind = N.Gamma then (stack, delta) else (gamma, stack)
          in
          walk (fuel - 1) (N.Agent (name, port)) gamma delta
        in
        match (kind, port, stack) with
        | N.Epsilon, _, _ | _, 0, [] -> None
        | _, 0, bit :: stack ->
            continue (if kind = N.Gamma then 3 - bit else bit) stack
        | _, port, stack -> continue 0 (port :: stack))
  in
  walk 100 (N.Free (id start)) gamma delta

let path_semantics () =
  let stacks = [ []; [ 1 ]; [ 2 ]; [ 1; 1 ]; [ 1; 2 ]; [ 2; 1 ]; [ 2; 2 ] ] in
  List.iter
    (fun left ->
      List.iter
        (fun right ->
          let state = start (pair left right) in
          let next, _ = tick state in
          let roots =
            (if left = "epsilon" then [] else [ "x"; "y" ])
            @ if right = "epsilon" then [] else [ "u"; "v" ]
          in
          List.iter
            (fun root ->
              List.iter
                (fun gamma ->
                  List.iter
                    (fun delta ->
                      check "rewrite changed two-stack path semantics"
                        (trace state.net root gamma delta
                        = trace next.net root gamma delta))
                    stacks)
                stacks)
            roots)
        [ "gamma"; "delta"; "epsilon" ])
    [ "gamma"; "delta"; "epsilon" ]

let loops_and_context () =
  List.iter
    (fun (kind, context, count) ->
      let state =
        start (Printf.sprintf "\\%s a; \\%s b; a.0 ~ b.0; %s" kind kind context)
      in
      let next, _ = tick state in
      check "closed wire accounting" ((halt next).loops = count))
    [
      ("gamma", "a.1 ~ b.2; a.2 ~ b.1;", 2);
      ("gamma", "a.1 ~ b.1; a.2 ~ b.2;", 1);
      ("gamma", "a.1 ~ a.2; b.1 ~ b.2;", 1);
      ("delta", "a.1 ~ b.1; a.2 ~ b.2;", 2);
      ("delta", "a.1 ~ b.2; a.2 ~ b.1;", 1);
      ("delta", "a.1 ~ a.2; b.1 ~ b.2;", 1);
    ];
  let erased, _ =
    tick (start {|\gamma a; \epsilon b; a.0 ~ b.0; a.1 ~ a.2;|})
  in
  check "erasure creates a new active pair"
    (List.length (N.active_pairs erased.net) = 1);
  same_net "erase internal wire" (halt (fst (tick erased))) "";
  same_net "free wires and cyclic wires remain"
    (halt (start {|@x ~ @y; \loops 3;|}))
    {|@x ~ @y; \loops 3;|};
  let stuck = start {|\gamma a; a.0 ~ a.1; a.2 ~ @out;|} in
  check "principal-to-auxiliary is not active" (N.active_pairs stuck.net = []);
  ignore (halt stuck);
  let initial =
    start
      {|\gamma a; \gamma b; \delta c; \delta d;
      a.0 ~ b.0; c.0 ~ d.0; a.1 ~ c.1; a.2 ~ @x; b.1 ~ @y;
      b.2 ~ c.2; d.1 ~ @u; d.2 ~ @v;|}
  in
  let reduce x y = fst (tick ~input:y (fst (tick ~input:x initial))) |> halt in
  check "independent interactions commute"
    (reduce "a b" "c d" = reduce "d c" "b a");
  same_net "context rewiring" (reduce "a b" "c d") "@x ~ @y; @u ~ @v;"

let replace key value = function
  | `Assoc fields ->
      `Assoc (List.map (fun (k, v) -> (k, if k = key then value else v)) fields)
  | _ -> failwith "Expected object"

let duplicate_tree () =
  let initial =
    start
      {|\delta dup; \gamma pair; \epsilon l; \epsilon r;
      dup.0 ~ pair.0; dup.1 ~ @left; dup.2 ~ @right;
      pair.1 ~ l.0; pair.2 ~ r.0;|}
  in
  let state = fst (tick (fst (tick (fst (tick initial))))) in
  let net = halt state in
  check "duplicate tree has two roots and four leaves"
    (List.length net.agents = 6
    && List.length
         (List.filter (fun (a : N.agent) -> a.kind = N.Gamma) net.agents)
       = 2
    && List.length
         (List.filter (fun (a : N.agent) -> a.kind = N.Epsilon) net.agents)
       = 4);
  let neighbour p =
    List.find_map
      (fun (a, b) -> if a = p then Some b else if b = p then Some a else None)
      net.wires
    |> Option.get
  in
  List.iter
    (fun name ->
      match neighbour (N.Free (id name)) with
      | N.Agent (root, 0) ->
          check "root is gamma"
            ((List.find (fun (a : N.agent) -> Id.equal a.id root) net.agents)
               .kind = N.Gamma);
          List.iter
            (fun i ->
              match neighbour (N.Agent (root, i)) with
              | N.Agent (leaf, 0) ->
                  check "leaf is epsilon"
                    ((List.find
                        (fun (a : N.agent) -> Id.equal a.id leaf)
                        net.agents)
                       .kind = N.Epsilon)
              | _ -> failwith "Invalid leaf connection")
            [ 1; 2 ]
      | _ -> failwith "Invalid root connection")
    [ "left"; "right" ]

let validation () =
  List.iter
    (fun code ->
      check ("invalid net accepted: " ^ code) (Result.is_error (N.parse code)))
    [
      {|gamma a;|};
      {|\gamma a;|};
      {|\epsilon a; \epsilon a; a.0 ~ @x;|};
      {|\epsilon a; a.1 ~ @x;|};
      {|\epsilon a; a.0 ~ a.0;|};
      {|@x ~ @y; @x ~ @z;|};
      {|a.0 ~ @x;|};
      {|\loops -1;|};
      {|\loops 1; \loops 2;|};
      {|@x ~ @y|};
      {|\unknown a;|};
    ];
  let state = start (pair "gamma" "gamma") in
  let json = M.Snapshot.to_yojson state in
  let net = N.to_yojson state.net in
  List.iter
    (fun bad ->
      check "invalid snapshot accepted"
        (Result.is_error (M.Snapshot.of_yojson bad)))
    [
      `Null;
      replace "steps" (`Int (-1)) json;
      replace "net" (replace "wires" (`List []) net) json;
      replace "net" (replace "loops" (`Int (-1)) net) json;
      replace "net" (replace "agents" (`List []) net) json;
    ];
  check "invalid pair accepted"
    (Result.is_error (M.step state (Some (id "a", id "missing"))));
  check "overflow accepted"
    (Result.is_error (M.step { state with steps = max_int } None));
  let module S = Utils_runtime.Session.Make (M) in
  let session = S.create () in
  get (S.make session (pair "gamma" "gamma") "");
  let saved = get (S.snapshot session) in
  check "bad selection accepted by session"
    (Result.is_error (S.step session "a missing"));
  check "bad selection changed session" (get (S.snapshot session) = saved);
  check "bad input accepted" (Result.is_error (M.Rinput.parse "a b c"));
  check "nonempty AInput accepted" (Result.is_error (M.Ainput.parse "x"));
  let graph_json = Utils_runtime.Render.to_yojson (M.render state) in
  match graph_json with
  | `List blocks ->
      check "missing graph block"
        (List.exists
           (function
             | `Assoc fields ->
                 List.assoc_opt "kind" fields = Some (`String "graph")
             | _ -> false)
           blocks)
  | _ -> failwith "Expected render blocks"

let () =
  rules ();
  path_semantics ();
  loops_and_context ();
  duplicate_tree ();
  validation ();
  print_endline
    "Passed Interaction Combinators: six rules, path semantics, cyclic wiring, \
     scheduling, codecs and snapshots."
