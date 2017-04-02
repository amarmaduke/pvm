use ast;

/* Grammar
    main {  }



*/

#[derive(Debug)]
enum Token {
    OpenBrace,
    CloseBrace,
    OpenParen,
    CloseParen,
    OpenBracket,
    CloseBracket,
    SingleQuote,
    DoubleQuote,
    Dot,
    Plus,
    Asterik,
    Question,
    Exclamation,
    Ambersand,
    Slash,
    Name(String)
}

fn tokenize(grammar : String) -> Vec<Token> {
    let iterator = grammar.chars();
    let tokens = vec![];
    let name = String::new();

    while let item = iterator.next() {
        match item {
            '{' => tokens.push(Token::OpenBrace),
            '}' => tokens.push(Token::CloseBrace),
            '(' => tokens.push(Token::OpenParen),
            ')' => tokens.push(Token::CloseParen),
            '[' => tokens.push(Token::OpenBracket),
            ']' => tokens.push(Token::CloseBracket),
            '\'' => tokens.push(Token::SingleQuote),
            '\"' => tokens.push(Token::DoubleQuote),
            '.' => tokens.push(Token::Dot),
            '+' => tokens.push(Token::Plus),
            '*' => tokens.push(Token::Asterik),
            '?' => tokens.push(Token::Question),
            '!' => tokens.push(Token::Exclamation),
            '&' => tokens.push(Token::Ambersand),
            '/' => tokens.push(Token::Slash),
            _ if item.is_alphanumeric() => name.push(item),
            _ if item.is_whitespace() && name.len() > 0 => {
                tokens.push(Token::Name(name.clone()));
                name.clear();
            }
            _ => { }
        }
    }

    tokens
}


