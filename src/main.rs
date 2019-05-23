extern crate serde_json;
extern crate time;
#[macro_use]
extern crate nom;
extern crate heapsize;
extern crate itertools;
extern crate rustyline;

mod aggregator;
mod columns;
mod csv_loader;
mod expression;
mod parser;
mod query_engine;
mod util;
mod value;
use columns::{columnarize, Batch, Column};
use heapsize::HeapSizeOf;
use time::precise_time_s;
use value::{RecordType, ValueType};

use itertools::Itertools;
use serde_json::Value;
use std::env;
use std::fs::File;
use std::io::BufReader;
use std::rc::Rc;

const LOAD_CHUNK_SIZE: usize = 100_000;

fn json_to_value(json: Value) -> ValueType {
    match json {
        Value::Null => ValueType::Null,
        Value::Bool(b) => ValueType::Integer(b as i64),
        Value::Number(n) => {
            n.as_i64()
                .map(ValueType::Integer)
                .or(n.as_f64().map(|f| ValueType::Integer((1000.0 * f) as i64)))
                .unwrap()
        }
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

fn repl(datasource: &Vec<Batch>) {
    use std::io::{stdin, stdout, Write};
    let mut rl = rustyline::Editor::<()>::new();
    rl.load_history(".rivi_history");
    loop {
        let mut s = rl.readline("rivi>> ").expect(
            "Did not enter a correct string",
        );
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
        rl.add_history_entry(s.as_ref());
        match parser::parse_query(s.as_bytes()) {
            Ok((remaining, query)) => {
                println!("{:?}, {:?}\n", query, remaining);
                let result = query.run_batches(datasource);
                query_engine::print_query_result(&result);
            }
            err => {
                println!("Failed to parse query! {:?}", err);
                println!("Example for supported query:");
                println!(
                    "select url, count(1), app_name, sum(events) where and( >(timestamp, 1000), =(version, \"1.5.3\") )\n"
                );
            }
        }
        rl.save_history(".rivi_history").unwrap();
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let data_iter = csv_loader::load_csv_file(&args[1]);
    let columnarization_start_time = precise_time_s();
    let batches: Vec<Batch> = data_iter
        .chunks(LOAD_CHUNK_SIZE)
        .into_iter()
        .map(|chunk| columnarize(chunk.collect()))
        .collect();
    let bytes_in_ram: usize = batches
        .iter()
        .map(|batch| batch.cols.heap_size_of_children())
        .sum();
    println!(
        "Loaded data into {:.2} MB in RAM in {} chunk(s) in {:.1} seconds.",
        bytes_in_ram as f64 / 1024f64 / 1024f64,
        batches.len(),
        precise_time_s() - columnarization_start_time
    );
    repl(&batches)
}
