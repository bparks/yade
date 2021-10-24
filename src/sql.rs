/*use nom::Parser;
use nom::error::ParseError;*/
use core::fmt::Display;
use nom::error::{Error, ErrorKind};
use nom::Err;
use nom::combinator::opt;
use nom::IResult;
use nom::sequence::delimited;
use nom::character::complete::alphanumeric0;
use nom::character::complete::space0;
use nom::multi::many0;
use nom::branch::alt;
use nom::bytes::complete::tag;
use nom::character::complete::{alphanumeric1, space1};
use nom::sequence::tuple;

#[derive(Clone)]
pub struct Setter {
    pub column: Value, // TODO: How do we enforce that this is a column?
    pub value: Value
}

#[derive(Clone, Debug)]
pub enum Value {
    Column(String),
    StringLiteral(String),
    SystemValue(String),
    Asterisk(Option<String>)
}

pub enum Predicate {
    Equals(Value, Value),
    And(Box<Predicate>, Box<Predicate>),
    Or(Box<Predicate>, Box<Predicate>)
}

impl Display for Predicate {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Predicate::Equals(value1, value2) => write!(f, "{:?} = {:?}", value1, value2),
            Predicate::And(predicate1, predicate2) => write!(f, "{} and {}", predicate1, predicate2),
            Predicate::Or(predicate1, predicate2) => write!(f, "{} or {}", predicate1, predicate2)
        }
    }
}

pub enum Statement {
    Select(String),
    Update(String, Option<Vec<Setter>>, Option<Predicate>)
}

    fn parse_column_name(s: &str) -> IResult<&str, Value> {
        let (s, col_name) = alphanumeric1(s)?;
        return Ok((s, Value::Column(col_name.to_string())));
    }

    fn parse_string_literal(s: &str) -> IResult<&str, Value> {
        let (s, value) = delimited(tag("'"), alphanumeric0, tag("'"))(s)?;
        return Ok((s, Value::StringLiteral(value.to_string())));
    }

    fn parse_system_value(s: &str) -> IResult<&str, Value> {
        let (s, (_, var_name)) = tuple((tag("@@"), alphanumeric1))(s)?;
        return Ok((s, Value::SystemValue(var_name.to_string())));
    }

    fn parse_asterisk(s: &str) -> IResult<&str, Value> {
        let (s, _) = tag("*")(s)?;
        return Ok((s, Value::Asterisk(None)))
    }

    // TODO: Something is wrong with `parse_to`. See below
    /*fn parse_to<I, O, E: ParseError<I>, P>(mut parser: P, val: O) -> impl FnMut(I) -> IResult<I, O, E> where P: Parser<I, O, E>, O: Copy {
        return move |s| {
            let (s, _) = parser.parse(s)?;
            return Ok((s, val));
        }
    }*/

    fn parse_value(s: &str) -> IResult<&str, Value> {
        return delimited(space0, alt((
            parse_column_name,
            parse_string_literal,
            parse_system_value,
            // TODO: Something is wrong with `parse_to`. See above
            //parse_to(tag("*"), Value::Asterisk),
            parse_asterisk,
        )), space0)(s);
    }

    fn set_value_clause(s: &str) -> IResult<&str, Setter> {
        let mut set_value_clause = tuple((parse_column_name, space0, tag("="), space0, parse_value));

        let (s, (column, _, _, _, value)) = set_value_clause(s)?;

        return Ok((s, Setter { 
            column,
            value
        }));
    }

    fn where_clause(s: &str) -> IResult<&str, Predicate> {
        let mut where_clause_parser = tuple((tag("where"), parse_ors));
        let (s, (_, predicate)) = where_clause_parser(s)?;
        return Ok((s, predicate));
    }

    fn parse_ors(s: &str) -> IResult<&str, Predicate> {
        let mut ors = tuple((parse_ands, many0(tuple((tag("or"), parse_ands)))));
        let (s, (value1, addtl_values)) = ors(s)?;
        let predicate = addtl_values.into_iter().map(|(_, value)| value).fold(value1, |acc, val| Predicate::Or(Box::new(acc), Box::new(val)));
        return Ok((s, predicate));
    }

    fn parse_ands(s: &str) -> IResult<&str, Predicate> {
        let mut ands = tuple((parse_comparison, many0(tuple((tag("and"), parse_comparison)))));
        let (s, (pred1, addtl_preds)) = ands(s)?;
        let predicate = addtl_preds.into_iter().map(|(_, pred)| pred).fold(pred1, |acc, pred| Predicate::And(Box::new(acc), Box::new(pred)));
        return Ok((s, predicate));
    }

    fn parse_op(s: &str) -> IResult<&str, &str> {
        return alt((tag("="), tag("!="), tag("<>"), tag(">"), tag(">="), tag("<"), tag("<=")))(s);
    }

    fn parse_comparison(s: &str) -> IResult<&str, Predicate> {
        let (s, (value1, op, value2)) = tuple((parse_value, parse_op, parse_value))(s)?;
        return match op {
            "=" => Ok((s, Predicate::Equals(value1, value2))),
            _ => Err(Err::Error(Error{input: s, code: ErrorKind::NoneOf})) // TODO: Figure out how we're supposed to really error here
        }
    }

    fn parse_value_list(s: &str) -> IResult<&str, Vec<Value>> {
        let (s, (value1, addtl_values)) = tuple((parse_value, many0(tuple((tag(","), parse_value)))))(s)?;
        let mut value_list = Vec::new();
        value_list.push(value1);
        let _ = addtl_values.into_iter().map(|(_, valuen)| value_list.push(valuen));
        return Ok((s, value_list));
    }

    fn select_stmt(s: &str) -> IResult<&str, Statement> {
        let mut select = tuple((tag("select"), space1, parse_value_list, tag("from"), space1, alphanumeric1));
        let (s, (_, _, _values, _, _, table)) = select(s)?;

        return Ok((s, Statement::Select(table.to_string())));
    }
    fn update_stmt(s: &str) -> IResult<&str, Statement> {
        let mut update = tuple((
            tag("update"),
            space1,
            alphanumeric1,
            space1,
            tag("set"),
            space1,
            set_value_clause,
            many0(tuple((space0, tag(","), space0, set_value_clause))),
            opt(where_clause)
        ));
        let (s, (_, _, table, _, _, _, first_setter, additional_setters, where_val)) = update(s)?;

        let mut setters: Vec<Setter> = additional_setters.iter().map(|(_, _, _, setter)| setter.clone()).collect();
        setters.push(first_setter);

        return Ok((s, Statement::Update(table.to_string(), Some(setters), where_val)))
    }

pub fn parse(s: &str) -> Option<Statement> {

    let mut parser = alt((select_stmt, update_stmt));

    let result = parser(s);
    return match result {
        Ok((_, stmt)) => Some(stmt),
        Err(_) => None
    }
}