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
    Dash,
    Name(i32),
    Letter(u8)
}

pub fn tokenize(grammar : &String) -> (Vec<Token>, i32) {
    let mut iterator = grammar.chars();
    let mut tokens = vec![];
    let mut name = vec![];
    let mut i = 1;
    let mut in_quote = false;
    let mut in_bracket = false;
    let mut escaped = false;
    let main = vec![b'm', b'a', b'i', b'n'];
    let mut map = HashMap::new();
    map.insert(main, 0);

    while let Some(item) = iterator.next() {
        if in_quote || in_bracket {
            if escaped {
                match item {
                    't' => tokens.push(Token::Letter(b'\t')),
                    'r' => tokens.push(Token::Letter(b'\r')),
                    'n' => tokens.push(Token::Letter(b'\n')),
                    _ => tokens.push(Token::Letter(item as u8))
                }
                escaped = false;
            } else {
                match item {
                    '-' => {
                        if in_bracket {
                            tokens.push(Token::Dash);
                        } else {
                            tokens.push(Token::Letter('-' as u8));
                        }
                    },
                    '\'' if !in_bracket => {
                        tokens.push(Token::SingleQuote); 
                        in_quote = false;
                    },
                    '\"' if !in_bracket => {
                        tokens.push(Token::DoubleQuote); 
                        in_quote = false;
                    },
                    ']' if !in_quote => {
                        tokens.push(Token::CloseBracket);
                        in_bracket = false;
                    }
                    '\\' => { escaped = true; },
                    _ if in_quote => tokens.push(Token::Letter(item as u8)),
                    _ if in_bracket && !item.is_whitespace() => tokens.push(Token::Letter(item as u8)),
                    _ => { }
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
                '.' => tokens.push(Token::Dot),
                '+' => tokens.push(Token::Plus),
                '*' => tokens.push(Token::Asterik),
                '?' => tokens.push(Token::Question),
                '!' => tokens.push(Token::Exclamation),
                '&' => tokens.push(Token::Ambersand),
                '/' => tokens.push(Token::Slash),
                '-' => tokens.push(Token::Dash),
                '[' => {
                    tokens.push(Token::OpenBracket);
                    in_bracket = true;
                },
                ']' => {
                    tokens.push(Token::CloseBracket);
                    in_bracket = false;
                },
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
                    let pattern = parse_expression(grammar, &mut iterator);
                    insert_order.push((id, pattern));
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

fn parse_expression<'a, Iter>(grammar : &String, iterator : &mut Peekable<Iter>) -> ast::Pattern
    where Iter : Iterator<Item=&'a Token>
{
    let mut patterns = vec![parse_sequence(grammar, iterator)];

    while Some(&&Token::Slash) == iterator.peek() {
        iterator.next();
        patterns.push(parse_sequence(grammar, iterator));
    }

    if patterns.len() >= 2 {
        let left = patterns.pop().unwrap();
        let right = patterns.pop().unwrap();
        let mut result = ast::Pattern::Choice(Box::new(right), Box::new(left));

        while let Some(remaining) = patterns.pop() {
            let temp = result;
            result = ast::Pattern::Choice(Box::new(remaining), Box::new(temp));
        }
        result
    } else {
        patterns.pop().unwrap()
    }
}

fn parse_sequence<'a, Iter>(grammar : &String, iterator : &mut Peekable<Iter>) -> ast::Pattern
    where Iter : Iterator<Item=&'a Token>
{
    let mut patterns = vec![];

    while let Ok(p) = parse_prefix(grammar, iterator) {
        patterns.push(p);
    }

    let boxed_patterns = patterns.drain(..).map(|x| Box::new(x)).collect();
    ast::Pattern::Sequence(boxed_patterns)
}

fn parse_prefix<'a, Iter>(grammar : &String, iterator : &mut Peekable<Iter>) -> Result<ast::Pattern, u8>
    where Iter : Iterator<Item=&'a Token>
{
    let mut lookahead = None;

    if Some(&&Token::Ambersand) == iterator.peek() {
        lookahead = Some(true);
        iterator.next();
    } else if Some(&&Token::Exclamation) == iterator.peek() {
        lookahead = Some(false);
        iterator.next();
    }

    let suffix = parse_suffix(grammar, iterator);

    match suffix {
        Ok(p) => {
            match lookahead {
                Some(b) => Ok(ast::Pattern::Lookahead(b, Box::new(p))),
                None => Ok(p)
            }
        },
        Err(x) => Err(x)
    }
}

fn parse_suffix<'a, Iter>(grammar : &String, iterator : &mut Peekable<Iter>) -> Result<ast::Pattern, u8>
    where Iter : Iterator<Item=&'a Token>
{
    match parse_primary(grammar, iterator) {
        Ok(p) => {
            if let Some(&token) = iterator.peek() {
                match token {
                    &Token::Plus => Ok(ast::Pattern::OneOrMore(Box::new(p))),
                    &Token::Asterik => Ok(ast::Pattern::ZeroOrMore(Box::new(p))),
                    &Token::Question => Ok(ast::Pattern::Optional(Box::new(p))),
                    _ => Ok(p)
                }
            } else { Ok(p) }
        },
        Err(x) => Err(x)
    }
}

fn parse_primary<'a, Iter>(grammar : &String, iterator : &mut Peekable<Iter>) -> Result<ast::Pattern, u8>
    where Iter : Iterator<Item=&'a Token>
{
    if let Some(&token) = iterator.peek() {
        match token {
            &Token::Name(id) => {
                iterator.next();
                let test = iterator.peek();
                if Some(&&Token::OpenBrace) != test {
                    Ok(ast::Pattern::Variable(id))
                } else {
                    Err(0)
                }
            },
            &Token::OpenParen => {
                iterator.next();
                let expression = parse_expression(grammar, iterator);
                if Some(&Token::CloseParen) == iterator.next() {
                    Ok(expression)
                } else {
                    Err(0)
                }
            },
            &Token::SingleQuote | &Token::DoubleQuote => parse_literal(grammar, iterator),
            &Token::OpenBracket => parse_class(grammar, iterator),
            &Token::Dot => {
                iterator.next();
                Ok(ast::Pattern::CharAny)
            }
            _ => Err(0)
        }
    } else {
        Err(0)
    }
}

fn parse_literal<'a, Iter>(grammar : &String, iterator : &mut Peekable<Iter>) -> Result<ast::Pattern, u8>
    where Iter : Iterator<Item=&'a Token>
{
    let left_quote = iterator.next();
    let mut letters = vec![];
    while let Some(&token) = iterator.peek() {
        match token {
            &Token::Letter(x) => {
                iterator.next();
                letters.push(x)
            },
            _ => return Err(0)
        }
    }
    let right_quote = iterator.next();
    let test = left_quote.is_some() &&
        (*left_quote.unwrap() == Token::SingleQuote
        || *left_quote.unwrap() == Token::DoubleQuote);

    if left_quote == right_quote && test {
        
        Ok(ast::Pattern::CharSequence(letters))
    } else {
        Err(0)
    }
}

fn parse_class<'a, Iter>(grammar : &String, iterator : &mut Peekable<Iter>) -> Result<ast::Pattern, u8>
    where Iter : Iterator<Item=&'a Token>
{
    Err(0)
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
            c { [a-\\\" c]? }
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
            Token::Name(3), Token::OpenBrace, Token::OpenBracket, Token::Letter(b'a'),
                Token::Dash, Token::Letter(b'\"'), Token::Letter(b'c'), Token::CloseBracket,
                Token::Question, Token::CloseBrace,
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
            char_class { [a-z A] }
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
            grammar { s rule+ }
            rule { name '{' s expression '}' s }
            expression { sequence (slash sequence)* }
            sequence { prefix* }
            prefix { (and / not)? suffix }
            suffix { primary (question / star / plus)? }
            primary { name !'{' / open expression close / literal / class / dot }

            name { [a-z A-Z 0-9 _]+ s }

            literal { 
                    '\\'' (!'\\'' char)* '\\'' s
                /   '\\\"' (!'\\\"' char)* '\\\"' s
            }
            class { '[' (!']' range)* ']' s }
            range { char '-' char / char }
            char {
                    '\\\\' [t r n ' \"]
                /   !'\\\\' .
            }

            slash { '/' s }
            and { '&' s }
            not { '!' s }
            question { '?' s }
            star { '*' s }
            plus { '+' s }
            open { '(' s }
            close { ')' s }
            dot { '.' s }

            s { ws* }
            ws { [\\ \\t\\r\\n] }
        ".to_string();
        let subjects = vec![
            "main { ('a' / 'b') / 'c' ('d'.)+ }",
            "main { 'a' &'b' 'b' 'c' !'d' }",
            "main { 'a' / 'b' / 'c' }",
            "main { 'a'+ 'b'* 'c'? }",
            "main { .char_class char_seq }
                char_class { [a-z A] }
                char_seq { \"abc\" }",
            &grammar
        ];
        let expected = vec![true, true, true, true, true];
        execute_test(&grammar, &subjects, &expected);
    }

}


