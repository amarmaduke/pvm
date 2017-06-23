extern crate pvm;

use pvm::Machine;


fn execute_test(grammar : &str, subjects : &Vec<&str>, expected : &Vec<bool>) {
    let machine_result = Machine::<String>::new(&grammar);
    assert!(machine_result.is_ok());
    let mut machine = machine_result.ok().unwrap();
    assert!(subjects.len() == expected.len());
    for i in 0..expected.len() {
        let result = machine.execute(subjects[i].to_string().into_bytes());
        let fail = result.is_err();
        println!("{}", subjects[i]);
        println!("{:?}", result);
        assert!(!fail == expected[i]);
    }
}

#[test]
fn simple() {
    let grammar = "
        main { 
            main:1 '+' main:2
            / main:2 '*' main:3
            / 'n'
        }
    ";
    let subjects = vec!["n", "n+n", "n*n", "n+n+n", "n*n*n", "n+n*n", "n*n+n"];
    let expected = vec![true, true, true, true, true, true, true];
    execute_test(grammar, &subjects, &expected);
}