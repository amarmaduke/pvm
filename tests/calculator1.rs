extern crate pvm;

use std::path::Path;
use std::io;
use std::str::FromStr;
use std::cmp::Ordering;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
enum Rules {
    Main,
    Expr,
    Plus,
    Minus,
    Times,
    Divide,
    Num,
    Open,
    Close,
    S,
    Ws
}

impl FromStr for Rules {
    type Err = usize;

    fn from_str(s : &str) -> Result<Self, Self::Err> {
        match s {
            "main" => Ok(Rules::Main),
            "expr" => Ok(Rules::Expr),
            "plus" => Ok(Rules::Plus),
            "minus" => Ok(Rules::Minus),
            "times" => Ok(Rules::Times),
            "divide" => Ok(Rules::Divide),
            "num" => Ok(Rules::Num),
            "open" => Ok(Rules::Open),
            "close" => Ok(Rules::Close),
            "s" => Ok(Rules::S),
            "ws" => Ok(Rules::Ws),
            _ => Err(0)
        }
    }
}

fn order_rule(left : &Rules, right : &Rules) -> Ordering {
    use self::Rules::*;
    match *left {
        Expr => {
            match *right {
                Expr => Ordering::Equal,
                _ => Ordering::Less,
            }
        },
        Num => {
            match *right {
                Num => Ordering::Equal,
                _ => Ordering::Greater
            }
        },
        _ => Ordering::Equal
    }
}

#[derive(Debug)]
enum Syntax {
    Plus(Box<Syntax>, Box<Syntax>),
    Minus(Box<Syntax>, Box<Syntax>),
    Times(Box<Syntax>, Box<Syntax>),
    Divide(Box<Syntax>, Box<Syntax>),
    Negation(Box<Syntax>),
    Grouping(Box<Syntax>),
    Number(i64)
}

impl Syntax {
    pub fn eval(&self) -> i64 {
        Syntax::eval_syntax(self)
    }

    fn eval_syntax(tree : &Syntax) -> i64 {
        use self::Syntax::*;
        match *tree {
            Plus(ref left, ref right) => Syntax::eval_syntax(left) + Syntax::eval_syntax(right),
            Minus(ref left, ref right) => Syntax::eval_syntax(left) - Syntax::eval_syntax(right),
            Times(ref left, ref right) => Syntax::eval_syntax(left) * Syntax::eval_syntax(right),
            Divide(ref left, ref right) => Syntax::eval_syntax(left) / Syntax::eval_syntax(right),
            Negation(ref nested) => -Syntax::eval_syntax(nested),
            Grouping(ref nested) => Syntax::eval_syntax(nested),
            Number(value) => value
        }
    }

    pub fn parse(input : &str, data : &mut Vec<(Rules, usize, usize)>) -> Syntax {
        data.sort_by(|a, b| a.1.cmp(&b.1).then(b.2.cmp(&a.2)).then(order_rule(&a.0, &b.0)));
        println!("{:?}", data);
        Syntax::parse_expr(0, input, data).0
    }

    fn parse_expr(index : usize, input : &str, data : &mut Vec<(Rules, usize, usize)>) -> (Syntax, usize) {
        use self::Rules::*;
        let nested = data[index + 1].clone();
        match nested.0 {
            Expr => {
                let left = Syntax::parse_expr(index + 1, input, data);
                let op = data[left.1].clone();
                let right = Syntax::parse_expr(left.1 + 1, input, data);
                match op.0 {
                    Plus => (Syntax::Plus(Box::new(left.0), Box::new(right.0)), right.1),
                    Minus => (Syntax::Minus(Box::new(left.0), Box::new(right.0)), right.1),
                    Times => (Syntax::Times(Box::new(left.0), Box::new(right.0)), right.1),
                    Divide => (Syntax::Divide(Box::new(left.0), Box::new(right.0)), right.1),
                    _ => panic!("Impossible 1")
                }
            },
            Minus => {
                let interior = Syntax::parse_expr(index + 2, input, data);
                (Syntax::Negation(Box::new(interior.0)), interior.1)
            },
            Open => {
                let interior = Syntax::parse_expr(index + 2, input, data);
                (Syntax::Grouping(Box::new(interior.0)), interior.1 + 1)
            },
            Num => {
                let slice = &input[nested.1..nested.2];
                let number = slice.trim().parse().ok().expect("Parse failed to be a number.");
                (Syntax::Number(number), index + 2)
            },
            _ => panic!("Impossible 2")
        }
    }
}



#[test]
fn calculator() {
    let path = Path::new("./tests/grammars/calculator1.peg");
    match pvm::Machine::<Rules>::from_path(&path) {
        Ok(mut machine) => {
            println!("Hello");
            let input = "1+2+3 * 2 *(  2+3  +4)";
            let mut temp = machine.execute(input.to_string().into_bytes()).ok().unwrap();
            let mut result = temp.drain(..)
                .filter(|x| x.0 != Rules::S && x.0 != Rules::Ws)
                .collect();
            let tree = Syntax::parse(input, &mut result);
            println!("{:?}", tree);
            let eval = tree.eval();
            println!("{:?}", eval);
        },
        Err(x) => { }
    }
}