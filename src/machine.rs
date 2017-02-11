

enum StackFrame {
    Return(isize),
    Backtrack(isize, usize),
}

pub enum Instruction {
    Char(u8),
    TestChar(u8, isize),
    Any,
    TestAny(usize, isize),
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
    Stop
}

pub struct Machine {
    program: Vec<Instruction>,
    stack: Vec<StackFrame>,
    pos_stack : Vec<usize>,
    pub result : Vec<(usize, usize)>,
    pc: isize,
    i: usize,
    pub fail: bool,
}

pub struct Token {

}

impl Machine {
    pub fn execute(&mut self, input : Vec<u8>) {
        self.stack.clear();
        self.pos_stack.clear();
        self.result.clear();
        self.pc = 0;
        self.i = 0;
        self.fail = false;

        loop {
            if self.fail {
                if let Some(frame) = self.stack.pop() {
                    if let StackFrame::Backtrack(ret, j) = frame {
                        self.pc = ret;
                        self.i = j;
                        self.fail = false;
                    } else {
                        self.pos_stack.pop();
                    }
                } else {
                    break;
                }
            } else {
                match self.program[self.pc as usize] {
                    Instruction::Char(c) => {
                        if self.i < input.len() && input[self.i] == c {
                            self.pc += 1;
                            self.i += 1;
                        } else {
                            self.fail = true;
                        }
                    },
                    Instruction::TestChar(c, j) => {
                        if self.i < input.len() && input[self.i] == c {
                            self.pc += 1;
                            self.i += 1;
                        } else {
                            self.pc += j;
                        }
                    },
                    Instruction::Any => {
                        if self.i + 1 < input.len() {
                            self.pc += 1;
                            self.i += 1;
                        } else {
                            self.fail = true;
                        }
                    },
                    Instruction::TestAny(n, j) => {
                        if self.i + n < input.len() {
                            self.pc += 1;
                            self.i += n;
                        } else {
                            self.pc += j;
                        }
                    },
                    Instruction::Choice(j) => {
                        self.stack.push(StackFrame::Backtrack(self.pc + j, self.i));
                        self.pc += 1;
                    }
                    Instruction::Jump(j) => {
                        self.pc += j;
                    }
                    Instruction::Call(j) => {
                        self.stack.push(StackFrame::Return(self.pc + 1));
                        self.pc += j;
                    }
                    Instruction::Return => {
                        if let Some(frame) = self.stack.pop() {
                            if let StackFrame::Return(ret) = frame {
                                self.pc = ret;
                            }
                        }
                    }
                    Instruction::Commit(j) => {
                        self.stack.pop();
                        self.pc += j;
                    },
                    Instruction::BackCommit(j) => {
                        if let Some(frame) = self.stack.pop() {
                            if let StackFrame::Backtrack(p, k) = frame {
                                self.pc += j;
                                self.i = k;
                            }
                        }
                    },
                    Instruction::PartialCommit(j) => {
                        if let Some(frame) = self.stack.pop() {
                            if let StackFrame::Backtrack(p, k) = frame {
                                self.pc += j;
                                self.stack.push(StackFrame::Backtrack(p, self.i));
                            }
                        }
                    },
                    Instruction::PushPos => {
                        self.pos_stack.push(self.i);
                        self.pc += 1;
                    },
                    Instruction::SavePos => {
                        if let Some(j) = self.pos_stack.pop() {
                            self.result.push((j, self.i));
                        }
                        self.pc += 1;
                    },
                    Instruction::Fail => {
                        self.fail = true;
                    },
                    Instruction::FailTwice => {
                        self.stack.pop();
                        self.fail = true;
                    },
                    Instruction::Stop => {
                        if self.i < input.len() { self.fail = true; }
                        break;
                    }
                }
            }
        }
    }

    pub fn new(program : Vec<Instruction>) -> Machine {
        Machine {
            program: program,
            stack: vec![],
            pos_stack: vec![],
            result: vec![],
            pc: 0,
            i: 0,
            fail: false,
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
            machine.execute(subjects[i].to_string().into_bytes());
            println!("{}", machine.fail);
            assert!(!machine.fail == expected[i]);
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
            Instruction::Jump(13),         // -' (exit point)
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
        let mut machine = Machine::new(program);
        machine.execute("ab".to_string().into_bytes());
        println!("{:?}", machine.result);
        assert!(false);
    }

}
