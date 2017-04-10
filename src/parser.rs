use std::collections::{HashMap};
use std::iter::{Peekable};
use std::str;
use ast;

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
    Name(i32),
    Letter(u8)
}

fn tokenize(grammar : &String) -> (Vec<Token>, i32) {
    let mut iterator = grammar.chars();
    let mut tokens = vec![];
    let mut name = vec![];
    let mut i = 1;
    let mut in_quote = false;
    let mut escaped = false;
    let mut map = HashMap::new();
    map.insert("main".to_string(), 0);

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
                _ if item.is_alphanumeric() => name.push(item as u8),
                _ if item.is_whitespace() && name.len() != 0 => {
                    let id = map.entry(String::from_utf8(name.clone()).unwrap()).or_insert(i);
                    tokens.push(Token::Name(*id));
                    name.clear();
                    i += 1;
                }
                _ => { }
            }
        }
    }
    (tokens, i)
}

fn parse(grammar : &String, tokens : Vec<Token>, rule_count : i32) -> Result<ast::Grammar, u8> {
    let mut iterator = tokens.iter().peekable();
    let mut grammar_object = ast::Grammar { rules: vec![], main: 0 };
    let mut insert_order = vec![];

    while let Some(token) = iterator.next() {
        if let &Token::Name(id) = token {
            if let Some(brace_token) = iterator.next() {
                if brace_token == &Token::OpenBrace {
                    let pattern = parse_pattern(grammar, &mut iterator, false);
                    match pattern {
                        Ok(p) => insert_order.push((id, p)),
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

    insert_order.sort_by(|a, b| a.0.cmp(&b.0));

    if insert_order.iter().map(|x| x.0).eq(0..rule_count) {
        grammar_object.rules = insert_order.drain(..).map(|x| x.1).collect();
        Ok(grammar_object)
    } else {
        Err(0)
    }
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
            &Token::Name(id) => { initial_pattern = initial_pattern.or(parse_variable(grammar, id)); },
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
    let mut tuples = vec![];

    while let Some(&token) = iterator.peek() {
        if token == &Token::CloseBracket { 
            iterator.next();
            break;
        }
        let letter = parse_char_class_element(grammar, iterator);

        if let Some(&dot_token) = iterator.peek() {
            if dot_token == &Token::Dot {
                let range_letter = parse_char_class_element(grammar, iterator);
                if letter.is_ok() && range_letter.is_ok() {
                    tuples.push((letter.ok().unwrap(), Some(range_letter.ok().unwrap())));
                } else {
                    return Err(0);
                }
            }
        }

        match letter {
            Ok(x) => tuples.push((x, None)),
            Err(x) => return Err(x)
        };
    }

    Ok(ast::Pattern::CharClass(tuples))
}

fn parse_char_class_element<'a, Iter>(grammar : &String, iterator : &mut Peekable<Iter>) -> Result<u8, u8>
    where Iter : Iterator<Item=&'a Token>
{
    let first_quote = iterator.next();
    let utf8_byte = iterator.next();
    let second_quote = iterator.next();

    if first_quote.is_some() && utf8_byte.is_some() && second_quote.is_some() {
        let single_quote_check = *first_quote.unwrap() == Token::SingleQuote
            && first_quote.unwrap() == second_quote.unwrap();
        let double_quote_check = *first_quote.unwrap() == Token::DoubleQuote
            && first_quote.unwrap() == second_quote.unwrap();

        if single_quote_check || double_quote_check {
            match utf8_byte.unwrap() {
                &Token::Letter(x) => Ok(x),
                _ => Err(0)
            }
        } else {
            Err(0)
        }
    } else {
        Err(0)
    }
}

fn parse_char_sequence<'a, Iter>(grammar : &String, iterator : &mut Peekable<Iter>, is_single_quote : bool) -> Result<ast::Pattern, u8>
    where Iter : Iterator<Item=&'a Token>
{
    let mut characters = vec![];
    while let Some(item) = iterator.next() {
        match item {
            &Token::Letter(x) => { characters.push(x); },
            &Token::SingleQuote if is_single_quote => { break; },
            &Token::DoubleQuote if !is_single_quote => { break; },
            _ => { return Err(0); }
        }
    }
    Ok(ast::Pattern::CharSequence(characters))
}

fn parse_variable(grammar : &String, id : i32) -> Result<ast::Pattern, u8>
{
    Ok(ast::Pattern::Variable(id))
}




