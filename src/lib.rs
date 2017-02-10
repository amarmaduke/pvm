
mod ast;
mod machine;
mod syntax;

pub fn parse(text: &str) {
    let program = vec![machine::Instruction::Char(0)];
    let mut machine = machine::Machine::new(program);
    machine.execute(text.to_string().into_bytes());
}












#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
