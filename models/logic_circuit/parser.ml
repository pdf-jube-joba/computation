open Circuit
module L = Utils.Parser

type library = (Utils.Identifier.t * t) list

let by_const name =
  match Utils.Identifier.make name with
  | Ok name -> name
  | Error message -> invalid_arg message

let init_maps () =
  let gates kinds =
    List.concat_map
      (fun k ->
        List.map
          (fun b -> ((gate_name k ^ if b then "_T" else "_F"), new_gate k b))
          [ true; false ])
      kinds
  in
  gates [ Not; And; Or; Cst; Br ]
  @ [ ("END", new_gate End false) ]
  @ gates [ Delay ]

let insert library (name, circuit) =
  if List.mem_assoc name library then library else library @ [ (name, circuit) ]

let result_exn = function Ok x -> x | Error message -> raise (L.Error message)

let block p item =
  L.group p ~opening:'{' (fun p -> L.many ~skip_separator:',' p item)

let named_pin p =
  let n = L.identifier p in
  L.expect_symbol p '.';
  let pin = L.identifier p in
  (n, pin)

let definitions p maps =
  let maps = ref maps in
  let lookup p n =
    match List.assoc_opt n !maps with
    | Some c -> c
    | None -> L.error p ("not found name " ^ n)
  in
  while not (L.eof p) do
    let circuit_name, circuit =
      if L.bump_if_keyword p "graph" then (
        L.expect_symbol p ':';
        let n = L.identifier p in
        let circuit =
          L.group p ~opening:'{' (fun p ->
              L.expect_keyword p "in";
              let inpins = block p L.identifier in
              L.expect_keyword p "out";
              let otpin_maps =
                block p (fun p ->
                    let external_pin = L.identifier p in
                    L.expect_symbol p '=';
                    let target = named_pin p in
                    (external_pin, target))
              in
              let rec vertices verts edges inpin_maps =
                if L.eof p then
                  (List.rev verts, List.rev edges, List.rev inpin_maps)
                else
                  let vn = L.identifier p in
                  L.expect_symbol p ',';
                  let used = L.identifier p in
                  let c = lookup p used in
                  let conns =
                    block p (fun p ->
                        let pin = L.identifier p in
                        L.expect_symbol p '=';
                        let first = L.identifier p in
                        if L.bump_if_symbol p '.' then
                          let second = L.identifier p in
                          (pin, `Internal (first, second))
                        else (pin, `External first))
                  in
                  let edges, inpin_maps =
                    List.fold_left
                      (fun (edges, maps) (pin, source) ->
                        match source with
                        | `Internal source ->
                            ((source, (vn, pin)) :: edges, maps)
                        | `External external_pin ->
                            if not (List.mem external_pin inpins) then
                              L.error p ("not found inpin " ^ external_pin);
                            (edges, (external_pin, (vn, pin)) :: maps))
                      (edges, inpin_maps) conns
                  in
                  vertices ((vn, c) :: verts) edges inpin_maps
              in
              let verts, edges, inpin_maps = vertices [] [] [] in
              result_exn (new_mix n verts edges inpin_maps otpin_maps))
        in
        (n, circuit))
      else if L.bump_if_keyword p "iter" then (
        L.expect_symbol p ':';
        let n = L.identifier p in
        let circuit =
          L.group p ~opening:'{' (fun p ->
              let init_name = L.identifier p in
              L.expect_symbol p ',';
              let conn p =
                let a = L.identifier p in
                L.expect_symbol p '=';
                let b = L.identifier p in
                (a, b)
              in
              L.expect_keyword p "next";
              let next = block p conn in
              L.expect_keyword p "prev";
              let prev = block p conn in
              result_exn (new_iter n (lookup p init_name) next prev))
        in
        (n, circuit))
      else L.error p "expected graph or iter"
    in
    maps := insert !maps (circuit_name, circuit)
  done;
  !maps

let parse code maps =
  L.parse ~nested_comments:false (fun p -> definitions p maps) code

let parse_main_with_maps code maps =
  Result.bind (parse code maps) (fun maps ->
      match List.assoc_opt "main" maps with
      | Some c -> Ok c
      | None -> Error "not found main")

module Code = struct
  type nonrec t = t

  let parse text =
    match String.trim text with
    | text when String.starts_with ~prefix:"[" text ->
        Result.bind (Utils.Json.decode text) (fun json ->
            Result.bind (of_yojson json) (fun circuit ->
                Result.map (fun () -> circuit) (validate_identifiers circuit)))
    | _ -> parse_main_with_maps text (init_maps ())

  let print circuit = Yojson.Safe.pretty_to_string (to_yojson circuit)
end
