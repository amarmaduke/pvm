use std::str::FromStr;
use std::path::Path;
use std::fs::File;
use std::io::prelude::*;
use std::hash::Hash;
use std::collections::HashSet;
use std::marker::PhantomData;
use parser;

#[derive(Debug, Copy, Clone)]
enum StackFrame {
    Return(isize),
    Backtrack(isize, usize),
    PrecedenceBacktrack(isize, isize, usize, Option<usize>, isize, bool, bool)
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Instruction {
    Char(u8),
    TestChar(u8, isize),
    Any,
    TestAny(usize, isize),
    CharRange(u8, u8),
    CharRangeLink(u8, u8, isize),
    Choice(isize),
    Jump(isize),
    Call(isize),
    PrecedenceCall(isize, isize, bool),
    Return,
    Commit(isize),
    BackCommit(isize),
    PartialCommit(isize),
    PushPos(usize),
    SavePos,
    Fail,
    FailTwice,
    Stop,
    ToggleSkip
}

pub struct Machine<T> 
    where T : Eq + Hash + FromStr
{
    pub program: Vec<Instruction>,
    pub rule_names: Vec<String>,
    pub skip : Vec<(u8, u8)>,
    pub skip_on : bool,
    pub jump_table : Vec<isize>,
    pub marker : PhantomData<T>
}

#[derive(Debug)]
pub enum Error<T> {
    MarkerError(T),
    ParserError(usize),
    MachineError(usize)
}

impl<T> Machine<T>
    where T : Eq + Hash + FromStr
{
    pub fn skip_parser(&mut self, x : u8) -> bool {
        let mut result = false;
        for t in &self.skip {
            result |= x >= t.0 && x <= t.1;
        }

        result
    }

    pub fn execute(&mut self, input : Vec<u8>) -> Result<Vec<(T, usize, usize)>, Error<T::Err>> {
        let mut stack = Vec::new();
        let mut pos_stack = Vec::new();
        let mut result = HashSet::new();
        let mut pc = 0;
        let mut i = 0;
        let mut fail = false;

        loop {
            //println!("i: {}, fail: {}, pc: {}, \n {:?} \n {:?}", i, fail, pc, stack, pos_stack);
            if self.skip_on {
                while i < input.len() && self.skip_parser(input[i]) {
                    i += 1;
                }
            }

            if fail {
                if let Some(frame) = stack.pop() {
                    use self::StackFrame::*;
                    match frame {
                        Backtrack(ret, j) => {
                            pc = ret;
                            i = j;
                            fail = false;
                        },
                        PrecedenceBacktrack(ret, a, j, jp, k, f, is_left) => {
                            if (jp.is_none() || i > jp.unwrap()) && i != j {
                                stack.push(PrecedenceBacktrack(ret, a, j, Some(i), k, true, is_left));
                                pc = a;
                                i = j;
                                fail = false;
                            } else if jp.is_some() {
                                i = jp.unwrap();
                                fail = f;
                                
                                if is_left {
                                    pc = self.jump_table[ret as usize];
                                    while let Some(&StackFrame::Backtrack(_, _)) = stack.get(stack.len() - 1) {
                                        stack.pop();
                                    }
                                } else {
                                    pc = ret + 1;
                                }
                            }
                        },
                        StackFrame::Return(_) => {
                            pos_stack.pop();
                        }
                    }
                } else {
                    break;
                }
            } else {
                use self::Instruction::*;
                match self.program[pc as usize] {
                    Char(c) => {
                        if i < input.len() && input[i] == c {
                            pc += 1;
                            i += 1;
                        } else {
                            fail = true;
                        }
                    },
                    TestChar(c, j) => {
                        if i < input.len() && input[i] == c {
                            pc += 1;
                            i += 1;
                        } else {
                            pc += j;
                        }
                    },
                    Any => {
                        if i < input.len() {
                            pc += 1;
                            i += 1;
                        } else {
                            fail = true;
                        }
                    },
                    TestAny(n, j) => {
                        if i + n < input.len() {
                            pc += 1;
                            i += n;
                        } else {
                            pc += j;
                        }
                    },
                    CharRange(l, r) => {
                        if i < input.len() && input[i] >= l && input[i] <= r {
                            pc += 1;
                            i += 1;
                        } else {
                            fail = true;
                        }
                    },
                    CharRangeLink(l, r, j) => {
                        if i < input.len() && input[i] >= l && input[i] <= r {
                            pc += j;
                            i += 1;
                        } else {
                            pc += 1;
                        }
                    }
                    Choice(j) => {
                        stack.push(StackFrame::Backtrack(pc + j, i));
                        pc += 1;
                    }
                    Jump(j) => {
                        pc += j;
                    }
                    Call(j) => {
                        stack.push(StackFrame::Return(pc + 1));
                        pc += j;
                    },
                    PrecedenceCall(n, k, is_left) => {
                        let pc_clone = pc;
                        let stack_update = {
                            let mut result = false;
                            let memo = stack.iter().find(|&&x| match x {
                                StackFrame::PrecedenceBacktrack(_, a, j, _, _, _, _) => {
                                    pc + n == a && i == j
                                },
                                _ => false
                            });
                            match memo {
                                Some(&StackFrame::PrecedenceBacktrack(_, _, _, jp, kp, _, _)) => {
                                    match jp {
                                        Some(jr) => {
                                            if k >= kp {
                                                pc += 1;
                                                i = jr;
                                            } else {
                                                fail = true;
                                            }
                                        },
                                        None => {
                                            fail = true;
                                        }
                                    }
                                },
                                None => {
                                    pc += n;
                                    result = true;
                                },
                                _ => { }
                            }
                            result
                        };
                        if stack_update {
                            stack.push(StackFrame::PrecedenceBacktrack(pc_clone, pc_clone + n, i, None, k, false, is_left));
                        }
                    },
                    Return => {
                        if let Some(frame) = stack.pop() {
                            if let StackFrame::Return(ret) = frame {
                                pc = ret;
                            } else if let StackFrame::PrecedenceBacktrack(ret, a, j, jp, k, _, is_left) = frame {
                                if jp.is_none() || i > jp.unwrap() {
                                    stack.push(StackFrame::PrecedenceBacktrack(ret, a, j, Some(i), k, false, is_left));
                                    pc = a;
                                    i = j;
                                } else {
                                    i = jp.unwrap();
                                    
                                    if is_left {
                                        pc = self.jump_table[ret as usize];
                                        while let Some(&StackFrame::Backtrack(_, _)) = stack.get(stack.len() - 1) {
                                            stack.pop();
                                        }
                                    } else {
                                        pc = ret + 1;
                                    }
                                }
                            }
                        }
                    },
                    Commit(j) => {
                        stack.pop();
                        pc += j;
                    },
                    BackCommit(j) => {
                        if let Some(frame) = stack.pop() {
                            if let StackFrame::Backtrack(_, k) = frame {
                                pc += j;
                                i = k;
                            }
                        }
                    },
                    PartialCommit(j) => {
                        if stack.len() > 1 {
                            pc += j;
                            let pos = stack.len() - 1;
                            match stack[pos] {
                                StackFrame::Backtrack(p, _) => { 
                                    stack[pos] = StackFrame::Backtrack(p, i);
                                },
                                _ => { }
                            }
                        }
                    },
                    PushPos(id) => {
                        pos_stack.push((id, i));
                        pc += 1;
                    },
                    SavePos => {
                        if let Some((id, j)) = pos_stack.pop() {
                            if j != i {
                                match T::from_str(self.rule_names[id].as_str()) {
                                    Ok(marker) => result.insert((marker, j, i)),
                                    Err(e) => return Err(Error::MarkerError(e))
                                };
                            }
                        }
                        pc += 1;
                    },
                    Fail => {
                        fail = true;
                    },
                    FailTwice => {
                        stack.pop();
                        fail = true;
                    },
                    Stop => {
                        if i < input.len() { fail = true; }
                        break;
                    },
                    ToggleSkip => {
                        self.skip_on = !self.skip_on;
                        pc += 1;
                    }
                }
            }
        }

        if !fail && i == input.len() {
            Ok(result.drain().collect())
        } else {
            Err(Error::MachineError(i))
        }
    }

    pub fn get_jump_table(program : &Vec<Instruction>) -> Vec<isize> {
        let mut result = (0..program.len()).map(|_| -1).collect::<Vec<isize>>();
        let mut current : isize = -1;
        
        for i in (0..program.len()).rev() {
            match program[i] {
                Instruction::Return => current = i as isize,
                _ => { }
            }
            result[i] = current;
        }
        result
    }

    pub fn new(grammar : &str) -> Result<Machine<T>, usize> {
        let mut token_result = parser::tokenize(grammar);
        let mut parse_tree = parser::parse(token_result.0, token_result.1)?;
        let program = parse_tree.compile();

        let mut rules = token_result.2.drain().collect::<Vec<(Vec<u8>, i32)>>();
        rules.sort_by(|a, b| a.1.cmp(&b.1));
        let rules_map = rules.drain(..).map(|x| String::from_utf8(x.0).ok().unwrap()).collect();
        let jump_table = Machine::<T>::get_jump_table(&program);

        Ok(Machine {
            program: program,
            rule_names: rules_map,
            skip: vec![],
            skip_on: false,
            jump_table: jump_table,
            marker: PhantomData
        })
    }

    pub fn from_path(path : &Path) -> Result<Machine<T>, usize> {
        let mut file = File::open(path).ok().expect("rip");
        let mut contents = String::new();
        file.read_to_string(&mut contents).ok();
        Machine::new(contents.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn execute_test(program : Vec<Instruction>,
        subjects : &Vec<&str>,
        expected : &Vec<bool>,
        rule_names : Vec<String>)
    {
        let jump_table = Machine::<String>::get_jump_table(&program);
        let mut machine = Machine::<String> {
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
            println!("{:?}", result);
            println!("{}", subjects[i]);
            assert!(!fail == expected[i]);
        }
    }

    fn execute_test_with_skip(program : Vec<Instruction>,
        skip : Vec<(u8, u8)>,
        subjects : &Vec<&str>,
        expected : &Vec<bool>,
        rule_names : Vec<String>)
    {
        let jump_table = Machine::<String>::get_jump_table(&program);
        let mut machine = Machine::<String> {
            program: program,
            rule_names: rule_names,
            skip: vec![],
            skip_on: false,
            jump_table: jump_table,
            marker: PhantomData
        };
        machine.skip = skip;
        machine.skip_on = true;
        assert!(subjects.len() == expected.len());
        for i in 0..expected.len() {
            let result = machine.execute(subjects[i].to_string().into_bytes());
            let fail = result.is_err();
            println!("{}", subjects[i]);
            assert!(!fail == expected[i]);
        }
    }

    #[test]
    fn simple_char() { // 'a'
        let program = vec![
            Instruction::Char('a' as u8),
            Instruction::Stop
        ];
        let subjects = vec!["a", "aa"];
        let expected = vec![true, false];
        execute_test(program, &subjects, &expected, vec![]);
    }

    #[test]
    fn zero_or_more_chars() { // 'a'*
        let program = vec![
            Instruction::Choice(3),
            Instruction::Char('a' as u8),
            Instruction::Commit(-2),
            Instruction::Stop
        ];
        let subjects = vec!["", "a", "aaa", "b"];
        let expected = vec![true, true, true, false];
        execute_test(program, &subjects, &expected, vec![]);
    }

    #[test]
    fn one_or_more_chars() { // 'a'+
        let program = vec![
            Instruction::Char('a' as u8),
            Instruction::Choice(3),
            Instruction::Char('a' as u8),
            Instruction::Commit(-2),
            Instruction::Stop
        ];
        let subjects = vec!["a", "aaa", "b", ""];
        let expected = vec![true, true, false, false];
        execute_test(program, &subjects, &expected, vec![]);
    }

    #[test]
    fn simple_char_sequence() { // 'a''b'
        let program = vec![
            Instruction::Char('a' as u8),
            Instruction::Char('b' as u8),
            Instruction::Stop
        ];
        let subjects = vec!["ab", "a", "b", ""];
        let expected = vec![true, false, false, false];
        execute_test(program, &subjects, &expected, vec![]);
    }

    #[test]
    fn simple_char_choice() { // 'a'/'b'/'c'
        let program = vec![
            Instruction::Choice(6),
            Instruction::Choice(3),
            Instruction::Char('a' as u8),
            Instruction::Commit(2),
            Instruction::Char('b' as u8),
            Instruction::Commit(2),
            Instruction::Char('c' as u8),
            Instruction::Stop
        ];
        let subjects = vec!["a", "b", "c", "abc", ""];
        let expected = vec![true, true, true, false, false];
        execute_test(program, &subjects, &expected, vec![]);
    }

    #[test]
    fn simple_subparser() { // main { 'a'b+ } b { 'b' }
        let program1 = vec![
            Instruction::Char('a' as u8), //-- main
            Instruction::Call(5),         // | (b)
            Instruction::Choice(3),       // |
            Instruction::Call(3),         // | (b)
            Instruction::Commit(-2),      // |
            Instruction::Stop,            //-'
            Instruction::Char('b' as u8), //-- b
            Instruction::Return           //-' 
        ];
        let program2 = vec![
            Instruction::Call(4),         // -- entry point (main)
            Instruction::Jump(9),         // -' (exit point)
            Instruction::Char('b' as u8), // -- b
            Instruction::Return,          // -'
            Instruction::Char('a' as u8), // -- main
            Instruction::Call(-3),        //  | (b)
            Instruction::Choice(3),       //  |
            Instruction::Call(-5),        //  | (b)
            Instruction::Commit(-2),      //  |
            Instruction::Return,          // -'
            Instruction::Stop             // -- exit point
        ];
        let subjects = vec!["ab", "abbb", "a", ""];
        let expected = vec![true, true, false, false];
        execute_test(program1, &subjects, &expected, vec![]);
        execute_test(program2, &subjects, &expected, vec![]);
    }

    #[test]
    fn three_subparser() {
        /*
            main { a b c }
            a { 'a' / 'z' }
            b { 'b'* }
            c { a / b }
        */
        let program = vec![
            Instruction::Call(16),        // -- entry point (main)
            Instruction::Jump(19),        // -' (exit point)
            Instruction::Choice(3),       // -- a
            Instruction::Char('a' as u8), //  |
            Instruction::Commit(2),       //  |
            Instruction::Char('z' as u8), //  |
            Instruction::Return,          // -'
            Instruction::Choice(3),       // -- b
            Instruction::Char('b' as u8), //  |
            Instruction::Commit(-2),      //  |
            Instruction::Return,          // -'
            Instruction::Choice(3),       // -- c
            Instruction::Call(-10),       //  | (a)
            Instruction::Commit(2),       //  |
            Instruction::Call(-7),        //  | (b)
            Instruction::Return,          // -'
            Instruction::Call(-14),       // -- main (a)
            Instruction::Call(-10),       //  | (b)
            Instruction::Call(-7),        //  | (c)
            Instruction::Return,          // -'
            Instruction::Stop             // -- exit point
        ];
        let subjects = vec!["a", "ab", "aba", "abba", "z", "zbbaz", "garbage", ""];
        let expected = vec![true, true, true, true, true, false, false, false];
        execute_test(program, &subjects, &expected, vec![]);
    }

    #[test]
    fn self_reference_impossible_parser() {
        /*
            main { 'a'b;main }
            b { 'b'b }
        */
        let program = vec![
            Instruction::Call(5),         // -- entry point (main)
            Instruction::Jump(8),         // -' (exit point)
            Instruction::Char('b' as u8), // -- b
            Instruction::Call(-1),        //  | (b)
            Instruction::Return,          // -'
            Instruction::Char('a' as u8), // -- main
            Instruction::Call(-4),        //  | (b)
            Instruction::Call(-2),        //  | (main)
            Instruction::Return,          // -'
            Instruction::Stop             // -- exit point
        ];
        let subjects = vec!["a", "ab", "ababab", ""];
        let expected = vec![false, false, false, false];
        execute_test(program, &subjects, &expected, vec![]);
    }

    #[test]
    fn simple_token_stream_result() { // main { 'a'b+ } b { 'b' }
        let program = vec![
            Instruction::Call(6),         // -- entry point (main)
            Instruction::Jump(13),        // -' (exit point)
            Instruction::PushPos(1),      // -- b
            Instruction::Char('b' as u8), //  |
            Instruction::SavePos,         //  |
            Instruction::Return,          // -'
            Instruction::PushPos(0),      // -- main
            Instruction::Char('a' as u8), //  |
            Instruction::Call(-6),        //  | (b)
            Instruction::Choice(3),       //  |
            Instruction::Call(-8),        //  | (b)
            Instruction::Commit(-2),      //  |
            Instruction::SavePos,         //  |
            Instruction::Return,          // -'
            Instruction::Stop             // -- exit point
        ];
        let subjects = vec!["ab", "abbbb", "a", "b"];
        let expected = vec![true, true, false, false];
        let rule_names = vec!["main".to_string(), "b".to_string()];
        execute_test(program, &subjects, &expected, rule_names);
    }

    #[test]
    fn simple_char_range() { // main { ['a'..'z']* }
        let program = vec![
            Instruction::Call(2),
            Instruction::Stop,
            Instruction::Choice(3),
            Instruction::CharRange('a' as u8, 'z' as u8),
            Instruction::Commit(-2),
            Instruction::Return
        ];
        let subjects = vec!["a", "b", "z", "aaa", "zzz", "abcdefghijkxyz", "a.z"];
        let expected = vec![true, true, true, true, true, true, false];
        execute_test(program, &subjects, &expected, vec![]);
    }

    #[test]
    fn char_range_links() { // main { ['a'..'b''c'..'c''e'..'e']* }
        let program = vec![
            Instruction::Call(2),
            Instruction::Stop,
            Instruction::Choice(5),
            Instruction::CharRangeLink('a' as u8, 'b' as u8, 3),
            Instruction::CharRangeLink('c' as u8, 'c' as u8, 2),
            Instruction::CharRange('e' as u8, 'e' as u8),
            Instruction::Commit(-4),
            Instruction::Return
        ];
        let subjects = vec!["a", "b", "c", "e", "abce", "ecba", "acbe", "d", "f", "abcde"];
        let expected = vec![true, true, true, true, true, true, true, false, false ,false];
        execute_test(program, &subjects, &expected, vec![]);
    }

    #[test]
    fn skip_parser() { // main { ('a';'b')* } skip { [' '] }
        let program = vec![
            Instruction::Call(2),
            Instruction::Stop,
            Instruction::Choice(4),
            Instruction::Char('a' as u8),
            Instruction::Char('b' as u8),
            Instruction::Commit(-3),
            Instruction::Return
        ];
        let skip = vec![(' ' as u8, ' ' as u8)];
        let subjects = vec!["ababab", "ab a b ab", "ab a  b  a b", " a   b ", "c"];
        let expected = vec![true, true, true, true, false];
        execute_test_with_skip(program, skip, &subjects, &expected, vec![]);
    }

    #[test]
    fn skip_parser_with_toggle() { // main { #s;'a';' ';#s;'b' } skip { [' '] }
        let program = vec![
            Instruction::Call(2),
            Instruction::Stop,
            Instruction::ToggleSkip,
            Instruction::Char('a' as u8),
            Instruction::Char(' ' as u8),
            Instruction::ToggleSkip,
            Instruction::Char('b' as u8),
            Instruction::Return
        ];
        let skip = vec![(' ' as u8, ' ' as u8)];
        let subjects = vec!["a b", "a    b", "   a   b    ", "ab"];
        let expected = vec![true, true, true, false];
        execute_test_with_skip(program, skip, &subjects, &expected, vec![]);
    }

    #[test]
    fn partial_commit_zero_or_more() { // main { 'a'* }
        let program = vec![
            Instruction::Call(2),
            Instruction::Stop,
            Instruction::Choice(3),
            Instruction::Char('a' as u8),
            Instruction::PartialCommit(-1),
            Instruction::Return
        ];
        let subjects = vec!["", "a", "aaa", "b"];
        let expected = vec![true, true, true, false];
        execute_test(program, &subjects, &expected, vec![]);
    }

    #[test]
    fn not_predicate() { // main { !'a' ['a'..'b']+ }
        let program = vec![
            Instruction::Call(2),
            Instruction::Stop,
            Instruction::Choice(3),
            Instruction::Char('a' as u8),
            Instruction::FailTwice,
            Instruction::CharRange('a' as u8, 'b' as u8),
            Instruction::Choice(3),
            Instruction::CharRange('a' as u8, 'b' as u8),
            Instruction::PartialCommit(-1),
            Instruction::Return
        ];
        let subjects = vec!["b", "ba", "bababbaa", "a", "ab"];
        let expected = vec![true, true, true, false, false];
        execute_test(program, &subjects, &expected, vec![]);
    }

    #[test]
    fn ambersand_predicate() { // main { &'a' ['a'..'b']+ }
        let program = vec![
            Instruction::Call(2),
            Instruction::Stop,
            Instruction::Choice(5),
            Instruction::Choice(3),
            Instruction::Char('a' as u8),
            Instruction::FailTwice,
            Instruction::FailTwice,
            Instruction::CharRange('a' as u8, 'b' as u8),
            Instruction::Choice(3),
            Instruction::CharRange('a' as u8, 'b' as u8),
            Instruction::PartialCommit(-1),
            Instruction::Return
        ];
        let subjects = vec!["a", "aa", "ab", "abbaabaa", "b", "ba"];
        let expected = vec![true, true, true, true, false, false];
        execute_test(program, &subjects, &expected, vec![]);
    }

    #[test]
    fn optional_predicate() { // main { 'a'? 'b' }
        let program = vec![
            Instruction::Call(2),
            Instruction::Stop,
            Instruction::Choice(3),
            Instruction::Char('a' as u8),
            Instruction::Commit(1),
            Instruction::Char('b' as u8),
            Instruction::Return
        ];
        let subjects = vec!["ab", "b", "c", "aa"];
        let expected = vec![true, true, false, false];
        execute_test(program, &subjects, &expected, vec![]);
    }

    #[test]
    fn direct_left_recursion() {
        let program = vec![
            Instruction::Call(2),
            Instruction::Stop,
            Instruction::Choice(5),
            Instruction::PrecedenceCall(-1, 0, true),
            Instruction::Char(b'+'),
            Instruction::Char(b'n'),
            Instruction::Commit(2),
            Instruction::Char(b'n'),
            Instruction::Return
        ];
        let subjects = vec!["n", "n+n+n", "n+n", "n+n+n+n", "n+", "+n", "n+n+", "+n+n+"];
        let expected = vec![true, true, true, true, false, false, false, false];
        execute_test(program, &subjects, &expected, vec![]);
    }

    #[test]
    fn direct_left_recursion_with_tail() {
        let program = vec![
            Instruction::Call(2),
            Instruction::Stop,
            Instruction::Choice(5),
            Instruction::PrecedenceCall(-1, 0, true),
            Instruction::Char(b'+'),
            Instruction::Char(b'n'),
            Instruction::Commit(2),
            Instruction::Char(b'n'),
            Instruction::Char(b';'),
            Instruction::Return
        ];
        let subjects = vec!["n;", "n+n;", "n+n+n+n+n;", "n", "n+n", "n+", ";"];
        let expected = vec![true, true, true, false, false, false, false];
        execute_test(program, &subjects, &expected, vec![]);
    }
}
