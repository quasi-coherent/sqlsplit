use nom::error::Error;
use nom_sql::Statement;
use nom_sql::dialect::{Dialect, MySql, Postgres, Sqlite};

fn split<'a, D>(dialect: D, sql: &'a str) -> Vec<&'a str>
where
    D: Dialect<'a, Error = Error<&'a str>>,
{
    Statement::new(dialect).iter_statements(sql).map(Result::unwrap).collect()
}

#[test]
fn splits_on_semicolon() {
    let sql = "SELECT 1;\nSELECT 2";
    assert_eq!(split(Postgres, sql), ["SELECT 1;", "SELECT 2"]);
}

#[test]
fn skips_trailing_whitespace() {
    assert_eq!(split(Sqlite, "SELECT 1;\n\n  \n"), ["SELECT 1;"]);
}

#[test]
fn common_quotes_and_comments() {
    let sql = "SELECT 'a;b', \"c;d\" -- e;f\n/* g;h */;\nSELECT 2;";
    assert_eq!(
        split(Sqlite, sql),
        ["SELECT 'a;b', \"c;d\" -- e;f\n/* g;h */;", "SELECT 2;"]
    );
}

#[test]
fn postgres_nested_block_comment() {
    let sql = "/* a /* b; */ c; */ SELECT 1;\nSELECT 2;";
    assert_eq!(
        split(Postgres, sql),
        ["/* a /* b; */ c; */ SELECT 1;", "SELECT 2;"]
    );
}

#[test]
fn postgres_dollar_quoted() {
    let sql = "SELECT $fn$ a; $x$ b; $fn$;\nSELECT $$;$$;";
    assert_eq!(
        split(Postgres, sql),
        ["SELECT $fn$ a; $x$ b; $fn$;", "SELECT $$;$$;"]
    );
}

#[test]
fn postgres_positional_parameters() {
    let sql = "SELECT $1, $2;\nSELECT 2;";
    assert_eq!(split(Postgres, sql), ["SELECT $1, $2;", "SELECT 2;"]);
}

#[test]
fn postgres_backslash_does_not_escape() {
    let sql = "SELECT 'a\\';\nSELECT 'b';";
    assert_eq!(split(Postgres, sql), ["SELECT 'a\\';", "SELECT 'b';"]);
}

#[test]
fn mysql_escaped_quotes() {
    let sql = "SELECT 'a\\';b', \"c\\\";d\";\nSELECT 2;";
    assert_eq!(
        split(MySql, sql),
        ["SELECT 'a\\';b', \"c\\\";d\";", "SELECT 2;"]
    );
}

#[test]
fn mysql_hash_comment() {
    let sql = "SELECT 1 # a;b\n;\nSELECT 2;";
    assert_eq!(split(MySql, sql), ["SELECT 1 # a;b\n;", "SELECT 2;"]);
}

#[test]
fn sqlite_backtick_identifier() {
    let sql = "SELECT `a;b` FROM t;\nSELECT 2;";
    assert_eq!(split(Sqlite, sql), ["SELECT `a;b` FROM t;", "SELECT 2;"]);
}
