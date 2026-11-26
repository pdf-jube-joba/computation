///|
pub fn compile_code(input : String) -> Result[String, String] {
  try Ok((machine.compile_code)(input)) catch { @utils.Fault(message) => Err(message) }
}

///|
pub fn encode_ainput(ainput : String) -> Result[String, String] {
  try Ok((machine.encode_ainput)(ainput)) catch { @utils.Fault(message) => Err(message) }
}

///|
pub fn encode_rinput(rinput : String) -> Result[String, String] {
  try Ok((machine.encode_rinput)(rinput)) catch { @utils.Fault(message) => Err(message) }
}

///|
pub fn decode_routput(output : String) -> Result[String, String] {
  try Ok((machine.decode_routput)(output)) catch { @utils.Fault(message) => Err(message) }
}

///|
pub fn decode_foutput(output : String) -> Result[String, String] {
  try Ok((machine.decode_foutput)(output)) catch { @utils.Fault(message) => Err(message) }
}
