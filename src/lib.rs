
mod ast;
pub mod machine;
pub mod parser;

pub fn build(text: &str) -> Result<machine::Machine, u8> {
    let grammar = text.to_string();
    let token_result = parser::tokenize(&grammar);
    let parser_result = parser::parse(&grammar, token_result.0, token_result.1);

    if parser_result.is_err() { return Err(0); }
    let grammar_object = parser_result.ok().unwrap();

    let program = grammar_object.compile();
    Ok(machine::Machine::new(program))
}








#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
