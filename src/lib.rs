//! # sqlsplit
//!
//! A simple [`nom`] parser library for splitting a string containing SQL into
//! individual statements.
//!
//! # Examples
//!
//! Choose a [`Dialect`] implementation and hand it to [`Statement`]:
//!
//! ```rust
//! use nom_sql::Statement;
//!
//! const SQL: &str = "SELECT 1; SELECT /* comment; block */ 1;";
//! let stats: Vec<_> = Statement::postgres()
//!     .iter_statements(SQL)
//!     .map(Result::unwrap)
//!     .collect();
//! assert_eq!(stats, vec!["SELECT 1;", "SELECT /* comment; block */ 1;"]);
//! ```
//!
//! [`nom`]: https://docs.rs/nom/8.0.0/nom/
//! [`Dialect`]: dialect::Dialect
pub mod bytes;
pub mod dialect;

mod parser;
pub use parser::{Statement, Statements};

#[doc(hidden)]
pub use nom::Parser;
