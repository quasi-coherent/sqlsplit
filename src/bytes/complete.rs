//! Parsers recognizing complete byte streams.
use nom::branch::alt;
use nom::bytes::complete::{
    tag, take_till, take_till1, take_while, take_while1,
};
use nom::bytes::{tag_no_case, take_until};
use nom::character::complete::{anychar, char as chr};
use nom::character::satisfy;
use nom::combinator::{not, recognize};
use nom::error::ParseError;
use nom::sequence::{delimited, terminated};
use nom::{IResult, Parser};

use crate::dialect::Dialect;

/// Parse one statement through its terminating `;`, or to the end of input.
///
/// The `dialect` rules are tried first, so they take precedence over the
/// common rules they overlap with.
pub fn statement<'a, D, E>(d: &D, i: &'a str) -> IResult<&'a str, &'a str, E>
where
    D: Dialect<'a, Error = E>,
    E: ParseError<&'a str>,
{
    let base = alt((
        |i: &'a str| dialect(i, d),
        |i: &'a str| line_comment(i),
        |i: &'a str| block_comment(i),
        quoted(Quoted::new('\'', '\'')),
        quoted(Quoted::new('\"', '\"')),
        word,
        recognize(anychar),
    ));
    through(i, base, chr(';'))
}

// Custom `Dialect` parser.
fn dialect<'a, D, E>(
    input: &'a str,
    dialect: &D,
) -> IResult<&'a str, &'a str, E>
where
    D: Dialect<'a, Error = E>,
    E: ParseError<&'a str>,
{
    let (rest, out) = dialect.parse(input)?;
    Ok((rest, out))
}

/// Apply the parser `element` repeatedly until the parser `terminator` matches,
/// returning the complete input consumed up to and through `terminator`.
pub fn through<'a, P, Q, E: ParseError<&'a str>>(
    input: &'a str,
    mut element: P,
    mut terminator: Q,
) -> IResult<&'a str, &'a str, E>
where
    P: Parser<&'a str, Error = E>,
    Q: Parser<&'a str, Error = E>,
{
    let mut rest = input;
    loop {
        if let Ok((after, _)) = terminator.parse(rest) {
            rest = after;
            break;
        }
        if rest.is_empty() {
            break;
        }
        let (after, _) = element.parse(rest)?;
        rest = after;
    }
    Ok((rest, recognized(input, rest)))
}

/// Parses a SQL single-line comment: from the first occurrence of "--" to the
/// end of the line.
pub fn line_comment<'a, E: ParseError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, &'a str, E> {
    recognize((tag("--"), take_till(|c| c == '\n'))).parse(i)
}

/// Parses a MySQL single-line comment: from the first occurrence of "--" or "#"
/// to the end of the line.
pub fn mysql_line_comment<'a, E: ParseError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, &'a str, E> {
    let opened = alt((tag("--"), tag("#")));
    recognize((opened, take_till(|c| c == '\n'))).parse(i)
}

/// Parse `/* ... */`-style comments.
///
/// This does not handle block comments that contain nested block comments.  For
/// this scenario, use [`nested_block_comment`].
pub fn block_comment<'a, E: ParseError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, &'a str, E> {
    let (rest, _) = tag("/*").parse(i)?;
    let plain = take_till1(|c| c == '*' || c == '/');
    let (rest, _) = through(rest, alt((plain, recognize(anychar))), tag("*/"))?;
    Ok((rest, recognized(i, rest)))
}

/// Parse `/* ... */`-style comments that can contain nested block comments.
///
/// Only one level of nesting is supported.
pub fn nested_block_comment<'a, E: ParseError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, &'a str, E> {
    let (rest, _) = tag("/*").parse(i)?;
    let plain = take_till1(|c| c == '*' || c == '/');
    let (rest, _) = through(
        rest,
        alt((plain, block_comment, recognize(anychar))),
        tag("*/"),
    )?;
    Ok((rest, recognized(i, rest)))
}

/// Characters that quote a region in a SQL dialect.
#[derive(Clone, Copy, Debug)]
pub struct Quoted {
    open: char,
    close: char,
    escapable: bool,
}

impl Quoted {
    /// New quoting character set.
    pub fn new(open: char, close: char) -> Self {
        Self { open, close, escapable: false }
    }

    /// Whether a preceding `\\` can escape the quote character.
    pub fn can_escape(self) -> Self {
        Self { escapable: true, ..self }
    }
}

/// Parse a quoted region.
pub fn quoted<'a, E: ParseError<&'a str>>(
    chars: Quoted,
) -> impl FnMut(&'a str) -> IResult<&'a str, &'a str, E> {
    move |i: &str| {
        let (rest, _) = chr(chars.open).parse(i)?;
        let plain = take_till1(move |c| {
            c == chars.close || (chars.escapable && c == '\\')
        });
        let element =
            alt((plain, recognize((chr('\\'), anychar)), recognize(anychar)));
        let (rest, _) = through(rest, element, chr(chars.close))?;
        Ok((rest, recognized(i, rest)))
    }
}

/// Parse a `$tag$`-quoted region in PostgreSQL.
pub fn dollar_quoted<'a, E: ParseError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, &'a str, E> {
    let (rest, ident) =
        delimited(chr('$'), take_while(is_tag_char), chr('$')).parse(i)?;
    let close = format!("${ident}$");
    let (rest, _) =
        terminated(take_until(&*close), tag(&*close)).parse(rest)?;
    Ok((rest, recognized(i, rest)))
}

/// Parse a `BEGIN ... [END/COMMIT/ROLLBACK]` block.
pub fn begin_end<'a, E: ParseError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, &'a str, E> {
    let (rest, _) = keyword("begin").parse(i)?;
    let (rest, _) = terminated(
        take_while1(is_any_text_char),
        alt((tag("end"), tag("commit"), tag("rollback"))),
    )
    .parse(rest)?;
    Ok((rest, recognized(i, rest)))
}

/// Parse a keyword `kw`.
pub fn keyword<'a, E: ParseError<&'a str>>(
    kw: &'a str,
) -> impl FnMut(&'a str) -> IResult<&'a str, &'a str, E> {
    move |i: &str| {
        terminated(tag_no_case(kw), not(satisfy(is_word_char))).parse(i)
    }
}

fn word<'a, E: ParseError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, &'a str, E> {
    take_while1(is_word_char).parse(i)
}

fn is_word_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_' || c == '$'
}

fn is_tag_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

fn is_any_text_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c.is_ascii_whitespace()
}

// The prefix of `input` consumed in reaching `rest`.
fn recognized<'a>(input: &'a str, rest: &'a str) -> &'a str {
    &input[..input.len() - rest.len()]
}
