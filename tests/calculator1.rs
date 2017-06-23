extern crate rand;
extern crate pvm;

use std::path::Path;
use std::io;
use std::str::FromStr;
use std::cmp::Ordering;

use rand::Rng;

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
    Number(f64)
}

impl Syntax {
    pub fn eval(&self) -> f64 {
        Syntax::eval_syntax(self)
    }

    fn eval_syntax(tree : &Syntax) -> f64 {
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

    pub fn print(&self) -> String {
        use self::Syntax::*;
        match *self {
            Plus(ref left, ref right) => format!("({}+{})", left.print(), right.print()),
            Minus(ref left, ref right) => format!("({}-{})", left.print(), right.print()),
            Times(ref left, ref right) => format!("({}*{})", left.print(), right.print()),
            Divide(ref left, ref right) => format!("({}/{})", left.print(), right.print()),
            Negation(ref nested) => format!("-{}", nested.print()),
            Grouping(ref nested) => format!("({})", nested.print()),
            Number(value) => format!("{}", value)
        }
    }

    pub fn gen(total : usize, rng : &mut rand::Rng) -> Syntax {
        if total > 0 {
            let choice = rng.next_u32() % 6;
            match choice {
                0 => Syntax::Plus(Box::new(Syntax::gen(total - 1, rng)), Box::new(Syntax::gen(total - 1, rng))),
                1 => Syntax::Minus(Box::new(Syntax::gen(total - 1, rng)), Box::new(Syntax::gen(total - 1, rng))),
                2 => Syntax::Times(Box::new(Syntax::gen(total - 1, rng)), Box::new(Syntax::gen(total - 1, rng))),
                3 => Syntax::Divide(Box::new(Syntax::gen(total - 1, rng)), Box::new(Syntax::gen(total - 1, rng))),
                4 => Syntax::Negation(Box::new(Syntax::gen(total - 1, rng))),
                5 => Syntax::Grouping(Box::new(Syntax::gen(total - 1, rng))),
                _ => panic!("Impossible.")
            }
        } else {
            let number = (rng.next_u64() % 100) as f64;
            Syntax::Number(number)
        }
    }

    pub fn parse(input : &str, data : &mut Vec<(Rules, usize, usize)>) -> Syntax {
        data.sort_by(|a, b| a.1.cmp(&b.1).then(b.2.cmp(&a.2)).then(order_rule(&a.0, &b.0)));
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

fn run_test(tree : Syntax) {
    let path = Path::new("./tests/grammars/calculator1.peg");
    match pvm::Machine::<Rules>::from_path(&path) {
        Ok(mut machine) => {
            let input = tree.print();
            println!("input: {}", input);
            let mut temp = match machine.execute(input.to_string().into_bytes()) {
                Ok(x) => x,
                Err(x) => panic!("Parse Error: {:?}", x)
            };
            let mut result = temp.drain(..)
                .filter(|x| x.0 != Rules::S && x.0 != Rules::Ws)
                .collect();
            let new_tree = Syntax::parse(&input, &mut result);
            println!("old: {:?}, new: {:?}", tree, new_tree);
            let new_eval = new_tree.eval();
            let old_eval = tree.eval();
            if new_eval.is_nan() || old_eval.is_nan() {
                assert_eq!(new_eval.is_nan(), old_eval.is_nan());
            } else {
                assert_eq!(new_eval, old_eval);
            }
        },
        Err(x) => {
            println!("Error: {}", x); 
            assert!(false);
        }
    }
}

#[test]
fn constants() {
    let data = vec![
        Syntax::Number(0f64),
        Syntax::Number(1f64),
        Syntax::Number(2f64),
        Syntax::Number(1000f64),
        Syntax::Number(10000000f64),
    ];
    for tree in data {
        run_test(tree)
    }
}

#[test]
fn simple_expressions() {
    let data = vec![
        Syntax::Grouping(Box::new(Syntax::Divide(
            Box::new(Syntax::Plus(
                Box::new(Syntax::Number(4f64)),
                Box::new(Syntax::Number(6f64)))),
            Box::new(Syntax::Negation(Box::new(Syntax::Number(6f64))))
        )))
    ];
    for tree in data {
        run_test(tree)
    }
}

#[test]
fn random_expressions() {
    let mut rng = rand::thread_rng();
    for _ in 0..20 {
        let total = (rng.next_u32() % 10) as usize;
        run_test(Syntax::gen(total, &mut rng));
    }
}
