use std::boxed::Box;
use std::collections::btree_map::Entry;
use std::collections::BTreeMap;
use std::iter;
use std::rc::Rc;
use value::{RecordType, ValueType};

pub trait Column {
    fn get_name(&self) -> &str;
    fn iter(&self) -> ColIter;
}

pub struct ColIter<'a> {
    iter: Box<Iterator<Item = ValueType> + 'a>,
}

impl<'a> Iterator for ColIter<'a> {
    type Item = ValueType;

    fn next(&mut self) -> Option<ValueType> {
        self.iter.next()
    }
}

struct NullColumn {
    name: String,
    length: usize,
}

impl NullColumn {
    fn new(name: String, length: usize) -> NullColumn {
        NullColumn {
            name: name,
            length: length,
        }
    }
}

impl Column for NullColumn {
    fn get_name(&self) -> &str {
        &self.name
    }

    fn iter<'a>(&'a self) -> ColIter<'a> {
        let iter = iter::repeat(ValueType::Null).take(self.length);
        ColIter {
            iter: Box::new(iter),
        }
    }
}

struct BoolColumn {
    name: String,
    values: Vec<bool>,
}

impl BoolColumn {
    fn new(name: String, values: Vec<bool>) -> BoolColumn {
        BoolColumn {
            name: name,
            values: values,
        }
    }
}

impl Column for BoolColumn {
    fn get_name(&self) -> &str {
        &self.name
    }

    fn iter<'a>(&'a self) -> ColIter<'a> {
        let iter = self.values.iter().map(|&b| ValueType::Bool(b));
        ColIter {
            iter: Box::new(iter),
        }
    }
}

struct TimestampColumn {
    name: String,
    values: Vec<u64>,
}

impl TimestampColumn {
    fn new(name: String, values: Vec<u64>) -> TimestampColumn {
        TimestampColumn {
            name: name,
            values: values,
        }
    }
}

impl Column for TimestampColumn {
    fn get_name(&self) -> &str {
        &self.name
    }

    fn iter<'a>(&'a self) -> ColIter<'a> {
        let iter = self.values.iter().map(|&t| ValueType::Timestamp(t));
        ColIter {
            iter: Box::new(iter),
        }
    }
}

struct IntegerColumn {
    name: String,
    values: Vec<i64>,
}

impl IntegerColumn {
    fn new(name: String, values: Vec<i64>) -> IntegerColumn {
        IntegerColumn {
            name: name,
            values: values,
        }
    }
}

impl Column for IntegerColumn {
    fn get_name(&self) -> &str {
        &self.name
    }

    fn iter<'a>(&'a self) -> ColIter<'a> {
        let iter = self.values.iter().map(|&i| ValueType::Integer(i));
        ColIter {
            iter: Box::new(iter),
        }
    }
}

struct StringColumn {
    name: String,
    values: Vec<String>,
}

impl StringColumn {
    fn new(name: String, values: Vec<String>) -> StringColumn {
        StringColumn {
            name: name,
            values: values,
        }
    }
}

impl Column for StringColumn {
    fn get_name(&self) -> &str {
        &self.name
    }

    fn iter<'a>(&'a self) -> ColIter<'a> {
        let iter = self
            .values
            .iter()
            .map(|s| ValueType::Str(Rc::new(s.clone())));
        ColIter {
            iter: Box::new(iter),
        }
    }
}

struct SetColumn {
    name: String,
    values: Vec<Vec<String>>,
}

impl SetColumn {
    fn new(name: String, values: Vec<Vec<String>>) -> SetColumn {
        SetColumn {
            name: name,
            values: values,
        }
    }
}

impl Column for SetColumn {
    fn get_name(&self) -> &str {
        &self.name
    }

    fn iter<'a>(&'a self) -> ColIter<'a> {
        let iter = self
            .values
            .iter()
            .map(|s| ValueType::Set(Rc::new(s.clone())));
        ColIter {
            iter: Box::new(iter),
        }
    }
}

struct MixedColumn {
    name: String,
    values: Vec<ValueType>,
}

impl MixedColumn {
    fn new(name: String, values: Vec<ValueType>) -> MixedColumn {
        MixedColumn {
            name: name,
            values: values,
        }
    }
}

impl Column for MixedColumn {
    fn get_name(&self) -> &str {
        &self.name
    }

    fn iter<'a>(&'a self) -> ColIter<'a> {
        let iter = self.values.iter().cloned();
        ColIter {
            iter: Box::new(iter),
        }
    }
}

