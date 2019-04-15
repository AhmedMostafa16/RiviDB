extern crate serde_json;

mod value;
use serde_json::Value;
use std::collections::HashSet;
use std::env;
use std::fs::File;
use std::io::BufReader;
use value::{RecordType, ValueType};

fn json_to_value(json: Value) -> ValueType {
    match json {
        Value::Null => ValueType::Null,
        Value::String(s) => ValueType::String(s),
        Value::Bool(b) => ValueType::Integer(b as i64),
        Value::Number(n) => n
            .as_i64()
            .map(ValueType::Integer)
            .or(n.as_f64().map(ValueType::Float))
            .unwrap(),
        Value::Array(arr) => ValueType::Set(
            arr.into_iter()
                .map(|v| match v {
                    Value::String(s) => s,
                    _ => panic!("Expected list of strings"),
                })
                .collect(),
        ),
        o => panic!("Objects not suppoted: {:?}", o),
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
        panic!("Non-record object: {:?}", json)
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let data = read_data(&args[1]);
    println!("{:?}", data);
}
