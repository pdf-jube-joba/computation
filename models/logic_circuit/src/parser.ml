open Circuit
module L = Utils.Lexer

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

let get = List.assoc_opt

let insert library (name, circuit) =
  if List.mem_assoc name library then library else library @ [ (name, circuit) ]

let result_exn = function Ok x -> x | Error message -> raise (L.Error message)
let identifier = L.identifier

let block p item =
  L.expect_symbol p '{';
  L.many_until ~skip_separator:',' p ~closing:'}' item

let named_pin p =
  let n = identifier p in
  L.expect_symbol p '.';
  let pin = identifier p in
  (n, pin)

let definitions p maps =
  let maps = ref maps in
  let lookup n =
    match get n !maps with
    | Some c -> c
    | None -> L.error p ("not found name " ^ n)
  in
  while not (L.eof p) do
    let circuit_name, circuit =
      if L.eat_keyword p "graph" then (
        L.expect_symbol p ':';
        let n = identifier p in
        L.expect_symbol p '{';
        L.expect_keyword p "in";
        let inpins = block p identifier in
        L.expect_keyword p "out";
        let otpin_maps =
          block p (fun p ->
              let external_pin = identifier p in
              L.expect_symbol p '=';
              let target = named_pin p in
              (external_pin, target))
        in
        let rec vertices verts edges inpin_maps =
          if L.eat_symbol p '}' then
            (List.rev verts, List.rev edges, List.rev inpin_maps)
          else
            let vn = identifier p in
            L.expect_symbol p ',';
            let used = identifier p in
            let c = lookup used in
            let conns =
              block p (fun p ->
                  let pin = identifier p in
                  L.expect_symbol p '=';
                  let first = identifier p in
                  if L.eat_symbol p '.' then
                    let second = identifier p in
                    (pin, `Internal (first, second))
                  else (pin, `External first))
            in
            let edges, inpin_maps =
              List.fold_left
                (fun (edges, maps) (pin, source) ->
                  match source with
                  | `Internal source -> ((source, (vn, pin)) :: edges, maps)
                  | `External external_pin ->
                      if not (List.mem external_pin inpins) then
                        L.error p ("not found inpin " ^ external_pin);
                      (edges, (external_pin, (vn, pin)) :: maps))
                (edges, inpin_maps) conns
            in
            vertices ((vn, c) :: verts) edges inpin_maps
        in
        let verts, edges, inpin_maps = vertices [] [] [] in
        (n, result_exn (new_mix n verts edges inpin_maps otpin_maps)))
      else if L.eat_keyword p "iter" then (
        L.expect_symbol p ':';
        let n = identifier p in
        L.expect_symbol p '{';
        let init_name = identifier p in
        L.expect_symbol p ',';
        let conn p =
          let a = identifier p in
          L.expect_symbol p '=';
          let b = identifier p in
          (a, b)
        in
        L.expect_keyword p "next";
        let next = block p conn in
        L.expect_keyword p "prev";
        let prev = block p conn in
        L.expect_symbol p '}';
        (n, result_exn (new_iter n (lookup init_name) next prev)))
      else L.error p "expected graph or iter"
    in
    maps := insert !maps (circuit_name, circuit)
  done;
  !maps

let parse code maps =
  L.parse ~nested_comments:false (fun p -> definitions p maps) code

let parse_main_with_maps code maps =
  Result.bind (parse code maps) (fun maps ->
      match get "main" maps with
      | Some c -> Ok c
      | None -> Error "not found main")

let parse_main code = parse_main_with_maps code (init_maps ())

let parse_signal text =
  if Utf8_trim.trim text = "" then Ok []
  else
    L.protect (fun () ->
        String.split_on_char ',' text
        |> List.map (fun item ->
            match String.rindex_opt item '=' with
            | None -> raise (L.Error "Expected NAME.PIN=BOOL")
            | Some at ->
                let pin_text = Utf8_trim.trim (String.sub item 0 at) in
                let pin =
                  match String.rindex_opt pin_text '.' with
                  | None -> raise (L.Error "Expected NAME.PIN")
                  | Some dot ->
                      let n = String.sub pin_text 0 dot in
                      let p =
                        String.sub pin_text (dot + 1)
                          (String.length pin_text - dot - 1)
                      in
                      ( result_exn (Utils.Identifier.make n),
                        result_exn (Utils.Identifier.make p) )
                in
                let b =
                  Utf8_trim.trim
                    (String.sub item (at + 1) (String.length item - at - 1))
                in
                let b =
                  match b with
                  | "#true" -> true
                  | "#false" -> false
                  | _ -> raise (L.Error ("Invalid boolean text: " ^ b))
                in
                (pin, b)))
