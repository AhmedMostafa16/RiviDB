use nom::{alphanumeric, digit, is_alphabetic, multispace};
use std::rc::Rc;
use std::str;
use std::str::FromStr;

use aggregator::Aggregator;
use expression::*;
use query_engine::*;
use value::*;

pub fn test() {
    let qstring = "select 1 where 2";
    let res = parse_query(qstring.as_bytes());
    println!("{:?}", res);
    println!("{:?}", res.unwrap().1);
}

named!(pub parse_query<&[u8], Query>, alt_complete!(full_query | simple_query));

named!(full_query<&[u8], Query>,
    do_parse!(
        tag_no_case!("select") >>
        multispace >>
        select: select_clauses >>
        multispace >>
        tag_no_case!("where") >>
        multispace >>
        filter: expr >>
        opt!(multispace) >>
        char!(';') >>
        (construct_query(select, filter))
    )
);

named!(simple_query<&[u8], Query>,
    do_parse!(
        tag_no_case!("select") >>
        multispace >>
        select: select_clauses >>
        opt!(multispace) >>
        opt!(char!(';')) >>
        (construct_query(select, Expr::Const(ValueType::Bool(true))))
    )
);

fn construct_query(select_clauses: Vec<AggregateOrSelect>, filter: Expr) -> Query {
    let (select, aggregate) = partition(select_clauses);
    Query {
        select: select,
        filter: filter,
        aggregate: aggregate,
    }
}

fn partition(select_or_aggregates: Vec<AggregateOrSelect>) -> (Vec<Expr>, Vec<(Aggregator, Expr)>) {
    let (selects, aggregates): (Vec<AggregateOrSelect>, Vec<AggregateOrSelect>) =
        select_or_aggregates.into_iter().partition(|x| match x {
            &AggregateOrSelect::Select(_) => true,
            _ => false,
        });

    (
        selects
            .into_iter()
            .filter_map(|x| match x {
                AggregateOrSelect::Select(expr) => Some(expr),
                _ => None,
            })
            .collect(),
        aggregates
            .into_iter()
            .filter_map(|x| match x {
                AggregateOrSelect::Aggregate(agg) => Some(agg),
                _ => None,
            })
            .collect(),
    )
}

named!(select_clauses<&[u8], Vec<AggregateOrSelect>>,
    separated_list!(
        tag!(","),
        alt_complete!(aggregate_clause | select_clause)
    )
);

named!(aggregate_clause<&[u8], AggregateOrSelect>,
    do_parse!(
        opt!(multispace) >>
        atype: aggregate_func >>
        char!('(') >>
        e: expr >>
        opt!(multispace) >>
        char!(')') >>
        (AggregateOrSelect::Aggregate((atype, e)))
    )
);

named!(select_clause<&[u8], AggregateOrSelect>, map!(expr, AggregateOrSelect::Select));

named!(aggregate_func<&[u8], Aggregator>, alt!(count | sum));

named!(count<&[u8], Aggregator>,
    map!( tag_no_case!("count"), |_| Aggregator::Count )
);

named!(sum<&[u8], Aggregator>,
    map!( tag_no_case!("sum"), |_| Aggregator::Sum )
);

named!(expr<&[u8], Expr>,
    do_parse!(
        opt!(multispace) >>
        result: alt!(function | colname | constant) >>
        (result)
    )
);

named!(function<&[u8], Expr>,
    do_parse!(
        ft: function_name >>
        char!('(') >>
        e1: expr >>
        char!(',') >>
        e2: expr >>
        char!(')') >>
        (Expr::func(ft, e1, e2))
    )
);

named!(constant<&[u8], Expr>,
    map!(
        alt!(integer |  string),
        Expr::Const
    )
);

named!(integer<&[u8], ValueType>,
    map!(
        map_res!(
            map_res!(
                digit,
                str::from_utf8
            ),
            FromStr::from_str
        ),
        |int| ValueType::Integer(int)
    )
);

named!(string<&[u8], ValueType>,
    do_parse!(
        char!('"') >>
        s: is_not!("\"") >>
        char!('"') >>
        (ValueType::Str(Rc::new(str::from_utf8(s).unwrap().to_string())))
    )
);

named!(colname<&[u8], Expr>,
    map!(
        identifier,
        |ident: &str| Expr::ColName(Rc::new(ident.to_string()))
    )
);

named!(function_name<&[u8], FuncType>,
    alt!( equals | and | greater | less )
);

named!(equals<&[u8], FuncType>,
    map!( tag!("="), |_| FuncType::Equals)
);

named!(greater<&[u8], FuncType>,
    map!( tag!(">"), |_| FuncType::GT)
);

named!(less<&[u8], FuncType>,
    map!( tag!("<"), |_| FuncType::LT)
);

named!(and<&[u8], FuncType>,
    map!( tag_no_case!("and"), |_| FuncType::And)
);

named!(identifier<&[u8], &str>,
    map_res!(
        take_while1!(is_alphabetic),
        str::from_utf8
    )
);

enum AggregateOrSelect {
    Aggregate((Aggregator, Expr)),
    Select(Expr),
}
