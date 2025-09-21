use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::sync::Arc;
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
enum RawAllayData {
    String(String),
    Int(i64),
    Bool(bool),
    List(Vec<RawAllayData>),
    Object(HashMap<String, RawAllayData>),
    Null,
}

impl From<RawAllayData> for AllayData {
    fn from(raw: RawAllayData) -> Self {
        match raw {
            RawAllayData::String(str) => AllayData::String(str),
            RawAllayData::Int(int) => AllayData::Int(int),
            RawAllayData::Bool(bool) => AllayData::Bool(bool),
            RawAllayData::List(list) => {
                AllayData::List(list.into_iter().map(AllayData::from).map(Arc::new).collect())
            }
            RawAllayData::Object(obj) => AllayData::Object(
                obj.into_iter().map(|(k, v)| (k, Arc::new(AllayData::from(v)))).collect(),
            ),
            RawAllayData::Null => AllayData::Null,
        }
    }
}

pub type AllayList = Vec<Arc<AllayData>>;
pub type AllayObject = HashMap<String, Arc<AllayData>>;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum AllayData {
    String(String),
    Int(i64),
    Bool(bool),
    List(AllayList),
    Object(AllayObject),
    #[default]
    Null,
}

impl AllayData {
    pub fn from_toml(content: &str) -> DataResult<AllayObject> {
        let data: RawAllayData = toml::from_str(content)?;
        match data.into() {
            AllayData::Object(obj) => Ok(obj),
            _ => Err(AllayDataError::TypeConversion(
                "TOML root is not a table".to_string(),
            )),
        }
    }

    pub fn from_yaml(content: &str) -> DataResult<AllayObject> {
        let data: RawAllayData = serde_yaml::from_str(content)?;
        match data.into() {
            AllayData::Object(obj) => Ok(obj),
            _ => Err(AllayDataError::TypeConversion(
                "YAML root is not an object".to_string(),
            )),
        }
    }

    pub fn from_json(content: &str) -> DataResult<AllayObject> {
        let data: RawAllayData = serde_json::from_str(content)?;
        match data.into() {
            AllayData::Object(obj) => Ok(obj),
            _ => Err(AllayDataError::TypeConversion(
                "JSON root is not an object".to_string(),
            )),
        }
    }

    pub fn is_str(&self) -> bool {
        matches!(self, AllayData::String(_))
    }

    pub fn is_int(&self) -> bool {
        matches!(self, AllayData::Int(_))
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

    pub fn as_str(&self) -> DataResult<&String> {
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
            Err(AllayDataError::TypeConversion("not an object".to_string()))
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
            Err(AllayDataError::TypeConversion("not an object".to_string()))
        }
    }

    pub fn to_list(self) -> DataResult<AllayList> {
        if let AllayData::List(list) = self {
            Ok(list)
        } else {
            Err(AllayDataError::TypeConversion("not a list".to_string()))
        }
    }

    pub fn to_obj(self) -> DataResult<AllayObject> {
        if let AllayData::Object(obj) = self {
            Ok(obj)
        } else {
            Err(AllayDataError::TypeConversion("not an object".to_string()))
        }
    }

    pub fn arc_to_list(data: Arc<AllayData>) -> DataResult<AllayList> {
        Ok(data.as_list()?.iter().map(Arc::clone).collect())
    }

    /// NOTE: This function will copy the keys
    pub fn arc_to_obj(data: Arc<AllayData>) -> DataResult<AllayObject> {
        Ok(data.as_obj()?.iter().map(|(k, v)| (k.clone(), Arc::clone(v))).collect())
    }
}

impl fmt::Display for AllayData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AllayData::String(str) => write!(f, "{}", str),
            AllayData::Int(int) => write!(f, "{}", int),
            AllayData::Bool(bool) => write!(f, "{}", bool),
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

impl From<bool> for AllayData {
    fn from(b: bool) -> Self {
        AllayData::Bool(b)
    }
}

impl From<AllayList> for AllayData {
    fn from(list: AllayList) -> Self {
        AllayData::List(list)
    }
}

impl From<AllayObject> for AllayData {
    fn from(obj: AllayObject) -> Self {
        AllayData::Object(obj)
    }
}

impl From<()> for AllayData {
    fn from(_: ()) -> Self {
        AllayData::Null
    }
}
