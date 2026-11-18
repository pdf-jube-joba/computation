///|
pub fn of_bytes(text : String) -> @base.Nat {
  let bytes = @utf8.encode(text)
  let mut n = @base.zero()
  for i = bytes.length() - 1; i >= 0; i = i - 1 {
    n = n * @base.nint(256) + @base.nint(bytes[i].to_int())
  }
  n
}

///|
pub fn to_bytes(n : @base.Nat) -> String raise @base.Fault {
  let mut n = n
  let bytes = []
  while !n.is_zero() {
    bytes.push((n % @base.nint(256)).to_int().to_byte())
    n = n / @base.nint(256)
  }
  @utf8.decode(Bytes::from_array(bytes), ignore_bom=false) catch {
    _ => @base.error("print expects valid UTF-8")
  }
}
