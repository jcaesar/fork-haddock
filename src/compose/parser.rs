use nom::{
    branch::alt,
    bytes::complete::{tag, take_while1},
    character::complete::{anychar, char},
    combinator::{all_consuming, cut, eof, map, map_parser, value, verify},
    multi::{fold_many0, many_till},
    sequence::{delimited, preceded, tuple},
    IResult,
};
use parse_hyperlinks::take_until_unbalanced;

#[derive(Clone, Debug)]
pub(crate) enum Token {
    Str(String),
    Var(String, Option<Var>),
}

#[derive(Clone, Debug)]
pub(crate) enum Var {
    Default(State, Vec<Token>),
    Err(State, Vec<Token>),
}

#[derive(Clone, Debug)]
pub(crate) enum State {
    Unset,
    UnsetOrEmpty,
}

fn dollar(input: &str) -> IResult<&str, Token> {
    value(Token::Str('$'.to_string()), preceded(char('$'), char('$')))(input)
}

fn parameter(input: &str) -> IResult<&str, &str> {
    take_while1(|char: char| char.is_ascii_alphanumeric() || char == '_')(input)
}

fn variable(input: &str) -> IResult<&str, Token> {
    alt((variable_unexpanded, variable_expanded))(input)
}

fn variable_unexpanded(input: &str) -> IResult<&str, Token> {
    map(preceded(char('$'), parameter), |param: &str| {
        Token::Var(param.to_owned(), None)
    })(input)
}

fn variable_expanded(input: &str) -> IResult<&str, Token> {
    map_parser(
        preceded(
            char('$'),
            delimited(char('{'), take_until_unbalanced('{', '}'), char('}')),
        ),
        cut(alt((
            map(all_consuming(parameter), |param| {
                Token::Var(param.to_owned(), None)
            }),
            map(
                tuple((
                    parameter,
                    alt((tag(":-"), tag("-"), tag(":?"), tag("?"))),
                    string,
                )),
                |(param, separator, tokens)| {
                    Token::Var(
                        param.to_owned(),
                        match separator {
                            ":-" => Some(Var::Default(State::UnsetOrEmpty, tokens)),
                            "-" => Some(Var::Default(State::Unset, tokens)),
                            ":?" => Some(Var::Err(State::UnsetOrEmpty, tokens)),
                            "?" => Some(Var::Err(State::Unset, tokens)),
                            _ => None,
                        },
                    )
                },
            ),
        ))),
    )(input)
}

fn string(input: &str) -> IResult<&str, Vec<Token>> {
    all_consuming(fold_many0(
        verify(
            many_till(
                anychar,
                alt((map(alt((variable, dollar)), Some), value(None, eof))),
            ),
            |(chars, output)| output.is_some() || !chars.is_empty(),
        ),
        Vec::new,
        |mut tokens, token| {
            if !token.0.is_empty() {
                if let Some(Token::Str(string)) = tokens.last_mut() {
                    for char in token.0 {
                        string.push(char);
                    }
                } else {
                    let mut string = String::new();

                    for char in token.0 {
                        string.push(char);
                    }

                    tokens.push(Token::Str(string));
                }
            }

            if let Some(var) = token.1 {
                if let (Some(Token::Str(last)), Token::Str(string)) = (tokens.last_mut(), &var) {
                    *last = format!("{last}{string}");
                } else {
                    tokens.push(var);
                }
            }

            tokens
        },
    ))(input)
}

pub(crate) fn parse(input: &str) -> IResult<&str, Vec<Token>> {
    string(input)
}
