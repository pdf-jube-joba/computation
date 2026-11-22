///|
pub fn of_bytes(text : String) -> Nat {
  let bytes = @utf8.encode(text)
  let mut n = zero()
  for i = bytes.length() - 1; i >= 0; i = i - 1 {
    n = n * nint(256) + nint(bytes[i].to_int().reinterpret_as_uint())
  }
  n
}

///|
pub fn to_bytes(n : Nat) -> String raise Fault {
  let mut n = n
  let bytes = []
  while !n.is_zero() {
    bytes.push((n % nint(256)).to_int().to_byte())
    n = n / nint(256)
  }
  @utf8.decode(Bytes::from_array(bytes), ignore_bom=false) catch {
    _ => error("print expects valid UTF-8")
  }
}
