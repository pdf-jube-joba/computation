pub fn parse(code: &str, maps: &mut crate::manipulation::List) -> anyhow::Result<()> {
    crate::parser::parse(code, maps)
}

pub fn parse_main_with_maps(
    code: &str,
    maps: crate::manipulation::List,
) -> anyhow::Result<crate::machine::LogicCircuit> {
    crate::parser::parse_main_with_maps(code, maps)
}

pub fn parse_main(code: &str) -> anyhow::Result<crate::machine::LogicCircuit> {
    crate::parser::parse_main(code)
}
