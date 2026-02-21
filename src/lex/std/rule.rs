use crate::error::{Error, ErrorKind};
use crate::iter::{Buffer, Consumer};
use crate::lex::pattern::{Class, Pattern, Repeat};
use crate::lex::{Rule, Rules};
use crate::store::Store;
use std::ops::Deref;

const fn is_whitespace(ch: char) -> bool {
    match ch {
        ' ' | '\t'..'\r' => true,
        _ => false,
    }
}

const fn is_ident(ch: char, i: usize) -> bool {
    ch == '_'
        || if i == 0 {
            ch.is_ascii_alphabetic()
        } else {
            ch.is_ascii_alphanumeric()
        }
}

fn parse_hex_digit(ch: char) -> Option<u8> {
    match ch {
        '0'..='9' => Some(ch as u8 - b'0'),
        'a'..='f' => Some(ch as u8 - b'a' + 10),
        'A'..='F' => Some(ch as u8 - b'A' + 10),
        _ => None,
    }
}

fn parse_char<'a, T: Consumer<'a, char>>(v: &mut T) -> Result<char, ErrorKind> {
    let Some(&ch) = v.next().as_deref() else {
        return Err(ErrorKind::NoChar);
    };
    if ch != '\\' {
        return Ok(ch);
    }

    let Some(&prefix) = v.next().as_deref() else {
        return Err(ErrorKind::NoEscapePrefix);
    };

    let ch = match prefix {
        't' => '\t',
        'n' => '\n',
        'v' => '\x0B',
        'f' => '\x0C',
        'r' => '\r',

        '\\' => '\\',
        '^' => '^',
        '-' => '-',
        '\'' => '\'',
        '[' => '[',
        ']' => ']',

        'x' => {
            let Some(Some(first)) = v.next().as_deref().copied().map(parse_hex_digit) else {
                return Err(ErrorKind::NoHexDigit);
            };

            let Some(Some(second)) = v.next().as_deref().copied().map(parse_hex_digit) else {
                return Err(ErrorKind::InsufficientHexDigit);
            };

            char::from_u32((first << 4 | second) as u32).ok_or(ErrorKind::InvalidUnicode)?
        }
        _ => return Err(ErrorKind::InvalidEscapePrefix),
    };

    Ok(ch)
}

fn parse_class<'a, T: Consumer<'a, char>>(v: &mut T) -> Result<Class<char>, ErrorKind> {
    let inclusive = v.next_if(|&ch| ch == '^').is_none();

    let mut wait_right_char = false;
    let mut class = Vec::new();
    loop {
        if let Some(&ch) = v.peek().as_deref() {
            match ch {
                ']' => {
                    if v.next().is_none() {
                        unreachable!("!v.peek().is_none()");
                    }

                    if wait_right_char {
                        return Err(ErrorKind::NoRightChar);
                    }

                    break;
                }
                '-' => {
                    if v.next().is_none() {
                        unreachable!("!v.peek().is_none()");
                    }

                    if wait_right_char {
                        return Err(ErrorKind::NoRightChar);
                    }

                    wait_right_char = true;
                }
                _ => {}
            }
        }

        let ch = parse_char(v)?;
        if wait_right_char {
            wait_right_char = false;

            let Some(left) = class.pop() else {
                return Err(ErrorKind::NoLeftChar);
            };

            class.reserve((ch as usize) - (left as usize));
            for ch in left..=ch {
                class.push(ch);
            }
        } else {
            class.push(ch);
        }
    }

    Ok(Class::new(class.into_iter().collect(), inclusive))
}

macro_rules! pop {
    ($name:ident) => {{
        let Some(old) = $name.pop() else {
            return Err(ErrorKind::NoLeftPattern);
        };

        old
    }};
}

macro_rules! repeat {
    ($old:ident, $range:expr) => {
        Pattern::Repeat(Repeat::new(pop!($old), $range))
    };
}

fn parse_body_once<'a, T: Consumer<'a, char>>(
    v: &mut T,
    stack: &mut Vec<Pattern<char>>,
    root: bool,
) -> Result<bool, ErrorKind> {
    match v.next().ok_or(ErrorKind::UnexpectedEof)?.deref() {
        '\'' => {
            let mut buffer = String::new();
            loop {
                if v.peek().as_deref().copied() == Some('\'') {
                    if v.next().is_none() {
                        unreachable!("!v.peek().is_none()");
                    }

                    break;
                }

                let ch = parse_char(v)?;
                buffer.push(ch);
            }

            Pattern::Literal(buffer.into())
        }
        '[' => Pattern::Class(parse_class(v)?),
        '(' => {
            let mut local = Vec::new();
            while parse_body_once(v, &mut local, false)? {}
            Pattern::Group(local)
        }
        '|' => {
            let old = pop!(stack);

            let mut local = Vec::new();
            while local.len() < 1 {
                if !parse_body_once(v, &mut local, false)? {
                    return Err(ErrorKind::MismatchedParen);
                }
            }
            let Some(single) = local.pop() else {
                unreachable!("local.len() >= 1");
            };

            Pattern::Or(Box::new(old), Box::new(single))
        }

        '?' => repeat!(stack, 0..2),
        '*' => repeat!(stack, 0..usize::MAX),
        '+' => repeat!(stack, 1..usize::MAX),

        '\t'..'\r' | ' ' => return Ok(true),

        ')' => {
            return if root {
                Err(ErrorKind::MismatchedParen)
            } else {
                Ok(false)
            };
        }
        ';' => {
            return if root {
                Ok(false)
            } else {
                Err(ErrorKind::UnexpectedSemicolon)
            };
        }

        &ch => return Err(ErrorKind::UnexpectedChar(ch)),
    };

    Ok(true)
}

fn parse_body<'a, T: Consumer<'a, char>>(v: &mut T) -> Result<Pattern<char>, ErrorKind> {
    let mut stack = Vec::new();
    while parse_body_once(v, &mut stack, true)? {}
    Ok(Pattern::Group(stack))
}

pub fn parse_rules<'a, T: Consumer<'a, char>>(
    v: &mut T,
    store: &mut Store<Buffer<'a, char>>,
) -> Result<Rules<'a, char>, Error> {
    let mut rules = Vec::new();
    loop {
        v.consume(|&ch, _| is_whitespace(ch));

        if v.eof() {
            break;
        }

        let ignore = v.next_if(|&ch| ch == '!').is_some();

        let ident = v.consume(|&ch, i| is_ident(ch, i));
        if ident.is_empty() {
            return Err(ErrorKind::NoIdent.at(v.at()));
        };

        v.consume(|&ch, _| is_whitespace(ch));

        if Some('=') != v.next().as_deref().copied() {
            return Err(ErrorKind::NoEq.at(v.at()));
        }

        let pattern = match parse_body(v) {
            Ok(pattern) => pattern,
            Err(e) => return Err(e.at(v.at())),
        };

        let Some(id) = store.add(ident) else {
            return Err(ErrorKind::ConflictName.at(v.at()));
        };

        rules.push(Rule::new(
            id,
            pattern,
            ignore,
        ));
    }

    Ok(Rules::new(rules))
}
