include Utils.Lexer

(* Rust's lex_block_comment stops at the first terminator. Mask comments before
   using the shared (nested-comment) lexer, keeping byte offsets unchanged. *)
let without_comments text =
  let n = String.length text in
  let masked = Bytes.of_string text in
  let has i a b = i + 1 < n && text.[i] = a && text.[i + 1] = b in
  let blank first last =
    for i = first to last - 1 do
      if text.[i] <> '\n' && text.[i] <> '\r' then Bytes.set masked i ' '
    done
  in
  let rec scan i =
    if i < n then
      if has i '/' '/' then (
        let j = ref (i + 2) in
        while !j < n && text.[!j] <> '\n' do
          incr j
        done;
        blank i !j;
        scan !j)
      else if has i '/' '*' then (
        let j = ref (i + 2) in
        while !j + 1 < n && not (has !j '*' '/') do
          incr j
        done;
        if !j + 1 >= n then
          raise (Error (Printf.sprintf "Unclosed comment at byte %d" i));
        blank i (!j + 2);
        scan (!j + 2))
      else if text.[i] = '"' then (
        let j = ref (i + 1) in
        while !j < n && text.[!j] <> '"' do
          if text.[!j] = '\\' && !j + 1 < n then j := !j + 2 else incr j
        done;
        scan (min n (!j + 1)))
      else scan (i + 1)
  in
  scan 0;
  Bytes.to_string masked

let create text = Utils.Lexer.create (without_comments text)

let parse f text =
  protect (fun () ->
      let p = create text in
      let value = f p in
      expect_eof p;
      value)
