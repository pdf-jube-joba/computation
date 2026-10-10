(* Arbitrary-precision natural numbers in canonical decimal notation. Pure OCaml
   keeps the same arithmetic available to native, JS and Wasm builds. *)
type t = string [@@deriving yojson]

let zero = "0"
let one = "1"

let normalize text =
  let i = ref 0 in
  while !i < String.length text - 1 && text.[!i] = '0' do
    incr i
  done;
  String.sub text !i (String.length text - !i)

let of_string text =
  let text = String.trim text in
  if
    text <> ""
    && String.for_all (function '0' .. '9' -> true | _ -> false) text
  then Ok (normalize text)
  else Error "Expected a natural number"

let of_int n =
  if n < 0 then invalid_arg "Negative natural number" else string_of_int n

let to_string n = n

let to_int n =
  match int_of_string_opt n with
  | Some n when n >= 0 -> Ok n
  | _ -> Error "Natural number does not fit in an OCaml integer"

let compare a b =
  let c = Int.compare (String.length a) (String.length b) in
  if c = 0 then String.compare a b else c

let equal a b = compare a b = 0
let is_zero n = n = zero

let digit n i =
  if i < String.length n then Char.code n.[String.length n - 1 - i] - 48 else 0

let add a b =
  let length = max (String.length a) (String.length b) in
  let out = Bytes.make (length + 1) '0' and carry = ref 0 in
  for i = 0 to length - 1 do
    let n = digit a i + digit b i + !carry in
    Bytes.set out (length - i) (Char.chr (48 + (n mod 10)));
    carry := n / 10
  done;
  Bytes.set out 0 (Char.chr (48 + !carry));
  normalize (Bytes.to_string out)

let sub a b =
  if compare a b <= 0 then zero
  else
    let length = String.length a in
    let out = Bytes.make length '0' and borrow = ref 0 in
    for i = 0 to length - 1 do
      let n = digit a i - digit b i - !borrow in
      borrow := if n < 0 then 1 else 0;
      Bytes.set out
        (length - 1 - i)
        (Char.chr (48 + if n < 0 then n + 10 else n))
    done;
    normalize (Bytes.to_string out)

let succ a = add a one
let pred a = sub a one

let mul a b =
  let out = Array.make (String.length a + String.length b + 1) 0 in
  for i = 0 to String.length a - 1 do
    for j = 0 to String.length b - 1 do
      let k = i + j in
      let n = out.(k) + (digit a i * digit b j) in
      out.(k) <- n mod 10;
      out.(k + 1) <- out.(k + 1) + (n / 10)
    done
  done;
  for k = 0 to Array.length out - 2 do
    out.(k + 1) <- out.(k + 1) + (out.(k) / 10);
    out.(k) <- out.(k) mod 10
  done;
  normalize
    (String.init (Array.length out) (fun i ->
         Char.chr (48 + out.(Array.length out - 1 - i))))

let divmod a b =
  if is_zero b then Error "Division by zero"
  else
    let remainder = ref zero in
    let quotient =
      String.map
        (fun c ->
          remainder := normalize (!remainder ^ String.make 1 c);
          let q = ref 0 in
          while compare !remainder b >= 0 do
            remainder := sub !remainder b;
            incr q
          done;
          Char.chr (48 + !q))
        a
    in
    Ok (normalize quotient, !remainder)

let of_yojson = function
  | `String text -> of_string text
  | `Int n when n >= 0 -> Ok (of_int n)
  | _ -> Error "Expected a natural-number decimal string"
