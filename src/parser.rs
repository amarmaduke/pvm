use std::collections::{HashMap};
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
    Colon,
    Number(i32),
    Name(i32),
    Letter(u8)
}

pub fn tokenize(grammar : &str) -> (Vec<Token>, i32, HashMap<Vec<u8>, i32>) {
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
                    _ => tokens.push(Token::Letter(item as u8)),
                }
            }
        } else {
            if !item.is_alphanumeric() && item != '_' && name.len() != 0 {
                if name.iter().all(|&x| x >= b'0' && x <= b'9') {
                    let mut i = 1;
                    let mut result = 0;
                    while let Some(x) = name.pop() {
                        let n = (x - b'0') as i32;
                        result += n*i;
                        i *= 10;
                    }
                    tokens.push(Token::Number(result));
                } else {
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
                ':' => tokens.push(Token::Colon),
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
    (tokens, i, map)
}

pub fn parse(tokens : Vec<Token>, rule_count : i32) -> Result<ast::Grammar, usize> {
    let mut grammar_object = ast::Grammar { rules: vec![], main: 0 };
    let mut insert_order = vec![];
    let mut i = 0;

    while let Some(token) = tokens.get(i) {
        if let &Token::Name(id) = token {
            i += 1;
            if let Some(brace_token) = tokens.get(i) {
                i += 1;
                if brace_token == &Token::OpenBrace {
                    let pattern = parse_expression(&mut i, &tokens);
                    match tokens.get(i) {
                        Some(&Token::CloseBrace) => i += 1,
                        _ => return Err(i)
                    }
                    match pattern {
                        Ok(p) => insert_order.push((id, p)),
                        Err(x) => return Err(x)
                    }
                } else {
                    return Err(i);
                }
            } else {
                return Err(i);
            } 
        } else {
            return Err(i);
        }
    }

    insert_order.sort_by(|a, b| a.0.cmp(&b.0));

    if insert_order.iter().map(|x| x.0).eq(0..rule_count) {
        grammar_object.rules = insert_order.drain(..).map(|x| x.1).collect();
        Ok(grammar_object)
    } else {
        Err(i)
    }
}

fn parse_expression(i : &mut usize, tokens : &Vec<Token>) -> Result<ast::Pattern, usize> {
    let first_sequence = match parse_sequence(i, tokens) {
        Ok(p) => p,
        Err(x) => return Err(x)
    };
    let mut patterns = vec![first_sequence];

    while tokens.get(*i) == Some(&Token::Slash) {
        *i += 1;
        let sequence = match parse_sequence(i, tokens) {
            Ok(p) => p,
            Err(x) => return Err(x)
        };
        patterns.push(sequence);
    }

    if patterns.len() >= 2 {
        let left = patterns.pop().unwrap();
        let right = patterns.pop().unwrap();
        let mut result = ast::Pattern::Choice(Box::new(right), Box::new(left));

        while let Some(remaining) = patterns.pop() {
            let temp = result;
            result = ast::Pattern::Choice(Box::new(remaining), Box::new(temp));
        }
        Ok(result)
    } else {
        Ok(patterns.pop().unwrap())
    }
}

fn parse_sequence(i : &mut usize, tokens : &Vec<Token>) -> Result<ast::Pattern, usize> {
    let first_prefix = match parse_prefix(i, tokens) {
        Ok(p) => p,
        Err(x) => return Err(x)
    };
    let mut patterns = vec![first_prefix];

    let mut backtrack = *i;
    loop {
        match parse_prefix(i, tokens) {
            Ok(p) => {
                patterns.push(p);
                backtrack = *i;
            },
            Err(_) => {
                *i = backtrack;
                break;
            }
        }
    }

    if patterns.len() > 1 {
        let boxed_patterns = patterns.drain(..).map(|x| Box::new(x)).collect();
        Ok(ast::Pattern::Sequence(boxed_patterns))
    } else {
        Ok(patterns.pop().unwrap())
    }
}

fn parse_prefix(i : &mut usize, tokens : &Vec<Token>) -> Result<ast::Pattern, usize> {
    let mut lookahead = None;

    if tokens.get(*i) == Some(&Token::Ambersand) {
        *i += 1;
        lookahead = Some(true);
    } else if tokens.get(*i) == Some(&Token::Exclamation) {
        *i += 1;
        lookahead = Some(false);
    }

    match parse_suffix(i, tokens) {
        Ok(p) => {
            match lookahead {
                Some(b) => Ok(ast::Pattern::Lookahead(b, Box::new(p))),
                None => Ok(p)
            }
        },
        Err(x) => Err(x)
    }
}

fn parse_suffix(i : &mut usize, tokens : &Vec<Token>) -> Result<ast::Pattern, usize> {
    match parse_primary(i, tokens) {
        Ok(p) => {
            if let Some(token) = tokens.get(*i) {
                match token {
                    &Token::Plus => {
                        *i += 1;
                        Ok(ast::Pattern::OneOrMore(Box::new(p)))
                    },
                    &Token::Asterik => {
                        *i += 1;
                        Ok(ast::Pattern::ZeroOrMore(Box::new(p)))
                    },
                    &Token::Question => {
                        *i += 1;
                        Ok(ast::Pattern::Optional(Box::new(p)))
                    },
                    _ => Ok(p)
                }
            } else { Ok(p) }
        },
        Err(x) => Err(x)
    }
}

fn parse_primary(i : &mut usize, tokens : &Vec<Token>) -> Result<ast::Pattern, usize> {
    let backtrack = *i;
    if let Some(token) = tokens.get(*i) {
        match token {
            &Token::Name(id) => {
                *i += 1;
                if tokens.get(*i) != Some(&Token::OpenBrace) {
                    if tokens.get(*i) == Some(&Token::Colon) {
                        *i += 1;
                        match tokens.get(*i) {
                            Some(&Token::Number(num)) => {
                                *i += 1;
                                Ok(ast::Pattern::Variable(id, num))
                            },
                            _ => Err(*i)
                        }
                    } else {
                        Ok(ast::Pattern::Variable(id, -1))
                    }
                } else {
                    *i = backtrack;
                    Err(*i)
                }
            },
            &Token::OpenParen => {
                *i += 1;
                match parse_expression(i, tokens) {
                    Ok(p) => {
                        if tokens.get(*i) == Some(&Token::CloseParen) {
                            *i += 1;
                            Ok(p)
                        } else {
                            Err(*i)
                        }
                    },
                    Err(x) => {
                        *i = backtrack;
                        Err(x)
                    }
                }
            },
            &Token::SingleQuote | &Token::DoubleQuote => {
                match parse_literal(i, tokens) {
                    Ok(p) => Ok(p),
                    Err(x) => {
                        *i = backtrack;
                        Err(x)
                    }
                }
            },
            &Token::OpenBracket => {
                match parse_class(i, tokens) {
                    Ok(p) => Ok(p),
                    Err(x) => {
                        *i = backtrack;
                        Err(x)
                    }
                }
            },
            &Token::Dot => {
                *i += 1;
                Ok(ast::Pattern::CharAny)
            },
            _ => Err(*i)
        }
    } else {
        Err(*i)
    }
}

fn parse_literal(i : &mut usize, tokens : &Vec<Token>) -> Result<ast::Pattern, usize> {
    let left_quote = tokens.get(*i);
    *i += 1;
    let mut letters = vec![];
    while let Some(token) = tokens.get(*i) {
        match token {
            &Token::Letter(x) => {
                *i += 1;
                letters.push(x);
            },
            _ => break
        }
    }
    let right_quote = tokens.get(*i);
    *i += 1;
    let quote_test = left_quote.is_some()
        && right_quote.is_some()
        && left_quote == right_quote
        && (*left_quote.unwrap() == Token::SingleQuote
            || *left_quote.unwrap() == Token::DoubleQuote);

    if quote_test {
        Ok(ast::Pattern::CharSequence(letters))
    } else {
        Err(*i)
    }
}

fn parse_class(i : &mut usize, tokens : &Vec<Token>) -> Result<ast::Pattern, usize> {
    match tokens.get(*i) {
        Some(&Token::OpenBracket) => *i += 1,
        _ => return Err(*i)
    }

    let mut ranges = vec![];
    loop {
        let left_letter = match tokens.get(*i) {
            Some(&Token::Letter(x)) => x,
            _ => break
        };
        *i += 1;
        let mut right_letter = None;
        
        if tokens.get(*i) == Some(&Token::Dash) {
            *i += 1;
            right_letter = match tokens.get(*i) {
                Some(&Token::Letter(x)) => Some(x),
                _ => return Err(*i)
            }
        }

        ranges.push((left_letter, right_letter));
    }

    match tokens.get(*i) {
        Some(&Token::CloseBracket) => *i += 1,
        _ => return Err(*i)
    }

    if ranges.len() > 0 {
        Ok(ast::Pattern::CharClass(ranges))
    } else {
        Err(*i)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dummy::Dummy;
    use machine;

    fn execute_test(grammar : &str, subjects : &Vec<&str>, expected : &Vec<bool>) {
        let machine_result = machine::Machine::new(grammar);
        assert!(machine_result.is_ok());
        assert!(subjects.len() == expected.len());
        let mut machine = machine_result.ok().unwrap();
        for i in 0..expected.len() {
            let result = machine.execute::<Dummy>(subjects[i].to_string().into_bytes());
            let fail = result.is_err();
            println!("machine result: {:?}", result);
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
            c { [a-\\\"c]? }
            apple { &.!(\" \") }
        ";

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
            char_class { [a-zA] }
            char_seq { \"abc\" }
        ";

        let subjects = vec!["azabc", "Bkabc", "AAabc", "aqd", "xyz"];
        let expected = vec![true, true, true, false, false];
        execute_test(&grammar, &subjects, &expected);
    }

    #[test]
    fn simple_suffix_grammar() {
        let grammar = "main { 'a'+ 'b'* 'c'? }";
        let subjects = vec!["ac", "a", "abb", "aaabbbc", "aaabbb", "bb", "c", "z"];
        let expected = vec![true, true, true, true, true, false, false, false];
        execute_test(&grammar, &subjects, &expected);
    }

    #[test]
    fn simple_choice_grammar() {
        let grammar = "main { 'a' / 'b' / 'c' }";
        let subjects = vec!["a", "b", "c", "abc", "z"];
        let expected = vec![true, true, true, false, false];
        execute_test(&grammar, &subjects, &expected);
    }

    #[test]
    fn simple_prefix_grammar() {
        let grammar = "main { 'a' &'b' 'b' 'c' !'d' }";
        let subjects = vec!["abc", "abc", "ac", "abcd"];
        let expected = vec![true, true, false, false];
        execute_test(&grammar, &subjects, &expected);
    }

    #[test]
    fn simple_parentheticals() {
        let grammar = "main { ('a' / 'b') / 'c' ('d'.)+ }";
        let subjects = vec!["a", "b", "cdx", "cdxdcdy", "x", "c"];
        let expected = vec![true, true, true, true, false, false];
        execute_test(&grammar, &subjects, &expected);
    }

    #[test]
    fn parenthetical_and_lookahead() {
        let grammar = "main { &('b' / 'a') .* }";
        let subjects = vec!["a", "b", "aa", "ab", "azzd", "c", "zab"];
        let expected = vec![true, true, true, true, true, false, false];
        execute_test(&grammar, &subjects, &expected);
    }

    #[test]
    fn direct_left_recursion() {
        let grammar = "main { (main:1 \"+n\" / 'n') ';' }";
        let subjects = vec!["n;", "n+n;", "n+n+n+n+n+n;", "n", "n+;", "+n;", "n+n", ";"];
        let expected = vec![true, true, true, false, false, false, false, false];
        execute_test(&grammar, &subjects, &expected);
    }

    #[test]
    fn indirect_left_recursion() {
        let grammar = "main { L } L { P:1 '.x' / 'x' } P { P:1 '(n)' / L:1 }";
        let subjects = vec!["x", "x.x", "x(n).x", "x(n)(n).x(n).x", "x.", "x(n)x", "(n)"];
        let expected = vec![true, true, true, true, false, false, false];
        execute_test(&grammar, &subjects, &expected);
    }

    #[test]
    fn dogfood() {
        let grammar = "
            main { grammar }
            grammar { s rule+ }
            rule { name '{' s expression '}' s }
            expression { sequence (slash sequence)* }
            sequence { prefix+ }
            prefix { (and / not)? suffix }
            suffix { primary (question / star / plus)? }
            primary { name (colon num)? !'{' / open expression close / literal / class / dot }

            name { [a-zA-Z][a-zA-Z0-9_]* s }
            num { [1-9][0-9]* s }

            literal { 
                    '\\'' (!'\\'' char)* '\\'' s
                /   '\\\"' (!'\\\"' char)* '\\\"' s
            }
            class { '[' (!']' range)* ']' s }
            range { char '-' char / char }
            char {
                    '\\\\' [trn'\"\\\\]
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
            colon { ':' s }

            s { ws* }
            ws { [ \\t\\r\\n] }
        ";
        let subjects = vec![
            "main { ('a' / 'b') / 'c' ('d'.)+ }",
            "main { 'a' &'b' 'b' 'c' !'d' }",
            "main { 'a' / 'b' / 'c' }",
            "main { 'a'+ 'b'* 'c'? }",
            "main { .char_class char_seq }
                char_class { [a-zA] }
                char_seq { \"abc\" }",
            &grammar
        ];
        let expected = vec![true, true, true, true, true, true];
        execute_test(&grammar, &subjects, &expected);
    }

}


