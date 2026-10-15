module Number = Utils_base.Number
module Tape = Utils_codec.Tape_text.Identifier
module Tape_text = Utils_codec.Tape_text.Identifier
module Memory = Utils_data.Memory

let get = Utils_base.Error.get
let check message condition = if not condition then failwith message
let sign name = Some (Utils_names.Identifier.make_exn name)

let tape () =
  let original = get (Tape_text.parse "a,b|c|d,e") in
  let left = Tape.move_to original Tape.Left in
  check "tape moves toward nearest left cell" (left.head = sign "b");
  check "tape move preserves cell order"
    (Tape.equal original (Tape.move_to left Tape.Right));
  let outside =
    original
    |> Fun.flip Tape.move_to Tape.Left
    |> Fun.flip Tape.move_to Tape.Left
    |> Fun.flip Tape.move_to Tape.Left
  in
  check "tape extends with blank cells" (outside.head = None);
  let written = Tape.head_write outside (sign "x") in
  check "writing preserves the original tape" (outside.head = None);
  check "tape text roundtrip"
    (Tape.equal written (get (Tape_text.parse (Tape_text.print written))));
  check "tape JSON roundtrip"
    (written = get (Tape.of_yojson (Tape.to_yojson written)));
  let at_end = get (Tape.from_vec [ sign "a"; sign "b" ] 2) in
  check "head may follow the supplied cells" (at_end.head = None);
  check "tape vector keeps its head position"
    (Tape.into_vec at_end = ([ sign "a"; sign "b"; None ], 2));
  check "invalid tape position rejected" (Result.is_error (Tape.from_vec [] 1));
  check "blank padding does not change tape equality"
    (Tape.equal Tape.blank
       { Tape.blank with left = [ None ]; right = [ None ] })

let memory () =
  let original = [| Number.of_int 7 |] in
  let extended =
    Memory.write ~default:Number.zero original 3 (Number.of_int 9)
  in
  check "writing does not change saved memory" (original = [| Number.of_int 7 |]);
  check "memory preserves existing cells"
    (Number.equal
       (Memory.read ~default:Number.zero extended 0)
       (Number.of_int 7));
  check "memory fills gaps with zero"
    (Number.is_zero (Memory.read ~default:Number.zero extended 2));
  check "memory stores written cells"
    (Number.equal
       (Memory.read ~default:Number.zero extended 3)
       (Number.of_int 9));
  check "memory reads beyond its storage as zero"
    (Number.is_zero (Memory.read ~default:Number.zero extended 10));
  check "negative memory address rejected"
    (Result.is_error
       (Utils_base.Error.protect (fun () ->
            Memory.read ~default:Number.zero original (-1))))

let polymorphic_data () =
  let module Bit = struct
    include Utils_codec.Text.Bool

    let default = false
    let equal = Bool.equal
    let to_yojson value = `Bool value

    let of_yojson = function
      | `Bool value -> Ok value
      | _ -> Error "Expected bit"
  end in
  let module Bits = Utils_codec.Tape_text.Make (Bit) in
  let tape = get (Bits.parse "#true|#false|#true") in
  let left = Bits.move_to tape Bits.Left in
  check "generic tape stores boolean cells" left.head;
  check "generic tape uses its supplied default"
    (not (Bits.move_to left Bits.Left).head);
  check "generic tape preserves cells when moving"
    (Bits.equal tape (Bits.move_to left Bits.Right));
  check "generic tape JSON uses the element codec"
    (Bits.equal tape (get (Bits.of_yojson (Bits.to_yojson tape))));
  check "generic tape text roundtrip"
    (Bits.equal tape (get (Bits.parse (Bits.print tape))));
  let strings = [| "first" |] in
  let extended = Memory.write ~default:"empty" strings 2 "last" in
  check "generic memory preserves its source" (strings = [| "first" |]);
  check "generic memory fills gaps with its supplied default"
    (Memory.read ~default:"empty" extended 1 = "empty");
  check "generic memory stores string cells"
    (Memory.read ~default:"empty" extended 2 = "last");
  check "generic memory reads missing cells as its supplied default"
    (Memory.read ~default:"empty" extended 5 = "empty");
  let module Image = Utils_codec.Memory_image_text.Make (Bit) in
  let image = get (Image.parse ~register_count:2 "#false, #true\n#true") in
  check "generic image uses the element codec"
    (get (Image.parse ~register_count:2 (Image.print image)) = image)

let codecs () =
  let module Image = Utils_codec.Memory_image_text.Natural in
  let image = get (Image.parse ~register_count:2 "+0, 03\n+4\n5") in
  check "register layout is supplied by the model"
    (Image.print image = "0, 3\n4\n5");
  check "register count is validated"
    (Result.is_error (Image.parse ~register_count:8 "0, 3"));
  check "memory image roundtrip"
    (get (Image.parse ~register_count:2 (Image.print image)) = image);
  let module Chars = Utils_codec.Character_list_text in
  let chars = get (Chars.parse "((['あ', '\\n', '\'']))") in
  check "character list roundtrip"
    (get (Chars.parse (Chars.print chars)) = chars);
  check "character input rejects expressions"
    (Result.is_error (Chars.parse "head ['a']"));
  let module Tree = Utils_codec.Character_tree_text in
  let tree = get (Tree.parse "(cons (atom (('あ'))) (atom '\\n'))") in
  check "character tree roundtrip" (get (Tree.parse (Tree.print tree)) = tree);
  check "tree input rejects projections"
    (Result.is_error (Tree.parse "left (cons atom 'a' atom 'b')"));
  let module Env = Utils_codec.Environment_text.Natural in
  let env = get (Env.parse "x = 2\ny = 3\nx = 4") in
  check "environment keeps the last assignment"
    (Number.equal
       (Utils_data.Environment.Natural.get env
          (Utils_names.Identifier.make_exn "x"))
       (Number.of_int 4));
  check "environment roundtrip" (get (Env.parse (Env.print env)) = env)

let () =
  tape ();
  memory ();
  polymorphic_data ();
  codecs ();
  print_endline "Passed shared data and codec checks."
