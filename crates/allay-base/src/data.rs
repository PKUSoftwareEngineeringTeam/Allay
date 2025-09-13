use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AllayDataError {
    #[error("File error: {0}")]
    File(#[from] crate::file::FileError),

    #[error("TOML parse error: {0}")]
    TomlParse(#[from] toml::de::Error),

    #[error("TOML serialize error: {0}")]
    TomlSerialize(#[from] toml::ser::Error),

    #[error("YAML parse error: {0}")]
    YamlParse(#[from] serde_yaml::Error),

    #[error("JSON parse error: {0}")]
    JsonParse(#[from] serde_json::Error),

    #[error("Unsupported file format: {0}")]
    UnsupportedFormat(String),

    #[error("Type conversion error: {0}")]
    TypeConversion(String),

    #[error("Key not found: {0}")]
    KeyNotFound(String),

    #[error("Index out of bounds: {0}")]
    IndexOutOfBounds(usize),

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
}

pub type DataResult<T> = Result<T, AllayDataError>;

pub type AllayList = Vec<AllayData>;
pub type AllayObject = HashMap<String, AllayData>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AllayData {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    List(AllayList),
    Object(AllayObject),
    Null,
}

impl AllayData {
    pub fn from_toml(content: &str) -> DataResult<AllayObject> {
        let data: AllayData = toml::from_str(content)?;
        match data {
            AllayData::Object(obj) => Ok(obj),
            _ => Err(AllayDataError::TypeConversion(
                "TOML root is not a table".to_string(),
            )),
        }
    }

    pub fn from_yaml(content: &str) -> DataResult<AllayObject> {
        let data: AllayData = serde_yaml::from_str(content)?;
        match data {
            AllayData::Object(obj) => Ok(obj),
            _ => Err(AllayDataError::TypeConversion(
                "YAML root is not a object".to_string(),
            )),
        }
    }

    pub fn from_json(content: &str) -> DataResult<AllayObject> {
        let data: AllayData = serde_json::from_str(content)?;
        match data {
            AllayData::Object(obj) => Ok(obj),
            _ => Err(AllayDataError::TypeConversion(
                "JSON root is not an object".to_string(),
            )),
        }
    }

    pub fn to_toml(&self) -> DataResult<String> {
        Ok(toml::to_string(self)?)
    }

    pub fn to_yaml(&self) -> DataResult<String> {
        Ok(serde_yaml::to_string(self)?)
    }

    pub fn to_json(&self) -> DataResult<String> {
        Ok(serde_json::to_string(self)?)
    }

    pub fn is_string(&self) -> bool {
        matches!(self, AllayData::String(_))
    }

    pub fn is_int(&self) -> bool {
        matches!(self, AllayData::Int(_))
    }

    pub fn is_float(&self) -> bool {
        matches!(self, AllayData::Float(_))
    }

    pub fn is_bool(&self) -> bool {
        matches!(self, AllayData::Bool(_))
    }

    pub fn is_list(&self) -> bool {
        matches!(self, AllayData::List(_))
    }

    pub fn is_obj(&self) -> bool {
        matches!(self, AllayData::Object(_))
    }

    pub fn is_null(&self) -> bool {
        matches!(self, AllayData::Null)
    }

    pub fn as_string(&self) -> DataResult<&String> {
        if let AllayData::String(s) = self {
            Ok(s)
        } else {
            Err(AllayDataError::TypeConversion("not a string".to_string()))
        }
    }

    pub fn as_int(&self) -> DataResult<i64> {
        if let AllayData::Int(i) = self {
            Ok(*i)
        } else {
            Err(AllayDataError::TypeConversion("not an integer".to_string()))
        }
    }

    pub fn as_float(&self) -> DataResult<f64> {
        match self {
            AllayData::Float(f) => Ok(*f),
            AllayData::Int(i) => Ok(*i as f64),
            _ => Err(AllayDataError::TypeConversion("not a float".to_string())),
        }
    }

    pub fn as_bool(&self) -> DataResult<bool> {
        if let AllayData::Bool(b) = self {
            Ok(*b)
        } else {
            Err(AllayDataError::TypeConversion("not a boolean".to_string()))
        }
    }

    pub fn as_list(&self) -> DataResult<&AllayList> {
        if let AllayData::List(list) = self {
            Ok(list)
        } else {
            Err(AllayDataError::TypeConversion("not a list".to_string()))
        }
    }

    pub fn as_obj(&self) -> DataResult<&AllayObject> {
        if let AllayData::Object(obj) = self {
            Ok(obj)
        } else {
            Err(AllayDataError::TypeConversion("not a object".to_string()))
        }
    }

    pub fn as_list_mut(&mut self) -> DataResult<&mut AllayList> {
        if let AllayData::List(list) = self {
            Ok(list)
        } else {
            Err(AllayDataError::TypeConversion("not a list".to_string()))
        }
    }

    pub fn as_obj_mut(&mut self) -> DataResult<&mut AllayObject> {
        if let AllayData::Object(obj) = self {
            Ok(obj)
        } else {
            Err(AllayDataError::TypeConversion("not a object".to_string()))
        }
    }
}

impl fmt::Display for AllayData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AllayData::String(s) => write!(f, "{}", s),
            AllayData::Int(i) => write!(f, "{}", i),
            AllayData::Float(fl) => write!(f, "{}", fl),
            AllayData::Bool(b) => write!(f, "{}", b),
            AllayData::List(list) => {
                write!(f, "[")?;
                for (i, item) in list.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            }
            AllayData::Object(obj) => {
                write!(f, "{{")?;
                for (i, (key, value)) in obj.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", key, value)?;
                }
                write!(f, "}}")
            }
            AllayData::Null => write!(f, "null"),
        }
    }
}

impl From<String> for AllayData {
    fn from(s: String) -> Self {
        AllayData::String(s)
    }
}

impl From<&str> for AllayData {
    fn from(s: &str) -> Self {
        AllayData::String(s.to_string())
    }
}

impl From<i32> for AllayData {
    fn from(i: i32) -> Self {
        AllayData::Int(i as i64)
    }
}

impl From<i64> for AllayData {
    fn from(i: i64) -> Self {
        AllayData::Int(i)
    }
}

impl From<f32> for AllayData {
    fn from(f: f32) -> Self {
        AllayData::Float(f as f64)
    }
}

impl From<f64> for AllayData {
    fn from(f: f64) -> Self {
        AllayData::Float(f)
    }
}

impl From<bool> for AllayData {
    fn from(b: bool) -> Self {
        AllayData::Bool(b)
    }
}

impl<T: Into<AllayData>> From<Vec<T>> for AllayData {
    fn from(vec: Vec<T>) -> Self {
        AllayData::List(vec.into_iter().map(|x| x.into()).collect())
    }
}

impl From<HashMap<String, AllayData>> for AllayData {
    fn from(obj: HashMap<String, AllayData>) -> Self {
        AllayData::Object(obj)
    }
}

impl From<()> for AllayData {
    fn from(_: ()) -> Self {
        AllayData::Null
    }
}
