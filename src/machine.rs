

#[derive(Debug, Copy, Clone)]
enum StackFrame {
    Return(isize),
    Backtrack(isize, usize),
}

#[derive(Debug, Copy, Clone)]
pub enum Instruction {
    Char(u8),
    TestChar(u8, isize),
    Any,
    TestAny(usize, isize),
    CharRange(u8, u8),
    Choice(isize),
    Jump(isize),
    Call(isize),
    Return,
    Commit(isize),
    BackCommit(isize),
    PartialCommit(isize),
    PushPos,
    SavePos,
    Fail,
    FailTwice,
    Stop,
    ToggleSkip
}

pub struct Machine {
    program: Vec<Instruction>,
    pub skip : Vec<(u8, u8)>,
    pub skip_on : bool
}

impl Machine {

    pub fn skip_parser(&mut self, x : u8) -> bool {
        let mut result = false;
        for t in &self.skip {
            result |= x >= t.0 && x <= t.1;
        }

        result
    }

    pub fn execute(&mut self, input : Vec<u8>) -> Result<Vec<(usize, usize)>, u8> {
        let mut stack = Vec::new();
        let mut pos_stack = Vec::new();
        let mut result = Vec::new();
        let mut pc = 0;
        let mut i = 0;
        let mut fail = false;

        loop {
            if self.skip_on {
                while i < input.len() && self.skip_parser(input[i]) {
                    i += 1;
                }
            }

            if fail {
                if let Some(frame) = stack.pop() {
                    if let StackFrame::Backtrack(ret, j) = frame {
                        pc = ret;
                        i = j;
                        fail = false;
                    } else {
                        pos_stack.pop();
                    }
                } else {
                    break;
                }
            } else {
                match self.program[pc as usize] {
                    Instruction::Char(c) => {
                        if i < input.len() && input[i] == c {
                            pc += 1;
                            i += 1;
                        } else {
                            fail = true;
                        }
                    },
                    Instruction::TestChar(c, j) => {
                        if i < input.len() && input[i] == c {
                            pc += 1;
                            i += 1;
                        } else {
                            pc += j;
                        }
                    },
                    Instruction::Any => {
                        if i + 1 < input.len() {
                            pc += 1;
                            i += 1;
                        } else {
                            fail = true;
                        }
                    },
                    Instruction::TestAny(n, j) => {
                        if i + n < input.len() {
                            pc += 1;
                            i += n;
                        } else {
                            pc += j;
                        }
                    },
                    Instruction::CharRange(l, r) => {
                        if i < input.len() && input[i] >= l && input[i] <= r {
                            pc += 1;
                            i += 1;
                        } else {
                            fail = true;
                        }
                    },
                    Instruction::Choice(j) => {
                        stack.push(StackFrame::Backtrack(pc + j, i));
                        pc += 1;
                    }
                    Instruction::Jump(j) => {
                        pc += j;
                    }
                    Instruction::Call(j) => {
                        stack.push(StackFrame::Return(pc + 1));
                        pc += j;
                    }
                    Instruction::Return => {
                        if let Some(frame) = stack.pop() {
                            if let StackFrame::Return(ret) = frame {
                                pc = ret;
                            }
                        }
                    }
                    Instruction::Commit(j) => {
                        stack.pop();
                        pc += j;
                    },
                    Instruction::BackCommit(j) => {
                        if let Some(frame) = stack.pop() {
                            if let StackFrame::Backtrack(p, k) = frame {
                                pc += j;
                                i = k;
                            }
                        }
                    },
                    Instruction::PartialCommit(j) => {
                        if let Some(frame) = stack.pop() {
                            if let StackFrame::Backtrack(p, k) = frame {
                                pc += j;
                                stack.push(StackFrame::Backtrack(p, i));
                            }
                        }
                    },
                    Instruction::PushPos => {
                        pos_stack.push(i);
                        pc += 1;
                    },
                    Instruction::SavePos => {
                        if let Some(j) = pos_stack.pop() {
                            result.push((j, i));
                        }
                        pc += 1;
                    },
                    Instruction::Fail => {
                        fail = true;
                    },
                    Instruction::FailTwice => {
                        stack.pop();
                        fail = true;
                    },
                    Instruction::Stop => {
                        if i < input.len() { fail = true; }
                        break;
                    },
                    Instruction::ToggleSkip => {
                        self.skip_on = !self.skip_on;
                        pc += 1;
                    }
                }
            }
        }

        if !fail {
            Ok(result)
        } else {
            Err(0)
        }
    }

    pub fn new(program : Vec<Instruction>) -> Machine {
        Machine {
            program: program,
            skip: vec![],
            skip_on: false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn execute_test(program : Vec<Instruction>, subjects : &Vec<&str>, expected : &Vec<bool>) {
        let mut machine = Machine::new(program);
        assert!(subjects.len() == expected.len());
        for i in 0..expected.len() {
            let result = machine.execute(subjects[i].to_string().into_bytes());
            let fail = result.is_err();
            println!("{}", subjects[i]);
            assert!(!fail == expected[i]);
        }
    }

    fn execute_test_with_skip(program : Vec<Instruction>,
        skip : Vec<(u8, u8)>,
        subjects : &Vec<&str>,
        expected : &Vec<bool>) {

        let mut machine = Machine::new(program);
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
        execute_test(program, &subjects, &expected);
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
        execute_test(program, &subjects, &expected);
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
        execute_test(program, &subjects, &expected);
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
        execute_test(program, &subjects, &expected);
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
        execute_test(program, &subjects, &expected);
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
        execute_test(program1, &subjects, &expected);
        execute_test(program2, &subjects, &expected);
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
        execute_test(program, &subjects, &expected);
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
        execute_test(program, &subjects, &expected);
    }

    #[test]
    fn simple_token_stream_result() { // main { 'a'b+ } b { 'b' }
        let program = vec![
            Instruction::Call(6),         // -- entry point (main)
            Instruction::Jump(13),        // -' (exit point)
            Instruction::PushPos,         // -- b
            Instruction::Char('b' as u8), //  |
            Instruction::SavePos,         //  |
            Instruction::Return,          // -'
            Instruction::PushPos,         // -- main
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
        execute_test(program, &subjects, &expected);
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
        execute_test(program, &subjects, &expected);
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
        execute_test_with_skip(program, skip, &subjects, &expected);
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
        execute_test_with_skip(program, skip, &subjects, &expected);
    }

}
