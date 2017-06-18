
extern crate pvm;

use std::io;
use std::str::FromStr;

#[derive(Debug, PartialEq, Eq, Hash)]
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

fn main() {
    let minimal_grammar = "
        main { s expr ws }
        expr { 
            expr:1 '+' s expr:2
            / expr:1 minus expr:2
            / minus expr:4
            / [1-9]
        }

        minus { '-' s }
        
        s { ws* }
        ws { [ \\t\\r\\n] }
    ";
    let grammar = "
        main { s expr }
        expr { 
            expr:1 plus expr:2
            / expr:1 minus expr:2
            / expr:2 times expr:3
            / expr:2 divide expr:3
            / minus expr:4
            / open expr:1 s close
            / num
        }

        plus { '+' s }
        minus { '-' s }
        times { '*' s  }
        divide { '/' s }
        open { '(' s }
        close { ')' s }
        num { [1-9][0-9]* s }
        
        s { ws* }
        ws { [ \\t\\r\\n] }
    ";

    match pvm::Machine::<Rules>::new(&grammar) {
        Ok(mut machine) => {
            loop {
                let mut buffer = String::new();

                match io::stdin().read_line(&mut buffer) {
                    Ok(_) => {
                        if buffer == "quit" { return; }
                        let output = machine.execute(buffer.into_bytes());
                        println!("{:?}", output);
                    },
                    Err(error) => println!("Stdin Error: {}", error)
                }
            }
        },
        Err(x) => {
            println!("Encountered error: {}", x);
        }
    }

}