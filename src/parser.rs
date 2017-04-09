use std::iter::{Peekable};
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
    Name(usize, usize),
    Letter(u8)
}

fn tokenize(grammar : &String) -> Vec<Token> {
    let mut iterator = grammar.chars();
    let mut tokens = vec![];
    let mut name = (-1, -1);
    let mut i = 0;
    let mut in_quote = false;
    let mut escaped = false;

    while let Some(item) = iterator.next() {
        if in_quote {
            if escaped {
                tokens.push(Token::Letter(item as u8));
                escaped = false;
            } else {
                match item {
                    '\'' => {
                        tokens.push(Token::SingleQuote); 
                        in_quote = false;
                    },
                    '\"' => {
                        tokens.push(Token::DoubleQuote); 
                        in_quote = false;
                    },
                    '\\' => { escaped = true; },
                    _ => tokens.push(Token::Letter(item as u8))
                }
            }
        } else {
            match item {
                '{' => tokens.push(Token::OpenBrace),
                '}' => tokens.push(Token::CloseBrace),
                '(' => tokens.push(Token::OpenParen),
                ')' => tokens.push(Token::CloseParen),
                '[' => tokens.push(Token::OpenBracket),
                ']' => tokens.push(Token::CloseBracket),
                '.' => tokens.push(Token::Dot),
                '+' => tokens.push(Token::Plus),
                '*' => tokens.push(Token::Asterik),
                '?' => tokens.push(Token::Question),
                '!' => tokens.push(Token::Exclamation),
                '&' => tokens.push(Token::Ambersand),
                '/' => tokens.push(Token::Slash),
                '\'' => {
                    tokens.push(Token::SingleQuote);
                    in_quote = true;
                },
                '\"' => {
                    tokens.push(Token::DoubleQuote);
                    in_quote = true;
                },
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
        }
        i += 1;
    }
    tokens
}

fn parse(grammar : &String, tokens : Vec<Token>) -> Result<ast::Grammar, u8> {
    let mut iterator = tokens.iter().peekable();
    let mut grammar_object = ast::Grammar { rules: vec![], main: 0 };

    while let Some(token) = iterator.next() {
        if let &Token::Name(name_start, name_end) = token {
            if let Some(brace_token) = iterator.next() {
                if brace_token == &Token::OpenBrace {
                    let pattern = parse_pattern(grammar, &mut iterator, false);
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

fn parse_pattern<'a, Iter>(grammar : &String, iterator : &mut Peekable<Iter>, is_subpattern : bool) -> Result<ast::Pattern, u8>
    where Iter : Iterator<Item=&'a Token>
{
    let mut sequence = vec![];

    while let Some(token) = iterator.next() {
        let mut initial_pattern = Err(0);
        match token {
            &Token::OpenParen => { initial_pattern = initial_pattern.or(parse_pattern(grammar, iterator, true)); },
            &Token::OpenBracket => { initial_pattern = initial_pattern.or(parse_char_class(grammar, iterator)); },
            &Token::DoubleQuote => { initial_pattern = initial_pattern.or(parse_char_sequence(grammar, iterator, false)); },
            &Token::SingleQuote => { initial_pattern = initial_pattern.or(parse_char_sequence(grammar, iterator, true)); },
            &Token::Dot => { initial_pattern = initial_pattern.or(Ok(ast::Pattern::CharAny)); },
            &Token::Name(le, ri) => { initial_pattern = initial_pattern.or(parse_variable(grammar, le, ri)); },
            &Token::Ambersand => { initial_pattern = initial_pattern.or(parse_lookahead(grammar, iterator, true)); },
            &Token::Exclamation => { initial_pattern = initial_pattern.or(parse_lookahead(grammar, iterator, false)); },
            _ => { }
        }

        if initial_pattern.is_ok() {
            if let Some(&suffix_token) = iterator.peek() {
                match suffix_token {
                    &Token::Plus | &Token::Asterik | &Token::Question => {
                        initial_pattern = parse_suffix(grammar, iterator, initial_pattern.ok().unwrap());
                    }
                    _ => { }
                }
            }
        }

        match initial_pattern {
            Ok(p) => sequence.push(p),
            Err(x) => return Err(x)
        }

        if let Some(&test_token) = iterator.peek() {
            match test_token {
                &Token::CloseParen => if is_subpattern { 
                    iterator.next(); 
                } else { 
                    return Err(0);
                },
                &Token::CloseBrace => if !is_subpattern {
                    iterator.next();
                    break;
                } else {
                    return Err(0);
                },
                &Token::Slash => {
                    let sub_pattern = parse_choice(grammar, iterator, &mut sequence, is_subpattern);
                    match sub_pattern {
                        Ok(p) => sequence.push(p),
                        Err(x) => return Err(x)
                    }
                },
                _ => { }
            }
        }
    }

    let mut boxed_vec = vec![];
    while let Some(item) = sequence.pop() {
        boxed_vec.push(Box::new(item));
    }

    Ok(ast::Pattern::Sequence(boxed_vec))
}

fn parse_choice<'a, Iter>(grammar : &String, iterator : &mut Peekable<Iter>, patterns : &mut Vec<ast::Pattern>, is_subpattern : bool) -> Result<ast::Pattern, u8>
    where Iter : Iterator<Item=&'a Token>
{
    let mut boxed_vec = vec![];
    while let Some(item) = patterns.pop() {
        boxed_vec.push(Box::new(item));
    }

    iterator.next();

    let right = parse_pattern(grammar, iterator, is_subpattern);

    match right {
        Ok(p) => Ok(ast::Pattern::Choice(
            Box::new(ast::Pattern::Sequence(boxed_vec)),
            Box::new(p))),
        Err(x) => Err(x)
    }
}

fn parse_suffix<'a, Iter>(grammar : &String, iterator : &mut Peekable<Iter>, pattern : ast::Pattern) -> Result<ast::Pattern, u8>
    where Iter : Iterator<Item=&'a Token>
{
    if let Some(token) = iterator.next() {
        match token {
            &Token::Asterik => Ok(ast::Pattern::ZeroOrMore(Box::new(pattern))),
            &Token::Plus => Ok(ast::Pattern::OneOrMore(Box::new(pattern))),
            &Token::Question => Ok(ast::Pattern::Optional(Box::new(pattern))),
            _ => Err(0)
        }
    } else {
        Err(0)
    }
}

fn parse_lookahead<'a, Iter>(grammar : &String, iterator : &mut Peekable<Iter>, lookahead : bool) -> Result<ast::Pattern, u8>
    where Iter : Iterator<Item=&'a Token>
{
    let sub_pattern = parse_pattern(grammar, iterator, false);
    match sub_pattern {
        Ok(p) => Ok(ast::Pattern::Lookahead(lookahead, Box::new(p))),
        Err(x) => Err(x)
    }
}

fn parse_char_class<'a, Iter>(grammar : &String, iterator : &mut Peekable<Iter>) -> Result<ast::Pattern, u8>
    where Iter : Iterator<Item=&'a Token>
{
    Err(0)
}

fn parse_char_sequence<'a, Iter>(grammar : &String, iterator : &mut Peekable<Iter>, single_quote : bool) -> Result<ast::Pattern, u8>
    where Iter : Iterator<Item=&'a Token>
{

    Err(0)
}

fn parse_variable(grammar : &String, left : usize, right : usize) -> Result<ast::Pattern, u8>
{
    Err(0)
}




