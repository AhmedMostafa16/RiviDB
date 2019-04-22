extern crate serde_json;
extern crate time;
#[macro_use]
extern crate nom;
extern crate heapsize;

mod aggregator;
mod columns;
mod expression;
mod parser;
mod query_engine;
mod util;
mod value;
use columns::columnarize;
use columns::Column;
use heapsize::HeapSizeOf;
use time::precise_time_s;
use value::{RecordType, ValueType};

use serde_json::Value;
use std::env;
use std::fs::File;
use std::io::BufReader;
use std::rc::Rc;

fn json_to_value(json: Value) -> ValueType {
    match json {
        Value::Null => ValueType::Null,
        Value::Bool(b) => ValueType::Integer(b as i64),
        Value::Number(n) => n
            .as_i64()
            .map(ValueType::Integer)
            .or(n.as_f64().map(|f| ValueType::Integer((1000.0 * f) as i64)))
            .unwrap(),
        Value::String(s) => ValueType::Str(Rc::new(s)),
        Value::Array(arr) => ValueType::Set(Rc::new(
            arr.into_iter()
                .map(|v| match v {
                    Value::String(s) => s,
                    _ => panic!("Expected list of strings"),
                })
                .collect(),
        )),
        o => panic!("Objects not supported: {:?}", o),
    }
}

fn json_to_record(json: Value) -> RecordType {
    if let Value::Object(object) = json {
        object
            .into_iter()
            .map(|(k, v)| (k, json_to_value(v)))
            .collect()
    } else {
        panic!("Non-record object: {:?}", json)
    }
}

fn read_data(filename: &str) -> Vec<RecordType> {
    let file = BufReader::new(File::open(filename).unwrap());
    let json = serde_json::from_reader(file).unwrap();
    if let Value::Array(data) = json {
        data.into_iter().map(|v| json_to_record(v)).collect()
    } else {
        panic!("Unexpected JSON contents.")
    }
}

fn repl(datasource: &Vec<Box<Column>>) {
    use std::io::{stdin, stdout, Write};
    loop {
        let mut s = String::new();
        print!("rivi> ");
        let _ = stdout().flush();
        stdin()
            .read_line(&mut s)
            .expect("Did not enter a correct string");
        if let Some('\n') = s.chars().next_back() {
            s.pop();
        }
        if let Some('\r') = s.chars().next_back() {
            s.pop();
        }
        if s == "exit" {
            break;
        }
        if s.chars().next_back() != Some(';') {
            s.push(';');
        }
        match parser::parse_query(s.as_bytes()) {
            Ok((remaining, query)) => {
                println!("{:?}, {:?}\n", query, remaining);
                let result = query.run(datasource);
                query_engine::print_query_result(&result);
            }
            err => println!("Failed to parse query! {:?}", err),
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let data = read_data(if args.len() > 1 {
        &args[1]
    } else {
        "test2.json"
    });
    let cols = columnarize(data);
    let bytes_in_ram = cols.heap_size_of_children();
    let columnarization_start_time = precise_time_s();
    println!(
        "Loaded data into {:.2} MiB in RAM in {:.1} seconds.",
        bytes_in_ram as f64 / 1024f64 / 1024f64,
        precise_time_s() - columnarization_start_time
    );
    //query_engine::test(&cols);
    repl(&cols);
}
