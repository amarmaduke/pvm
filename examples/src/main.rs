
extern crate peg;

use peg::*;
use peg::machine::*;

fn execute_test(program : Vec<Instruction>, subjects : &Vec<&str>, expected : &Vec<bool>) {
    let mut machine = Machine::new(program);
    assert!(subjects.len() == expected.len());
    for i in 0..expected.len() {
        machine.execute(subjects[i].to_string().into_bytes());
        println!("{}", machine.fail);
        assert!(!machine.fail == expected[i]);
    }
}

fn many_simple_subparser() {
    /*
        main { a b c }
        a { 'a' / 'z' }
        b { 'b'* }
        c { a / b }
    */
    let program = vec![
        Instruction::Call(16),
        Instruction::Jump(19),
        Instruction::Choice(3),
        Instruction::Char('a' as u8),
        Instruction::Commit(2),
        Instruction::Char('z' as u8),
        Instruction::Return,
        Instruction::Choice(3),
        Instruction::Char('b' as u8),
        Instruction::Commit(-2),
        Instruction::Return,
        Instruction::Choice(3),
        Instruction::Call(-10),
        Instruction::Commit(2),
        Instruction::Call(-7),
        Instruction::Return,
        Instruction::Call(-14),
        Instruction::Call(-10),
        Instruction::Call(-7),
        Instruction::Return,
        Instruction::Stop
    ];
    let subjects = vec!["z"];
    let expected = vec![true];
    execute_test(program, &subjects, &expected);
}

fn main() {
    many_simple_subparser();
}