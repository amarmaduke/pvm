use machine;

#[derive(Debug)]
pub struct Grammar {
    pub rules : Vec<Pattern>,
    pub main : u32
}

#[derive(Debug)]
pub enum Pattern {
    CharClass(Vec<(u8, Option<u8>)>),
    CharSequence(Vec<u8>),
    CharAny,
    Variable(i32, i32, usize, bool),
    Choice(Box<Pattern>, Box<Pattern>),
    ZeroOrMore(Box<Pattern>),
    OneOrMore(Box<Pattern>),
    Sequence(Vec<Box<Pattern>>),
    Optional(Box<Pattern>),
    Lookahead(bool, Box<Pattern>)
}

impl Grammar {
    pub fn compile(&self) -> Vec<machine::Instruction> {
        let mut rules = Vec::new();
        let mut lookup = vec![];

        for p in &self.rules {
            rules.push(Grammar::compile_pattern(p));
        }

        let mut result = vec![
            machine::Instruction::Call(self.main as isize),
            machine::Instruction::Stop
        ];

        let mut k = 2isize;
        let mut id = 0;
        for mut rule in rules {
            lookup.push(k);
            k += 3 + rule.len() as isize;
            result.push(machine::Instruction::PushPos(id));
            result.append(&mut rule);
            result.push(machine::Instruction::SavePos);
            result.push(machine::Instruction::Return);
            id += 1;
        }

        for i in 0..result.len() {
            if let machine::Instruction::Call(r) = result[i] {
                let dist = lookup[r as usize] - i as isize;
                result[i] = machine::Instruction::Call(dist as isize);
            }
            if let machine::Instruction::PrecedenceCall(r, precedence) = result[i] {
                let dist = lookup[r as usize] - i as isize;
                result[i] = machine::Instruction::PrecedenceCall(dist as isize, precedence);
            }
        }
        result
    }

    pub fn compile_pattern(p : &Pattern) -> Vec<machine::Instruction> {
        match p {
            &Pattern::CharClass(ref data) => Grammar::compile_char_class(data),
            &Pattern::CharSequence(ref data) => Grammar::compile_char_sequence(data),
            &Pattern::CharAny => Grammar::compile_char_any(),
            &Pattern::Variable(id, precedence, _, _) => Grammar::compile_variable(id, precedence),
            &Pattern::Choice(ref le, ref ri) => Grammar::compile_choice(le, ri),
            &Pattern::ZeroOrMore(ref data) => Grammar::compile_zero_or_more(data),
            &Pattern::OneOrMore(ref data) => Grammar::compile_one_or_more(data),
            &Pattern::Sequence(ref data) => Grammar::compile_sequence(data),
            &Pattern::Optional(ref data) => Grammar::compile_optional(data),
            &Pattern::Lookahead(flag, ref data) => Grammar::compile_lookahead(flag, data)
        }
    }

