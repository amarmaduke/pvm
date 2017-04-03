use ast;

/* Grammar
    main {  }



*/

#[derive(Debug, Eq, PartialEq)]
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
    Name(usize, usize)
}

fn tokenize(grammar : &String) -> Vec<Token> {
    let mut iterator = grammar.chars();
    let mut tokens = vec![];
    let mut name = (-1, -1);
    let mut i = 0;

    while let Some(item) = iterator.next() {
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
            _ if item.is_alphanumeric() => {
                if name == (-1, -1) {
                    name = (i, i);
                } else {
                    name.1 += 1;
                }
            },
            _ if item.is_whitespace() && name != (-1, -1) => {
                tokens.push(Token::Name(name.0 as usize, name.1 as usize));
                name = (-1, -1);
            }
            _ => { }
        }
        i += 1;
    }
    tokens
}

fn parse(grammar : &String, tokens : Vec<Token>) -> Result<ast::Grammar, u8> {
    let mut iterator = tokens.iter();
    let mut grammar_object = ast::Grammar { rules: vec![], main: 0 };

    while let Some(token) = iterator.next() {
        if let &Token::Name(name_start, name_end) = token {
            if let Some(brace_token) = iterator.next() {
                if brace_token == &Token::OpenBrace {
                    let pattern = parse_pattern(grammar, &mut iterator);
                    match pattern {
                        Ok(p) => grammar_object.rules.push(p),
                        Err(x) => return Err(x)
                    }
                } else {
                    return Err(0);
                }
            } else {
                return Err(0);
            }
        } else {
            return Err(0);
        }
    }
    Ok(grammar_object)
}

fn parse_pattern<Iter>(grammar : &String, iterator : &mut Iter) -> Result<ast::Pattern, u8>
    where Iter : Iterator
{
    let mut result = None;

    while let Some(token) = iterator.next() {
        //match token {
        //    &Token::OpenParen => 
       // }
    }

    match result {
        Some(pattern) => Ok(pattern),
        None => Err(0)
    }
}




