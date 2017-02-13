

/*

*/


use machine;

#[derive(Debug)]
pub struct Grammar {
    rules : Vec<Pattern>,
    main : u32
}

#[derive(Debug)]
pub enum Pattern {
    CharClass(Vec<(u8, Option<u8>)>),
    CharSequence(Vec<u8>),
    CharAny,
    Variable(i32),
    Choice(Box<Pattern>, Box<Pattern>),
    ZeroOrMore(Box<Pattern>),
    OneOrMore(Box<Pattern>),
    Sequence(Vec<Box<Pattern>>),
    Optional(Box<Pattern>),
    Lookahead(bool, Box<Pattern>)
}

impl Grammar {
    pub fn compile(&mut self) -> Vec<machine::Instruction> {
        let mut rules = Vec::new();

        for p in &self.rules {
            rules.push(Grammar::compile_pattern(p));
        }

        let mut result = vec![
            machine::Instruction::Call(self.main as isize),
            machine::Instruction::Stop
        ];

        for mut rule in rules {
            result.append(&mut rule);
            result.push(machine::Instruction::Return);
        }

        result
    }

    pub fn compile_pattern(p : &Pattern) -> Vec<machine::Instruction> {
        match p {
            &Pattern::CharClass(ref data) => Grammar::compile_char_class(data),
            &Pattern::CharSequence(ref data) => Grammar::compile_char_sequence(data),
            &Pattern::CharAny => Grammar::compile_char_any(),
            &Pattern::Variable(id) => Grammar::compile_variable(id),
            &Pattern::Choice(ref le, ref ri) => Grammar::compile_choice(le, ri),
            &Pattern::ZeroOrMore(ref data) => Grammar::compile_zero_or_more(data),
            &Pattern::OneOrMore(ref data) => Grammar::compile_one_or_more(data),
            &Pattern::Sequence(ref data) => Grammar::compile_sequence(data),
            &Pattern::Optional(ref data) => Grammar::compile_optional(data),
            &Pattern::Lookahead(flag, ref data) => Grammar::compile_lookahead(flag, data)
        }
    }

    fn compile_char_class(data : &Vec<(u8, Option<u8>)>) -> Vec<machine::Instruction> {
        vec![]
    }

    fn compile_char_sequence(data : &Vec<u8>) -> Vec<machine::Instruction> {
        let mut result = vec![];
        for c in data {
            result.push(machine::Instruction::Char(*c));
        }
        result
    }

    fn compile_char_any() -> Vec<machine::Instruction> {
        vec![machine::Instruction::Any]
    }

    fn compile_variable(id : i32) -> Vec<machine::Instruction> {
        vec![machine::Instruction::Call(id as isize)]
    }

    fn compile_choice(left : &Box<Pattern>, right : &Box<Pattern>) -> Vec<machine::Instruction> {
        let mut inner_left = Grammar::compile_pattern(left);
        let mut inner_right = Grammar::compile_pattern(right);
        let mut result = vec![];
        result.push(machine::Instruction::Choice(inner_left.len() as isize + 2));
        result.append(&mut inner_left);
        result.push(machine::Instruction::Commit(inner_right.len() as isize + 1));
        result.append(&mut inner_right);
        result
    }

    fn compile_zero_or_more(data : &Box<Pattern>) -> Vec<machine::Instruction> {
        let mut inner = Grammar::compile_pattern(data);
        let instr_count = inner.len() as isize;
        let mut result = vec![];
        result.push(machine::Instruction::Choice(instr_count + 2));
        result.append(&mut inner);
        result.push(machine::Instruction::Commit(-instr_count - 1));
        result
    }

    fn compile_one_or_more(data : &Box<Pattern>) -> Vec<machine::Instruction> {
        let mut inner = Grammar::compile_pattern(data);
        let instr_count = inner.len() as isize;
        let mut result = vec![];
        result.append(&mut inner);
        result.push(machine::Instruction::Choice(instr_count + 2));
        result.append(&mut inner);
        result.push(machine::Instruction::Commit(-instr_count - 1));
        result
    }

    fn compile_sequence(data : &Vec<Box<Pattern>>) -> Vec<machine::Instruction> {
        let mut result = vec![];

        for p in data {
            let mut inner = Grammar::compile_pattern(p);
            result.append(&mut inner);
        }
        result
    }

    fn compile_optional(data : &Box<Pattern>) -> Vec<machine::Instruction> {
        vec![]
    }

    fn compile_lookahead(success : bool, data : &Box<Pattern>) -> Vec<machine::Instruction> {
        vec![]
    }
}

