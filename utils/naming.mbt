///|
pub struct Supply {
  used : Map[String, Bool]
}

///|
pub fn supply(reserved~ : Array[String] = []) -> Supply {
  let used : Map[String, Bool] = Map([])
  for name in reserved {
    used[name] = true
  }
  { used, }
}

///|
pub fn Supply::reserve(self : Supply, name : String) -> Unit {
  self.used[name] = true
}

///|
pub fn Supply::fresh(self : Supply, hint : String) -> String {
  let mut name = hint
  let mut i = 0
  while self.used.contains(name) {
    name = hint + "_" + i.to_string()
    i += 1
  }
  self.used[name] = true
  name
}