    fn compile_char_class(data : &Vec<(u8, Option<u8>)>) -> Vec<machine::Instruction> {
        let mut result = vec![];
        let mut jump = data.len();
        for range in data.iter().take(data.len() - 1) {
            let left = (*range).0;
            let right = (*range).1.unwrap_or(left);
            result.push(machine::Instruction::CharRangeLink(left, right, jump as isize));
            jump -= 1;
        }
        
        if let Some(last) = data.iter().last() {
            let left = (*last).0;
            let right = (*last).1.unwrap_or(left);
            result.push(machine::Instruction::CharRange(left, right));
        }
        result
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

    fn compile_variable(id : i32, precedence : i32) -> Vec<machine::Instruction> {
        if precedence == -1 {
            vec![machine::Instruction::Call(id as isize)]
        } else {
            vec![machine::Instruction::PrecedenceCall(id as isize, precedence as isize)]
        }
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
        result.push(machine::Instruction::PartialCommit(-instr_count));
        result
    }

    fn compile_one_or_more(data : &Box<Pattern>) -> Vec<machine::Instruction> {
        let mut inner = Grammar::compile_pattern(data);
        let mut inner_clone = inner.clone();
        let instr_count = inner.len() as isize;
        let mut result = vec![];
        result.append(&mut inner_clone);
        result.push(machine::Instruction::Choice(instr_count + 2));
        result.append(&mut inner);
        result.push(machine::Instruction::PartialCommit(-instr_count));
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
        let mut result = vec![];
        let mut inner = Grammar::compile_pattern(data);

        result.push(machine::Instruction::Choice(inner.len() as isize + 2));
        result.append(&mut inner);
        result.push(machine::Instruction::Commit(1));
        result
    }

    fn compile_lookahead(success : bool, data : &Box<Pattern>) -> Vec<machine::Instruction> {
        let mut result = vec![];
        let mut inner = Grammar::compile_pattern(data);
        let instr_count = inner.len() as isize;
        if success {
            result.push(machine::Instruction::Choice(instr_count + 4));
            result.push(machine::Instruction::Choice(instr_count + 2));
            result.append(&mut inner);
            result.push(machine::Instruction::FailTwice);
            result.push(machine::Instruction::FailTwice);
        } else {
            result.push(machine::Instruction::Choice(instr_count + 2));
            result.append(&mut inner);
            result.push(machine::Instruction::FailTwice);
        }
        result
    }

    /* Algorithm to find left recursive calls
        - First label all calls with a unique id
        - Starting with the root, iterate through the tree structure
        - If you find input consuming calls (that are not optional) mark as consuming input
        - Choices should be handled independently (and have there own mark for consumig input)
        - Calls recursively do the same
        - If a call turns out to be recursive, check the cycle to see if any input is consumed
            - If no input is consumed in the call chain then it is left recursive
            - Otherwise it is right recursive
            - May need to handle mutually recursive rules with a special method
        - You should give each call it's own call stack (with the rule, call id, and if it matched any input)
    */
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::marker::PhantomData;

    fn execute_test(grammar : &Grammar, subjects : &Vec<&str>, expected : &Vec<bool>, rule_names : Vec<String>) {
        let program = grammar.compile();
        let mut machine = machine::Machine::<String> {
            program: program,
            rule_names: rule_names,
            skip: vec![],
            skip_on: false,
            marker: PhantomData
        };
        assert!(subjects.len() == expected.len());
        for i in 0..expected.len() {
            let result = machine.execute(subjects[i].to_string().into_bytes());
            let fail = result.is_err();
            println!("{:?}", machine.program);
            println!("{}", subjects[i]);
            assert!(!fail == expected[i]);
        }
    }

    #[test]
    fn simple_char_grammar_rules() {
        /*
            main { any;char_class;char_seq }
            char_class { ['a'..'z''A'..'A'] }
            char_seq { 'a';'b';'c' }
            any { . }
        */
        let char_class = Pattern::CharClass(vec![
            ('a' as u8, Some('z' as u8)),
            ('A' as u8, Some('A' as u8))
        ]);
        let char_seq = Pattern::CharSequence(vec![
            'a' as u8,
            'b' as u8,
            'c' as u8
        ]);
        let main = Pattern::Sequence(vec![
            Box::new(Pattern::Variable(1, 1, 0, false)),
            Box::new(Pattern::Variable(2, 1, 0, false)),
            Box::new(Pattern::Variable(3, 1, 0, false)),
        ]);

        let grammar = Grammar {
            rules: vec![
                main,
                Pattern::CharAny,
                char_class,
                char_seq
            ],
            main: 0
        };
        let subjects = vec!["azabc", "Bkabc", "AAabc", "aqd", "xyz"];
        let expected = vec![true, true, true, false, false];
        let rule_names = vec!["main".to_string(), "any".to_string(), "char_class".to_string(), "char_seq".to_string()];
        execute_test(&grammar, &subjects, &expected, rule_names);
    }

    #[test]
    fn simple_lookahead_grammar() { // main { !a;. / &a;. }
        let ambersand = Pattern::Lookahead(true,
            Box::new(Pattern::CharSequence(vec!['a' as u8])));
        let not = Pattern::Lookahead(false,
            Box::new(Pattern::CharSequence(vec!['a' as u8])));
        let main = Pattern::Choice(
            Box::new(Pattern::Sequence(vec![
                Box::new(not),
                Box::new(Pattern::CharAny)
            ])),
            Box::new(Pattern::Sequence(vec![
                Box::new(ambersand),
                Box::new(Pattern::CharAny)
            ])),
        );

        let grammar = Grammar {
            rules: vec![
                main
            ],
            main: 0
        };
        let subjects = vec!["b", "a", "z", "aa", ""];
        let expected = vec![true, true, true, false, false];
        let rule_names = vec!["main".to_string()];
        execute_test(&grammar, &subjects, &expected, rule_names);
    }

    #[test]
    fn simple_iteration_grammar() { // main { a+ / b* }
        let main = Pattern::Choice(
            Box::new(Pattern::OneOrMore(
                Box::new(Pattern::CharSequence(vec!['a' as u8]))
            )),
            Box::new(Pattern::ZeroOrMore(
                Box::new(Pattern::CharSequence(vec!['b' as u8]))
            ))
        );

        let grammar = Grammar {
            rules: vec![
                main
            ],
            main: 0
        };
        let subjects = vec!["a", "aaaa", "", "b", "bbbbb", "c"];
        let expected = vec![true, true, true, true, true, false];
        let rule_names = vec!["main".to_string()];
        execute_test(&grammar, &subjects, &expected, rule_names);
    }

    #[test]
    fn simple_optional_subparser() {
        /*
            main { a? 'b' }
            a { a+ }
        */
        let a = Pattern::OneOrMore(
            Box::new(Pattern::CharSequence(vec!['a' as u8])));
        let main = Pattern::Sequence(vec![
            Box::new(Pattern::Optional(Box::new(Pattern::Variable(1, 1, 0, false)))),
            Box::new(Pattern::CharSequence(vec!['b' as u8]))
        ]);

        let grammar = Grammar {
            rules: vec![
                main,
                a
            ],
            main: 0
        };
        let subjects = vec!["b", "ab", "aaaaab", "", "bb"];
        let expected = vec![true, true, true, false, false];
        let rule_names = vec!["main".to_string(), "a".to_string()];
        execute_test(&grammar, &subjects, &expected, rule_names);
    }
}