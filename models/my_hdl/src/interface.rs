pub fn parse_value(text: &str) -> anyhow::Result<crate::machine::Value> {
    crate::parser::parse_value(text)
}
