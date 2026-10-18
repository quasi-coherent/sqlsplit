//! Custom parsing directives to include.
use nom::branch::alt;
use nom::combinator::fail;
use nom::error::{Error, ParseError};
use nom::{IResult, Parser};

use crate::bytes::complete::{self, Quoted, quoted};

/// A `Dialect` of SQL for parsing rules specific to one variant.
pub trait Dialect<'a> {
    /// The type of value returned when parsing fails.
    type Error: ParseError<&'a str>;

    /// Specific parsing rules to apply for this dialect.
    fn parse(&self, i: &'a str) -> IResult<&'a str, &'a str, Self::Error>;
}

/// Do not add any additional parsing rules beyond the defaults.
#[derive(Clone, Copy, Debug, Default)]
pub struct Defaults;

impl<'a> Dialect<'a> for Defaults {
    type Error = Error<&'a str>;

    fn parse(&self, i: &'a str) -> IResult<&'a str, &'a str, Self::Error> {
        // `fail` always fails, which just means "go to the next branch."
        fail().parse(i)
    }
}

/// PostgreSQL parser rules.
#[derive(Clone, Copy, Debug, Default)]
pub struct Postgres;

impl<'a> Dialect<'a> for Postgres {
    type Error = Error<&'a str>;

    fn parse(&self, i: &'a str) -> IResult<&'a str, &'a str, Self::Error> {
        alt((complete::nested_block_comment, complete::dollar_quoted)).parse(i)
    }
}

/// MySQL parser rules.
#[derive(Clone, Copy, Debug, Default)]
pub struct MySql;

impl<'a> Dialect<'a> for MySql {
    type Error = Error<&'a str>;

    fn parse(&self, i: &'a str) -> IResult<&'a str, &'a str, Self::Error> {
        alt((
            complete::mysql_line_comment,
            quoted(Quoted::new('\'', '\'').can_escape()),
            quoted(Quoted::new('"', '"').can_escape()),
        ))
        .parse(i)
    }
}

/// SQLite parser rules.
#[derive(Clone, Copy, Debug, Default)]
pub struct Sqlite;

impl<'a> Dialect<'a> for Sqlite {
    type Error = Error<&'a str>;

    fn parse(&self, i: &'a str) -> IResult<&'a str, &'a str, Self::Error> {
        quoted(Quoted::new('`', '`')).parse(i)
    }
}
