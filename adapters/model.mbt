///|
pub fn make(code : String, ainput : String) -> Result[Unit, String] {
  try Ok(machine.make(code, ainput)) catch { @utils.Fault(message) => Err(message) }
}

///|
pub fn step(rinput : String) -> Result[String, String] {
  try Ok(machine.step(rinput)) catch { @utils.Fault(message) => Err(message) }
}

///|
pub fn snapshot() -> Result[String, String] {
  try Ok(machine.snapshot()) catch { @utils.Fault(message) => Err(message) }
}

///|
pub fn restore(snapshot : String) -> Result[Unit, String] {
  try Ok(machine.restore(snapshot)) catch { @utils.Fault(message) => Err(message) }
}

///|
pub fn render(snapshot : String) -> Result[String, String] {
  try Ok(machine.render(snapshot)) catch { @utils.Fault(message) => Err(message) }
}
