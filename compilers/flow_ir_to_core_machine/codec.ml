module F = Flow_ir.Main
module C = Core_machine.Main
module N = Utils_base.Number
module Id = Utils_names.Identifier

let get = Utils_base.Error.get
let name n = N.of_bytes (Id.to_string n)
let identifier n = Id.make_exn (N.to_bytes n)

(* Three natural cells: tag, first payload, second payload. *)
let encode_value = function
  | F.Num n -> [ N.zero; n; N.zero ]
  | F.Code_label l -> [ N.of_int 1; name l; N.zero ]
  | F.Key (F.Static l) -> [ N.of_int 2; name l; N.zero ]
  | F.Key (F.Stack i) ->
      if i < 0 then failwith "Negative stack index";
      [ N.of_int 3; N.of_int i; N.zero ]
  | F.Key (F.Heap (h, i)) ->
      if i < 0 then failwith "Negative heap index";
      [ N.of_int 4; N.of_int (F.Heap_id.to_int h); N.of_int i ]

let decode_value t a b =
  match get (N.to_int t) with
  | 0 -> F.Num a
  | 1 -> F.Code_label (identifier a)
  | 2 -> F.Key (F.Static (identifier a))
  | 3 -> F.Key (F.Stack (get (N.to_int a)))
  | 4 ->
      F.Key
        (F.Heap (get (F.Heap_id.of_int (get (N.to_int a))), get (N.to_int b)))
  | _ -> failwith "Invalid flow value tag"

let encode input =
  Utils_base.Error.protect (fun () ->
      let flat =
        N.of_int (List.length input)
        :: List.concat_map (fun (l, v) -> name l :: encode_value v) input
      in
      List.mapi (fun i v -> (N.of_int i, C.Num v)) flat)

let decode mem =
  Utils_base.Error.protect (fun () ->
      let read i =
        match List.assoc_opt (N.of_int i) mem with
        | None -> N.zero
        | Some (C.Num n) -> n
        | _ -> failwith "Expected numeric encoded output"
      in
      let count = get (N.to_int (read 0)) in
      if count < 0 || count > List.length mem then
        failwith "Invalid output record count";
      List.init count (fun i ->
          let off = 1 + (4 * i) in
          ( identifier (read off),
            decode_value (read (off + 1)) (read (off + 2)) (read (off + 3)) )))
  |> Result.map Utils_base.Assoc.sorted