enum VecType {
    NullVec(usize),
    BoolVec(Vec<bool>),
    TimestampVec(Vec<u64>),
    IntegerVec(Vec<i64>),
    StringVec(Vec<String>),
    SetVec(Vec<Vec<String>>),
    MixedVec(Vec<ValueType>),
}

impl VecType {
    fn new_with_value(value: &ValueType) -> VecType {
        // TODO: fix
        use self::VecType::*;
        match value {
            &ValueType::Null => NullVec(0),
            &ValueType::Bool(_) => BoolVec(Vec::new()),
            &ValueType::Timestamp(_) => TimestampVec(Vec::new()),
            &ValueType::Integer(_) => IntegerVec(Vec::new()),
            &ValueType::Str(_) => StringVec(Vec::new()),
            &ValueType::Set(_) => SetVec(Vec::new()),
        }
    }

    fn push(&mut self, value: ValueType) -> Option<ValueType> {
        match self {
            &mut VecType::NullVec(ref mut n) => match value {
                ValueType::Null => {
                    *n += 1;
                    None
                }
                _ => Some(value),
            },
            &mut VecType::BoolVec(ref mut v) => match value {
                ValueType::Bool(b) => {
                    v.push(b);
                    None
                }
                _ => Some(value),
            },
            &mut VecType::TimestampVec(ref mut v) => match value {
                ValueType::Timestamp(t) => {
                    v.push(t);
                    None
                }
                _ => Some(value),
            },
            &mut VecType::IntegerVec(ref mut v) => match value {
                ValueType::Integer(i) => {
                    v.push(i);
                    None
                }
                _ => Some(value),
            },
            &mut VecType::StringVec(ref mut v) => match value {
                ValueType::Str(s) => {
                    v.push(Rc::try_unwrap(s).unwrap());
                    None
                }
                _ => Some(value),
            },
            &mut VecType::SetVec(ref mut v) => match value {
                ValueType::Set(s) => {
                    v.push(Rc::try_unwrap(s).unwrap());
                    None
                }
                _ => Some(value),
            },
            &mut VecType::MixedVec(ref mut v) => {
                v.push(value);
                None
            }
        }
    }

    fn to_mixed(self) -> VecType {
        match self {
            VecType::NullVec(n) => {
                VecType::MixedVec(iter::repeat(ValueType::Null).take(n).collect())
            }
            VecType::BoolVec(v) => {
                VecType::MixedVec(v.into_iter().map(|b| ValueType::Bool(b)).collect())
            }
            VecType::TimestampVec(v) => {
                VecType::MixedVec(v.into_iter().map(|t| ValueType::Timestamp(t)).collect())
            }
            VecType::IntegerVec(v) => {
                VecType::MixedVec(v.into_iter().map(|i| ValueType::Integer(i)).collect())
            }
            VecType::StringVec(v) => {
                VecType::MixedVec(v.into_iter().map(|s| ValueType::Str(Rc::new(s))).collect())
            }
            VecType::SetVec(v) => {
                VecType::MixedVec(v.into_iter().map(|s| ValueType::Set(Rc::new(s))).collect())
            }
            vec @ VecType::MixedVec(_) => vec,
        }
    }

    fn to_column(self, name: String) -> Box<Column> {
        match self {
            VecType::NullVec(n) => Box::new(NullColumn::new(name, n)),
            VecType::BoolVec(v) => Box::new(BoolColumn::new(name, v)),
            VecType::TimestampVec(v) => Box::new(TimestampColumn::new(name, v)),
            VecType::IntegerVec(v) => Box::new(IntegerColumn::new(name, v)),
            VecType::StringVec(v) => Box::new(StringColumn::new(name, v)),
            VecType::SetVec(v) => Box::new(SetColumn::new(name, v)),
            VecType::MixedVec(v) => Box::new(MixedColumn::new(name, v)),
        }
    }
}

pub fn columnarize(records: Vec<RecordType>) -> Vec<Box<Column>> {
    let mut field_map = BTreeMap::new();
    for record in records {
        for (name, value) in record {
            let to_insert = match field_map.entry(name) {
                Entry::Vacant(e) => {
                    e.insert(VecType::new_with_value(&value));
                    None
                }
                Entry::Occupied(mut e) => {
                    if let Some(value) = e.get_mut().push(value) {
                        let (name, vec) = e.remove_entry();
                        let mut mixed_vec = vec.to_mixed();
                        mixed_vec.push(value);
                        Some((name, mixed_vec))
                    } else {
                        None
                    }
                }
            };
            if let Some((name, vec)) = to_insert {
                field_map.insert(name, vec);
            }
        }
    }

    let mut columns = Vec::new();
    for (name, values) in field_map {
        columns.push(values.to_column(name))
    }

    columns
}
