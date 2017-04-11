use std::collections::{HashMap};
use std::iter::{Peekable};
use ast;

#[derive(Debug, Eq, PartialEq)]
pub enum Token {
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

pub fn tokenize(grammar : &String) -> (Vec<Token>, i32) {
    let mut iterator = grammar.chars();
    let mut tokens = vec![];
    let mut name = vec![];
    let mut i = 1;
    let mut in_quote = false;
    let mut escaped = false;
    let main = vec![b'm', b'a', b'i', b'n'];
    let mut map = HashMap::new();
    map.insert(main, 0);

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
            if !item.is_alphanumeric() && item != '_' && name.len() != 0 {
                let mut to_insert = false;
                let id = match map.get(&name) {
                    Some(j) => *j,
                    None => {
                        let j = i;
                        i += 1;
                        to_insert = true;
                        j
                    }
                };
                if to_insert { map.insert(name.clone(), id); }
                tokens.push(Token::Name(id));
                name.clear();
            }

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
                _ if item.is_alphanumeric() || item == '_' => name.push(item as u8),
                _ => { }
            }
        }
    }
    (tokens, i)
}

pub fn parse(grammar : &String, tokens : Vec<Token>, rule_count : i32) -> Result<ast::Grammar, u8> {
    let mut iterator = tokens.iter().peekable();
    let mut grammar_object = ast::Grammar { rules: vec![], main: 0 };
    let mut insert_order = vec![];

    while let Some(token) = iterator.next() {
        if let &Token::Name(id) = token {
            if let Some(brace_token) = iterator.next() {
                if brace_token == &Token::OpenBrace {
                    let pattern = parse_pattern(grammar, &mut iterator, false, true, false);
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

fn parse_pattern<'a, Iter>(grammar : &String, iterator : &mut Peekable<Iter>, is_subpattern : bool, greedy : bool, consume_paren : bool) -> Result<ast::Pattern, u8>
    where Iter : Iterator<Item=&'a Token>
{
    let mut sequence = vec![];

    while let Some(token) = iterator.next() {
        println!("{:?} {:?}", token, sequence);
        let mut initial_pattern = Err(0);
        match token {
            &Token::OpenParen => { initial_pattern = initial_pattern.or(parse_pattern(grammar, iterator, true, true, true)); },
            &Token::OpenBracket => { initial_pattern = initial_pattern.or(parse_char_class(iterator)); },
            &Token::DoubleQuote => { initial_pattern = initial_pattern.or(parse_char_sequence(iterator, false)); },
            &Token::SingleQuote => { initial_pattern = initial_pattern.or(parse_char_sequence(iterator, true)); },
            &Token::Dot => { initial_pattern = initial_pattern.or(Ok(ast::Pattern::CharAny)); },
            &Token::Name(id) => { initial_pattern = initial_pattern.or(parse_variable(id)); },
            &Token::Ambersand => { initial_pattern = initial_pattern.or(parse_lookahead(grammar, iterator, true, is_subpattern)); },
            &Token::Exclamation => { initial_pattern = initial_pattern.or(parse_lookahead(grammar, iterator, false, is_subpattern)); },
            &Token::CloseParen => { break; }
            _ => { }
        }

        if initial_pattern.is_ok() {
            if let Some(&suffix_token) = iterator.peek() {
                match suffix_token {
                    &Token::Plus | &Token::Asterik | &Token::Question => {
                        initial_pattern = parse_suffix(iterator, initial_pattern.ok().unwrap());
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
                    if consume_paren { iterator.next(); }
                    break;
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

        if !greedy && sequence.len() > 0 { break; }
    }

    let boxed_vec = sequence.drain(..).map(|x| Box::new(x)).collect();
    Ok(ast::Pattern::Sequence(boxed_vec))
}

fn parse_choice<'a, Iter>(grammar : &String, iterator : &mut Peekable<Iter>, patterns : &mut Vec<ast::Pattern>, is_subpattern : bool) -> Result<ast::Pattern, u8>
    where Iter : Iterator<Item=&'a Token>
{
    let boxed_vec = patterns.drain(..).map(|x| Box::new(x)).collect();
    iterator.next();

    let right = parse_pattern(grammar, iterator, is_subpattern, true, false);
    match right {
        Ok(p) => Ok(ast::Pattern::Choice(
            Box::new(ast::Pattern::Sequence(boxed_vec)),
            Box::new(p))),
        Err(x) => Err(x)
    }
}

fn parse_suffix<'a, Iter>(iterator : &mut Peekable<Iter>, pattern : ast::Pattern) -> Result<ast::Pattern, u8>
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

fn parse_lookahead<'a, Iter>(grammar : &String, iterator : &mut Peekable<Iter>, lookahead : bool, is_subpattern : bool) -> Result<ast::Pattern, u8>
    where Iter : Iterator<Item=&'a Token>
{
    let sub_pattern = parse_pattern(grammar, iterator, is_subpattern, false, false);
    match sub_pattern {
        Ok(p) => Ok(ast::Pattern::Lookahead(lookahead, Box::new(p))),
        Err(x) => Err(x)
    }
}

fn parse_char_class<'a, Iter>(iterator : &mut Peekable<Iter>) -> Result<ast::Pattern, u8>
    where Iter : Iterator<Item=&'a Token>
{
    let mut tuples = vec![];

    while let Some(&token) = iterator.peek() {
        if token == &Token::CloseBracket { 
            iterator.next();
            break;
        }
        let letter = parse_char_class_element(iterator);

        if let Some(&first_dot) = iterator.peek() {
            if first_dot == &Token::Dot {
                iterator.next();
                if let Some(&second_dot) = iterator.peek() {
                    if second_dot == &Token::Dot {
                        iterator.next();
                        let range_letter = parse_char_class_element(iterator);
                        if letter.is_ok() && range_letter.is_ok() {
                            tuples.push((letter.ok().unwrap(), Some(range_letter.ok().unwrap())));
                        } else {
                            return Err(0);
                        }
                    }
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

fn parse_char_class_element<'a, Iter>(iterator : &mut Peekable<Iter>) -> Result<u8, u8>
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

fn parse_char_sequence<'a, Iter>(iterator : &mut Peekable<Iter>, is_single_quote : bool) -> Result<ast::Pattern, u8>
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

fn parse_variable(id : i32) -> Result<ast::Pattern, u8>
{
    Ok(ast::Pattern::Variable(id))
}

#[cfg(test)]
mod tests {
    use super::*;
    use machine;

    fn execute_test(grammar : &String, subjects : &Vec<&str>, expected : &Vec<bool>) {
        let token_result = tokenize(grammar);
        println!("{:?}", token_result.0);
        let parse_result = parse(grammar, token_result.0, token_result.1);
        assert!(parse_result.is_ok());
        let grammar_object = parse_result.ok().unwrap();

        let program = grammar_object.compile();
        let mut machine = machine::Machine::new(program);
        assert!(subjects.len() == expected.len());
        for i in 0..expected.len() {
            let result = machine.execute(subjects[i].to_string().into_bytes());
            let fail = result.is_err();
            println!("{:?}", machine.program);
            println!("{}", subjects[i]);
            assert!(!fail == expected[i]);
        }
    }

    #[test]
    fn tokenizer() {
        let grammar = "
            main { a b c / b c a / c b a }
            a { apple+ }
            b { \"bu\"* }
            c { ['a'..'\\\"''c']? }
            apple { &.!(\" \") }
        ".to_string();

        let tokens = tokenize(&grammar).0;
        println!("{:?}", tokens);
        let expected = vec![
            Token::Name(0), Token::OpenBrace, Token::Name(1), Token::Name(2), Token::Name(3),
                Token::Slash, Token::Name(2), Token::Name(3), Token::Name(1), Token::Slash,
                Token::Name(3), Token::Name(2), Token::Name(1), Token::CloseBrace,
            Token::Name(1), Token::OpenBrace, Token::Name(4), Token::Plus, Token::CloseBrace,
            Token::Name(2), Token::OpenBrace, Token::DoubleQuote, Token::Letter(b'b'),
                Token::Letter(b'u'), Token::DoubleQuote, Token::Asterik, Token::CloseBrace,
            Token::Name(3), Token::OpenBrace, Token::OpenBracket, Token::SingleQuote,
                Token::Letter(b'a'), Token::SingleQuote, Token::Dot, Token::Dot, Token::SingleQuote,
                Token::Letter(b'\"'), Token::SingleQuote, Token::SingleQuote, Token::Letter(b'c'),
                Token::SingleQuote, Token::CloseBracket, Token::Question, Token::CloseBrace,
            Token::Name(4), Token::OpenBrace, Token::Ambersand, Token::Dot, Token::Exclamation,
                Token::OpenParen, Token::DoubleQuote, Token::Letter(b' '), Token::DoubleQuote,
                Token::CloseParen, Token::CloseBrace
        ];
        assert!(tokens.iter().eq(expected.iter()));
    }

    #[test]
    fn simple_char_grammar() {
        let grammar = "
            main { .char_class char_seq }
            char_class { ['a'..'z''A'] }
            char_seq { \"abc\" }
        ".to_string();

        let subjects = vec!["azabc", "Bkabc", "AAabc", "aqd", "xyz"];
        let expected = vec![true, true, true, false, false];
        execute_test(&grammar, &subjects, &expected);
    }

    #[test]
    fn simple_suffix_grammar() {
        let grammar = "main { 'a'+ 'b'* 'c'? }".to_string();
        let subjects = vec!["ac", "a", "abb", "aaabbbc", "aaabbb", "bb", "c", "z"];
        let expected = vec![true, true, true, true, true, false, false, false];
        execute_test(&grammar, &subjects, &expected);
    }

    #[test]
    fn simple_choice_grammar() {
        let grammar = "main { 'a' / 'b' / 'c' }".to_string();
        let subjects = vec!["a", "b", "c", "abc", "z"];
        let expected = vec![true, true, true, false, false];
        execute_test(&grammar, &subjects, &expected);
    }

    #[test]
    fn simple_prefix_grammar() {
        let grammar = "main { 'a' &'b' 'b' 'c' !'d' }".to_string();
        let subjects = vec!["abc", "abc", "ac", "abcd"];
        let expected = vec![true, true, false, false];
        execute_test(&grammar, &subjects, &expected);
    }

    #[test]
    fn simple_parentheticals() {
        let grammar = "main { ('a' / 'b') / 'c' ('d'.)+ }".to_string();
        let subjects = vec!["a", "b", "cdx", "cdxdcdy", "x", "c"];
        let expected = vec![true, true, true, true, false, false];
        execute_test(&grammar, &subjects, &expected);
    }

    #[test]
    fn parenthetical_and_lookahead() {
        let grammar = "main { &('b' / 'a') .* }".to_string();
        let subjects = vec!["a", "b", "aa", "ab", "azzd", "c", "zab"];
        let expected = vec![true, true, true, true, true, false, false];
        execute_test(&grammar, &subjects, &expected);
    }

    #[test]
    fn dogfood() {
        let grammar = "
            main { rule+ }
            rule { name '{' expression '}' }

            expression { sequence ('/' sequence)* }
            sequence { prefix* }
            prefix { ('&' / '!')? suffix }
            suffix { primary ('?' / '*' / '+')? }
            primary { name / '(' expression ')' / qletter / char_class / char_seq / any }

            char_class { '[' (qletter / qletter \"..\" qletter)+ ']' }
            char_seq { '\\\"' letter+ '\\\"' }
            any { '.' }

            qletter { ''' letter ''' / '\\\"' letter '\\\"' }
            name { letter+ }
            letter { ['a'..'z''A'..'Z''0'..'9''_'] }
        ".to_string();
        let subjects = vec![
            "main { ('a' / 'b') / 'c' ('d'.)+ }",
            "main { 'a' &'b' 'b' 'c' !'d' }",
            "main { 'a' / 'b' / 'c' }",
            "main { 'a'+ 'b'* 'c'? }",
            "main { .char_class char_seq }
                char_class { ['a'..'z''A'] }
                char_seq { \"abc\" }"
        ];
        let expected = vec![true, true, true, true, true];
        execute_test(&grammar, &subjects, &expected);
    }

}


