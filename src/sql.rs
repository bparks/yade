use nom::IResult;
use nom::bytes::complete::tag;
use nom::character::complete::{alphanumeric1, space1};
use nom::sequence::tuple;

pub fn parse(s: &str) -> &str {
    let select_kw = tag::<&str, &str, (_, _)>("select");
    let star = tag("*");
    let from_kw = tag("from");
    let table_name = alphanumeric1;

    let mut select_stmt = tuple((select_kw, space1, star, space1, from_kw, space1, table_name));

    let (_, (_, _, _, _, _, _, table)) = select_stmt(s).unwrap();

    return table;
}