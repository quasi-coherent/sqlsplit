use nom_sql::bytes::complete::statement;
use nom_sql::dialect::Postgres;
use nom_sql::{Parser as _, Statement};

const SQL: &str = r#"
SELECT blah
FROM whatever
WHERE x = 'a;b;c;d';

BEGIN;
ALTER TABLE whatever ADD COLUMN y int;
INSERT INTO whatever (x, y) VALUES ('lebron;james', 43330);
COMMIT;

SELECT * FROM whatever WHERE x = 'lebron;james';
"#;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (r1, r2, r3) = check(combinator()?);
    println!("Got expected queries from `statement`: {r1}, {r2}, {r3}");

    let ss = statements_iter()?;
    let (r1, r2, r3) = check(ss);
    println!("Got expected queries from `statements_iter`: {r1}, {r2}, {r3}");
    Ok(())
}

/// Using the `Statements` parser iterator.
fn statements_iter<'a>() -> Result<Vec<&'a str>, Box<dyn std::error::Error>> {
    Statement::postgres().iter_statements(SQL).try_fold(
        Vec::new(),
        |mut acc, r| {
            let q = r?;
            acc.push(q);
            Ok(acc)
        },
    )
}

/// Using the parser combinator [`statement`] to incrementally parse a query.
fn combinator<'a>() -> Result<Vec<&'a str>, Box<dyn std::error::Error>> {
    let mut statements = Vec::new();
    let (rest, stat) = statement(&Postgres).parse(SQL)?;
    statements.push(stat.trim());

    let (rest, stat) = statement(&Postgres).parse(rest)?;
    statements.push(stat.trim());

    let (rest, stat) = statement(&Postgres).parse(rest)?;
    statements.push(stat.trim());

    // Remaining input should be empty:
    println!("Parsed all input: {}", rest.trim().is_empty());

    Ok(statements)
}

fn check(mut qs: Vec<&str>) -> (bool, bool, bool) {
    let q3 = qs.pop();
    let q2 = qs.pop();
    let q1 = qs.pop();

    let r1 = q1.is_some_and(|q| {
        q == "SELECT blah\nFROM whatever\nWHERE x = 'a;b;c;d';"
    });
    let r2 = q2.is_some_and(
        |q| q == "BEGIN;\nALTER TABLE whatever ADD COLUMN y int;\nINSERT INTO whatever (x, y) VALUES ('lebron;james', 43330);\nCOMMIT;"
    );
    let r3 = q3.is_some_and(|q| {
        q == "SELECT * FROM whatever WHERE x = 'lebron;james';"
    });
    (r1, r2, r3)
}
