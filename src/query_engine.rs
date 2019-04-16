use std::iter::Iterator;
use std::rc::Rc;
use value::ValueType;

#[derive(Debug)]
pub enum FuncType {
    Equals,
    LT,
    GT,
    And,
    Or,
}

#[derive(Debug)]
pub enum Condition {
    True,
    False,
    Column(usize),
    Func(FuncType, Box<Condition>, Box<Condition>),
    Const(ValueType),
}

#[derive(Debug)]
pub struct Query {
    pub select: Vec<usize>,
    pub filter: Condition,
}

fn eval(record: &Vec<ValueType>, condition: &Condition) -> ValueType {
    use self::Condition::*;
    use self::ValueType::*;
    match condition {
        &True => Bool(true),
        &False => Bool(false),
        &Func(ref functype, ref exp1, ref exp2) => {
            match (functype, eval(record, &exp1), eval(record, &exp2)) {
                (&FuncType::Equals, v1, v2) => Bool(v1 == v2),
                (&FuncType::And, Bool(b1), Bool(b2)) => Bool(b1 && b2),
                (&FuncType::Or, Bool(b1), Bool(b2)) => Bool(b1 || b2),
                (&FuncType::LT, Integer(i1), Integer(i2)) => Bool(i1 < i2),
                (&FuncType::LT, Timestamp(t1), Timestamp(t2)) => Bool(t1 < t2),
                (&FuncType::LT, Float(f1), Float(f2)) => Bool(f1 < f2),
                (&FuncType::GT, Integer(i1), Integer(i2)) => Bool(i1 > i2),
                (&FuncType::GT, Timestamp(t1), Timestamp(t2)) => Bool(t1 > t2),
                (&FuncType::GT, Float(f1), Float(f2)) => Bool(f1 > f2),
                (functype, v1, v2) => panic!(
                    "Type error: function {:?} not defined for values {:?} and {:?}",
                    functype, v1, v2
                ),
            }
        }
        &Column(col) => record[col].clone(),
        &Const(ref value) => value.clone(),
    }
}

fn run(query: &Query, source: &Vec<Vec<ValueType>>) -> Vec<Vec<ValueType>> {
    let mut result = Vec::new();
    for record in source.iter() {
        if eval(record, &query.filter) == ValueType::Bool(true) {
            result.push(
                query
                    .select
                    .iter()
                    .map(|&col| record[col].clone())
                    .collect(),
            );
        }
    }
    result
}

fn record(timestamp: u64, url: &str, loadtime: f64) -> Vec<ValueType> {
    vec![
        ValueType::Timestamp(timestamp),
        ValueType::String(Rc::new(url.to_string())),
        ValueType::Float(loadtime),
    ]
}

pub fn test() {
    let dataset = vec![
        record(1200, "/", 0.4),
        record(1231, "/", 0.3),
        record(1132, "/admin", 1.2),
        record(994, "/admin/crashdash", 3.4),
        record(931, "/", 0.8),
    ];

    use self::Condition::*;
    use self::FuncType::*;
    use ValueType::*;

    let query1 = Query {
        select: vec![1usize],
        filter: Func(
            And,
            Box::new(Func(
                LT,
                Box::new(Column(2usize)),
                Box::new(Const(Float(1.0))),
            )),
            Box::new(Func(
                GT,
                Box::new(Column(0usize)),
                Box::new(Const(Timestamp(1000))),
            )),
        ),
    };

    let query2 = Query {
        select: vec![0usize, 2usize],
        filter: Func(
            Equals,
            Box::new(Column(1usize)),
            Box::new(Const(String(Rc::new("/".to_string())))),
        ),
    };

    let result1 = run(&query1, &dataset);
    let result2 = run(&query2, &dataset);

    println!("Result 1: {:?}", result1);
    println!("Result 2: {:?}", result2)
}
