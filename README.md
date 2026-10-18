<h1 align="center">sqlsplit</h1>
<br />
<div align="center">
  <!-- Version -->
  <a href="https://crates.io/crates/sqlsplit">
    <img src="https://img.shields.io/crates/v/sqlsplit.svg?style=flat-square"
    alt="Crates.io version" /></a>
  <!-- Docs -->
  <a href="https://docs.rs/sqlsplit">
    <img src="https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square" alt="docs.rs docs" /></a>
</div>

A simple [`nom`] parser library for splitting a string containing SQL into individual statements.

See the [documentation][docs] for more.

## Usage

Add the following to your Cargo.toml:

```toml
sqlsplit = "0.1.1"
```

`sqlsplit` provides a parser combinator `statement` that is used normally

```rust
use nom_sql::Parser as _;
use nom_sql::bytes::complete::statement;
use nom_sql::dialect::Postgres;

const SQL: &str = "SELECT 1; SELECT /* comment; block */ 1;";
// Tuple of (remainder after parse, parsed statement):
let (_rest, stat) = statement(&Postgres).parse(SQL).unwrap();
assert_eq!("SELECT 1;", stat);
```

and a parser iterator

```rust
use nom_sql::Statement;

Statement::postgres().iter_statements(SQL).for_each(|r| {
    println!("{}", r?);
});
```

See the [examples][eg] for more example use.

[`nom`]: https://docs.rs/nom/8.0.0/nom/
[docs]:  https://docs.rs/sqlsplit/latest/nom_sql/
[eg]: https://github.com/quasi-coherent/sqlsplit/tree/master/examples
