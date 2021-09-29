use nom::IResult;
use nom::sequence::delimited;
use nom::character::complete::alphanumeric0;
use nom::character::complete::space0;
use nom::multi::many0;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{alphanumeric1, space1};
use nom::sequence::tuple;

pub struct Setter {
    pub column: String,
    pub value: String
}

pub enum Statement {
    Select(String),
    Update(String, Option<Vec<Setter>>)
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
        let (s, (_, _, table, _, _, _, first_setter, additional_setters)) = update(s)?;

        let mut setters: Vec<&(&str, &str)> = additional_setters.iter().map(|(_, _, _, setter)| setter).collect();
        setters.push(&first_setter);

        return Ok((s, Statement::Update(table.to_string(), Some(setters.iter().map(|(col, val)| Setter {
            column: col.to_string(),
            value: val.to_string()
        }).collect()))))
    }

    let mut parser = alt((select_stmt, update_stmt));

    let (_, stmt) = parser(s).unwrap();

    return stmt;
}