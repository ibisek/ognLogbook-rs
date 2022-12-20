// Copied from https://gitlab.com/claudiomattera/rinfluxdb/-/blob/master/rinfluxdb-dataframe/src/lib.rs
// and modified to be somewhat usable in ognLogbook.

// Copyright Claudio Mattera 2021.
// Distributed under the MIT License or Apache 2.0 License at your option.
// See accompanying files License-MIT.txt and License-Apache-2.0, or online at
// https://opensource.org/licenses/MIT
// https://opensource.org/licenses/Apache-2.0

//! Dummy dataframe implementation

use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt;

use chrono::{DateTime, Utc};

use rinfluxdb_types::{DataFrameError, Value};

/// Column type
#[derive(Clone, Debug, PartialEq)]
pub enum Column {
    /// A column of floating point values
    Float(Vec<f64>),

    /// A column of integer values
    Integer(Vec<i64>),

    /// A column of unsigned integer values
    UnsignedInteger(Vec<u64>),

    /// A column of string values
    String(Vec<String>),

    /// A column of boolean values
    Boolean(Vec<bool>),

    /// A column of datetime values
    Timestamp(Vec<DateTime<Utc>>),
}

impl Column {
    fn display_index(&self, index: usize, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Column::Float(values) => write!(f, "{:16}  ", values[index])?,
            Column::Integer(values) => write!(f, "{:16}  ", values[index])?,
            Column::UnsignedInteger(values) => write!(f, "{:16}  ", values[index])?,
            Column::String(values) => write!(f, "{:16}  ", values[index])?,
            Column::Boolean(values) => write!(f, "{:16}  ", values[index])?,
            Column::Timestamp(values) => write!(f, "{:16}  ", values[index])?,
        }

        Ok(())
    }

    pub fn len(&self) -> usize {
        match self {
            Column::Float(values) => values.len(),
            _ => 0,
        }
    }

    pub fn get_float_value(&self, index: usize) -> Option<f64> {
        match self {
            Column::Float(values) => Some(values[index]),
            _ => None,
        }
    }

    pub fn get_int_value(&self, index: usize) -> Option<i64> {
        match self {
            Column::Integer(values) => Some(values[index]),
            _ => None,
        }
    }

}

/// A time-indexed dataframe
///
/// A dataframe contains multiple named columns indexed by the same index.
#[derive(Clone, Debug)]
pub struct DataFrame {
    pub name: String,
    pub index: Vec<DateTime<Utc>>,
    pub columns: HashMap<String, Column>,
}

impl fmt::Display for DataFrame {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:>23}  ", "datetime")?;
        for column in self.columns.keys() {
            write!(f, "{:>16}  ", column)?;
        }
        write!(f, "\n-----------------------  ")?;
        for _column in self.columns.keys() {
            write!(f, "----------------  ")?;
        }
        writeln!(f)?;

        for (i, index) in self.index.iter().enumerate() {
            write!(f, "{:>23}  ", index)?;
            for column in self.columns.values() {
                column.display_index(i, f)?;
            }
            writeln!(f)?;
        }

        Ok(())
    }
}

impl TryFrom<(String, Vec<DateTime<Utc>>, HashMap<String, Vec<Value>>)> for DataFrame {
    type Error = DataFrameError;

    fn try_from(
        (name, index, columns): (String, Vec<DateTime<Utc>>, HashMap<String, Vec<Value>>),
    ) -> Result<Self, Self::Error> {
        let columns: HashMap<String, Result<Column, Self::Error>> = columns
            .into_iter()
            .map(|(name, column)| {
                let column = match column.first() {
                    Some(Value::Float(_)) => Ok(Column::Float(
                        column
                            .into_iter()
                            .map(|element| element.into_f64())
                            .collect(),
                    )),
                    Some(Value::Integer(_)) => Ok(Column::Integer(
                        column
                            .into_iter()
                            .map(|element| element.into_i64())
                            .collect(),
                    )),
                    Some(Value::UnsignedInteger(_)) => Ok(Column::UnsignedInteger(
                        column
                            .into_iter()
                            .map(|element| element.into_u64())
                            .collect(),
                    )),
                    Some(Value::String(_)) => Ok(Column::String(
                        column
                            .into_iter()
                            .map(|element| element.into_string())
                            .collect(),
                    )),
                    Some(Value::Boolean(_)) => Ok(Column::Boolean(
                        column
                            .into_iter()
                            .map(|element| element.into_boolean())
                            .collect(),
                    )),
                    Some(Value::Timestamp(_)) => Ok(Column::Timestamp(
                        column
                            .into_iter()
                            .map(|element| element.into_timestamp())
                            .collect(),
                    )),
                    None => Err(DataFrameError::Creation),
                };
                (name, column)
            })
            .collect();

        let columns = flatten_map(columns)?;

        Ok(Self {
            name,
            index,
            columns,
        })
    }
}

fn flatten_map<K, V, E>(map: HashMap<K, Result<V, E>>) -> Result<HashMap<K, V>, E>
where
    K: Eq + std::hash::Hash,
    E: std::error::Error,
{
    map.into_iter()
        .try_fold(HashMap::new(), |mut accumulator, (name, column)| {
            let column = column?;
            accumulator.insert(name, column);
            Ok(accumulator)
        })
}
