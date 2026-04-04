pub fn parse(str: &str) -> Result<crate::machine::RecursiveFunctions, String> {
    crate::parser::parse(str)
}
