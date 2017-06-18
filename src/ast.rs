use std::collections::HashSet;
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
    pub fn compile(&mut self) -> Vec<machine::Instruction> {
        let mut rules = Vec::new();
        let mut lookup = vec![];

        self.name_variables();
        println!("{:?}", self);
        let left_recursive_calls = self.discover_left_recursion();
        println!("{:?}", left_recursive_calls);
        self.label_variables(&left_recursive_calls);

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
            if let machine::Instruction::PrecedenceCall(r, precedence, is_left) = result[i] {
                let dist = lookup[r as usize] - i as isize;
                result[i] = machine::Instruction::PrecedenceCall(dist as isize, precedence, is_left);
            }
        }
        result
    }

    pub fn compile_pattern(p : &Pattern) -> Vec<machine::Instruction> {
        match p {
            &Pattern::CharClass(ref data) => Grammar::compile_char_class(data),
            &Pattern::CharSequence(ref data) => Grammar::compile_char_sequence(data),
            &Pattern::CharAny => Grammar::compile_char_any(),
            &Pattern::Variable(id, precedence, _, is_left) => Grammar::compile_variable(id, precedence, is_left),
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

    fn compile_variable(id : i32, precedence : i32, is_left : bool) -> Vec<machine::Instruction> {
        if precedence == -1 && !is_left {
            vec![machine::Instruction::Call(id as isize)]
        } else {
            vec![machine::Instruction::PrecedenceCall(id as isize, precedence as isize, is_left)]
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

    fn name_variables(&mut self) {
        let mut id = 0;
        for pattern in &mut self.rules {
            Grammar::name_pattern(pattern, &mut id);
        }
    }

    fn name_pattern(pattern : &mut Pattern, id : &mut usize) {
        use self::Pattern::*;
        match *pattern {
            Variable(_, _, ref mut name, _) => { 
                *name = *id;
                *id += 1
            },
            Choice(ref mut le, ref mut ri) => { 
                Grammar::name_pattern(le, id); 
                Grammar::name_pattern(ri, id);
            },
            ZeroOrMore(ref mut data) => {
                Grammar::name_pattern(data, id);
            },
            OneOrMore(ref mut data) => {
                Grammar::name_pattern(data, id);
            },
            Sequence(ref mut data) => {
                for pattern in data {
                    Grammar::name_pattern(pattern, id);
                }
            },
            Optional(ref mut data) => {
                Grammar::name_pattern(data, id);
            },
            Lookahead(_, ref mut data) => {
                Grammar::name_pattern(data, id);
            },
            _ => { }
        }
    }

    fn label_variables(&mut self, left_recursive_calls : &HashSet<usize>)
    {
        for pattern in &mut self.rules {
            Grammar::label_pattern(pattern, left_recursive_calls);
        }
    }

    fn label_pattern(pattern : &mut Pattern, left_recursive_calls : &HashSet<usize>) {
        use self::Pattern::*;
        match *pattern {
            Variable(_, _, name, ref mut is_left_recursive) => {
                if left_recursive_calls.contains(&name) {
                    *is_left_recursive = true;
                }
            },
            Choice(ref mut le, ref mut ri) => { 
                Grammar::label_pattern(le, left_recursive_calls); 
                Grammar::label_pattern(ri, left_recursive_calls);
            },
            ZeroOrMore(ref mut data) => {
                Grammar::label_pattern(data, left_recursive_calls);
            },
            OneOrMore(ref mut data) => {
                Grammar::label_pattern(data, left_recursive_calls);
            },
            Sequence(ref mut data) => {
                for pattern in data {
                    Grammar::label_pattern(pattern, left_recursive_calls);
                }
            },
            Optional(ref mut data) => {
                Grammar::label_pattern(data, left_recursive_calls);
            },
            Lookahead(_, ref mut data) => {
                Grammar::label_pattern(data, left_recursive_calls);
            },
            _ => { }
        }
    }

    fn discover_left_recursion(&self) -> HashSet<usize> {
        let mut result = HashSet::new();
        let mut right_calls = HashSet::new();
        self.traverse_pattern(&self.rules[0], &mut vec![], &mut result, &mut right_calls, false);
        result
    }

    fn traverse_pattern(&self,
        pattern : &Pattern,
        stack : &mut Vec<(usize, bool)>,
        left_calls : &mut HashSet<usize>,
        right_calls : &mut HashSet<usize>,
        mut consumed : bool)
        -> bool
    {
        use self::Pattern::*;

        match pattern {
            &CharClass(_) | &CharSequence(_) | &CharAny => { consumed = true; },
            &Variable(r, _, id, _) => {
                if left_calls.contains(&id) {
                    
                } else if right_calls.contains(&id) {
                    consumed = true;
                } else if stack.iter().find(|&&x| x.0 == id).is_some() {
                    // We've found a cycle
                    let is_left_recursive = !stack.iter()
                        .skip_while(|x| x.0 != id)
                        .fold(false, |acc, &x| acc || x.1);
                    println!("{:?} is_left: {}", stack, is_left_recursive);
                    consumed = if is_left_recursive {
                        for x in stack.iter().skip_while(|x| x.0 != id) {
                            left_calls.insert(x.0);
                        }
                        false
                    } else {
                        for x in stack.iter().skip_while(|x| x.0 != id) {
                            right_calls.insert(x.0);
                        }
                        true
                    };
                    while let Some(x) = stack.pop() {
                        if x.0 == id { break; }
                    }
                } else {
                    stack.push((id, consumed));
                    let tmp = self.traverse_pattern(&self.rules[r as usize], stack, left_calls, right_calls, false);
                    consumed = consumed || tmp;
                }
            },
            &Choice(ref le, ref ri) => {
                let (tmp1, tmp2) = (
                    self.traverse_pattern(le, stack, left_calls, right_calls, consumed),
                    self.traverse_pattern(ri, stack, left_calls, right_calls, consumed));
                consumed = consumed || (tmp1 && tmp2);
            },
            &ZeroOrMore(ref p) | &Optional(ref p) => {
                self.traverse_pattern(p, stack,  left_calls, right_calls, consumed);
            },
            &OneOrMore(ref p) => {
                let tmp = self.traverse_pattern(p, stack,  left_calls, right_calls, consumed);
                consumed = consumed || tmp;
            },
            &Sequence(ref data) => {
                for p in data {
                    let tmp = self.traverse_pattern(p, stack,  left_calls, right_calls, consumed);
                    consumed = consumed || tmp;
                }
            },
            &Lookahead(_, ref p) => {
                let tmp = self.traverse_pattern(p, stack,  left_calls, right_calls, consumed);
                consumed = consumed || tmp;
            }
        }

        consumed
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

    fn execute_test(grammar : &mut Grammar, subjects : &Vec<&str>, expected : &Vec<bool>, rule_names : Vec<String>) {
        let program = grammar.compile();
        let jump_table = machine::Machine::<String>::get_jump_table(&program);
        let mut machine = machine::Machine::<String> {
            program: program,
            rule_names: rule_names,
            skip: vec![],
            skip_on: false,
            jump_table: jump_table,
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

        let mut grammar = Grammar {
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
        execute_test(&mut grammar, &subjects, &expected, rule_names);
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

        let mut grammar = Grammar {
            rules: vec![
                main
            ],
            main: 0
        };
        let subjects = vec!["b", "a", "z", "aa", ""];
        let expected = vec![true, true, true, false, false];
        let rule_names = vec!["main".to_string()];
        execute_test(&mut grammar, &subjects, &expected, rule_names);
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

        let mut grammar = Grammar {
            rules: vec![
                main
            ],
            main: 0
        };
        let subjects = vec!["a", "aaaa", "", "b", "bbbbb", "c"];
        let expected = vec![true, true, true, true, true, false];
        let rule_names = vec!["main".to_string()];
        execute_test(&mut grammar, &subjects, &expected, rule_names);
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

        let mut grammar = Grammar {
            rules: vec![
                main,
                a
            ],
            main: 0
        };
        let subjects = vec!["b", "ab", "aaaaab", "", "bb"];
        let expected = vec![true, true, true, false, false];
        let rule_names = vec!["main".to_string(), "a".to_string()];
        execute_test(&mut grammar, &subjects, &expected, rule_names);
    }
}