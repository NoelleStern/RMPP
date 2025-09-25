use std::io;
use serde::{Deserialize, Serialize};


/// Trait that represents an object holding a MessagePack value
pub trait MsgValue {
    fn get_value(&self) -> &MsgPackValue;
}

/// This is the main type representing a MessagePack entry
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MsgPackEntry {
    /// A raw marker value
    pub raw_marker: u8,
    /// A basic type used for easier JS integration
    pub basic_type: BasicTypes,
    /// The value itself
    pub data: MsgPackValue,
}
impl MsgPackEntry {
    pub fn new(raw_marker: u8, value: MsgPackValue) -> Self {
        Self { raw_marker, basic_type: value2type(&value), data: value }
    }
}
impl MsgValue for MsgPackEntry {
    fn get_value(&self) -> &MsgPackValue {
        &self.data
    }
}

/// Holds an actual type and value
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum MsgPackValue {
    Null,
    Bool(bool), // It doesn't make sense to separate it into False and True
    FixPos(u8), FixNeg(i8),
    U8(u8), U16(u16), U32(u32), U64(u64),
    I8(i8), I16(i16), I32(i32), I64(i64),
    F32(f32), F64(f64),
    FixStr(String), Str8(String), Str16(String), Str32(String),
    Bin8(Vec<u8>), Bin16(Vec<u8>), Bin32(Vec<u8>),
    FixArray(Vec<MsgPackEntry>), Array16(Vec<MsgPackEntry>), Array32(Vec<MsgPackEntry>),
    FixMap(Vec<(MsgPackEntry, MsgPackEntry)>), Map16(Vec<(MsgPackEntry, MsgPackEntry)>), Map32(Vec<(MsgPackEntry, MsgPackEntry)>),
    // Ext(i8, Vec<u8>),
}
impl MsgValue for MsgPackValue {
    fn get_value(&self) -> &MsgPackValue {
        self
    }
}

/// Basic type used for easier JS integration
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum BasicTypes {
    Null, Bool, Number, String, 
    Bin, Array, Map, // Ext
}

fn value2type(value: &MsgPackValue) -> BasicTypes {
    match value {
        // Null
        MsgPackValue::Null => BasicTypes::Null,
        // Boolean
        MsgPackValue::Bool(_) => BasicTypes::Bool,
        // Integer + Float
        MsgPackValue::FixPos(_)|MsgPackValue::FixNeg(_)|
        MsgPackValue::U8(_)|MsgPackValue::U16(_)|MsgPackValue::U32(_)|MsgPackValue::U64(_)|
        MsgPackValue::I8(_)|MsgPackValue::I16(_)|MsgPackValue::I32(_)|MsgPackValue::I64(_)|
        MsgPackValue::F32(_)|MsgPackValue::F64(_) => BasicTypes::Number,
        // String
        MsgPackValue::FixStr(_)|MsgPackValue::Str8(_)|
        MsgPackValue::Str16(_)|MsgPackValue::Str32(_) => BasicTypes::String,
        // Binary
        MsgPackValue::Bin8(_)|MsgPackValue::Bin16(_)|MsgPackValue::Bin32(_) => BasicTypes::Bin,
        // Array
        MsgPackValue::FixArray(_)|MsgPackValue::Array16(_)|MsgPackValue::Array32(_) => BasicTypes::Array,
        // Map
        MsgPackValue::FixMap(_)|MsgPackValue::Map16(_)|MsgPackValue::Map32(_) => BasicTypes::Map,
        // MsgPackValue::Ext(_,_) => BasicTypes::Ext
    }
}

/// Handles errors
#[derive(Debug)]
pub enum MsgPackError {
    Io(io::Error),
    Custom(String),
}
impl std::error::Error for MsgPackError {}
impl From<io::Error> for MsgPackError {
    fn from(e: io::Error) -> Self { MsgPackError::Io(e) }
}
impl std::fmt::Display for MsgPackError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MsgPackError::Io(e) => write!(f, "IO error: {}", e),
            MsgPackError::Custom(s) => write!(f, "{}", s),
        }
    }
}