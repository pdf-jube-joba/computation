let scalar_length text i =
  let byte j = Char.code text.[j] in
  let a = byte i in
  if a < 128 then 1
  else
    let count, lo, hi =
      if a >= 194 && a <= 223 then (1, 128, 191)
      else if a = 224 then (2, 160, 191)
      else if (a >= 225 && a <= 236) || (a >= 238 && a <= 239) then (2, 128, 191)
      else if a = 237 then (2, 128, 159)
      else if a = 240 then (3, 144, 191)
      else if a >= 241 && a <= 243 then (3, 128, 191)
      else if a = 244 then (3, 128, 143)
      else failwith "Invalid UTF-8 character"
    in
    if i + count >= String.length text then failwith "Truncated UTF-8 character";
    if byte (i + 1) < lo || byte (i + 1) > hi then
      failwith "Invalid UTF-8 character";
    for j = i + 2 to i + count do
      if byte j < 128 || byte j > 191 then failwith "Invalid UTF-8 character"
    done;
    count + 1

let scalar text i =
  let n = scalar_length text i in
  (String.sub text i n, i + n)

let valid text =
  let rec check i =
    if i = String.length text then true else check (i + scalar_length text i)
  in
  try check 0 with Failure _ -> false

let is_scalar text =
  text <> "" && valid text && scalar_length text 0 = String.length text
