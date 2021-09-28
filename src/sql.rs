use nom::IResult;
use nom::sequence::delimited;
use nom::character::complete::alphanumeric0;
use nom::character::complete::space0;
use nom::multi::many0;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{alphanumeric1, space1};
use nom::sequence::tuple;

pub enum Statement {
    Select(String),
    Update(String)
}

pub fn parse(s: &str) -> Statement {

    fn set_value_clause(s: &str) -> IResult<&str, (&str, &str)> {
        let value = alt((alphanumeric1, delimited(tag("'"), alphanumeric0, tag("'"))));
        let mut set_value_clause = tuple((alphanumeric1, space0, tag("="), space0, value));

        let (s, (column, _, _, _, val)) = set_value_clause(s)?;

        return Ok((s, (column, val)));
    }

    fn select_stmt(s: &str) -> IResult<&str, Statement> {
        let mut select = tuple((tag("select"), space1, tag("*"), space1, tag("from"), space1, alphanumeric1));
        let (s, (_, _, _, _, _, _, table)) = select(s)?;

        return Ok((s, Statement::Select(table.to_string())));
    }
    fn update_stmt(s: &str) -> IResult<&str, Statement> {
        let mut update = tuple((tag("update"), space1, alphanumeric1, space1, tag("set"), space1, set_value_clause, many0(tuple((space0, tag(","), space0, set_value_clause)))));
        let (s, (_, _, table, _, _, _, _, _)) = update(s)?;

        return Ok((s, Statement::Update(table.to_string())))
    }

    let mut parser = alt((select_stmt, update_stmt));

    let (_, stmt) = parser(s).unwrap();

    return stmt;
}