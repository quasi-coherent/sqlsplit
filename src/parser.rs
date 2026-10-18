//! Statement parsers and the supported SQL dialects.
use nom::{OutputMode, PResult, Parser};

use crate::bytes::complete;
use crate::dialect::{Defaults, Dialect, MySql, Postgres, Sqlite};

/// Parser for one statement, including its terminating `;` when present.
#[derive(Clone, Copy, Debug)]
pub struct Statement<D = Defaults> {
    dialect: D,
}

impl Default for Statement {
    fn default() -> Self {
        Self { dialect: Defaults }
    }
}

impl<D> Statement<D> {
    /// New statement parser for `dialect`.
    pub fn new(dialect: D) -> Self {
        Self { dialect }
    }

    /// Return an iterator over the parsed statements.
    pub fn iter_statements<'a>(self, input: &'a str) -> Statements<'a, D> {
        Statements::new(self, input)
    }
}

impl Statement<Postgres> {
    /// New PostgreSQL statement parser.
    pub fn postgres() -> Self {
        Self::new(Postgres)
    }
}

impl Statement<Sqlite> {
    /// New SQLite statement parser.
    pub fn sqlite() -> Self {
        Self::new(Sqlite)
    }
}

impl Statement<MySql> {
    /// New MySQL statement parser.
    pub fn mysql() -> Self {
        Self::new(MySql)
    }
}

impl<'a, D: Dialect<'a>> Parser<&'a str> for Statement<D> {
    type Error = D::Error;
    type Output = &'a str;

    fn process<OM: OutputMode>(
        &mut self,
        i: &'a str,
    ) -> PResult<OM, &'a str, &'a str, D::Error> {
        let dialect = &self.dialect;
        complete::statement(dialect).process::<OM>(i)
    }
}

/// Iterator over the statements in an input, skipping whitespace between them.
#[derive(Clone, Debug)]
pub struct Statements<'a, D> {
    statement: Statement<D>,
    input: &'a str,
}

impl<'a, D> Statements<'a, D> {
    fn new(statement: Statement<D>, input: &'a str) -> Self {
        Self { statement, input }
    }
}

impl<'a, D: Dialect<'a>> Iterator for Statements<'a, D> {
    type Item = Result<&'a str, nom::Err<D::Error>>;

    fn next(&mut self) -> Option<Self::Item> {
        self.input = self.input.trim_start();
        if self.input.is_empty() {
            return None;
        }
        match self.statement.parse(self.input) {
            Ok((rest, stmt)) => {
                self.input = rest;
                Some(Ok(stmt))
            },
            Err(e) => {
                self.input = "";
                Some(Err(e))
            },
        }
    }
}
