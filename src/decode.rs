use crate::types::{MsgPackError, MsgPackValue, MsgPackEntry};
use byteorder::{ReadBytesExt, BigEndian};
use wasm_bindgen::prelude::*;
use std::io::{Cursor, Read};
use rmp::Marker;


/// Turns a MessagePack-encoded buffer into a json-encoded MsgPackEntry string
/// 
/// # Examples 
/// 
/// ```
/// let input = vec![0xC3];
/// let value = rmpp::unpack_json(&input, Some(false)).unwrap();
/// 
/// assert_eq!(
///     r###"{"raw_marker":195,"basic_type":"Bool","data":{"type":"Bool","value":true}}"###, 
///     value
/// );
/// ```
#[wasm_bindgen]
pub fn unpack_json(data: &[u8], pretty: Option<bool>) -> Result<String, JsValue> {
    let value = read_value(&mut Cursor::new(data))
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    if pretty.unwrap_or(false) { serde_json::to_string_pretty(&value) } else { serde_json::to_string(&value) } 
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Turns a MessagePack-encoded buffer into a MsgPackEntry object
/// 
/// # Examples 
/// 
/// ```
/// let input = vec![0xC3];
/// let value = rmpp::unpack(&input).unwrap();
/// 
/// let entry = rmpp::MsgPackEntry::new(
///     195, rmpp::MsgPackValue::Bool(true)
/// );
/// 
/// assert_eq!(entry, value);
/// ```
pub fn unpack(data: &[u8]) -> Result<MsgPackEntry, MsgPackError> {
    read_value(&mut Cursor::new(data))
}

/// Reads a MessagePack buffer value and returns a MsgPackEntry object
/// 
/// If a value is of collection type (e.g. Array or Map), it'll read the entire collection
fn read_value<R: Read>(reader: &mut R) -> Result<MsgPackEntry, MsgPackError> {
    // Read the marker
    let raw_marker: u8 = reader.read_u8()?;
    let marker: Marker = Marker::from_u8(raw_marker);

    // Read the value
    // Sorted by format families
    let value: MsgPackValue = match marker {
        // Null - marker itself represents a value
        Marker::Null => MsgPackValue::Null,
        // Boolean - marker itself represents a value
        Marker::False => MsgPackValue::Bool(false),
        Marker::True  => MsgPackValue::Bool(true),
        // Positive Fixed Integer - marker itself represents a value from 0 to 127
        Marker::FixPos(val) => MsgPackValue::FixPos(val),
        // Negative Fixed Integer - marker itself represents a value from -32 to -1
        Marker::FixNeg(val) => MsgPackValue::FixNeg(val),
        // Unsigned Integer - first 1/2/4/8 byte(s) after the marker represent the value
        Marker::U8  => { MsgPackValue::U8(reader.read_u8()?) }
        Marker::U16 => { MsgPackValue::U16(reader.read_u16::<BigEndian>()?) }
        Marker::U32 => { MsgPackValue::U32(reader.read_u32::<BigEndian>()?) }
        Marker::U64 => { MsgPackValue::U64(reader.read_u64::<BigEndian>()?) }
        // Signed Integer - first 1/2/4/8 byte(s) after the marker represent the value
        Marker::I8  => { MsgPackValue::I8(reader.read_i8()?) }
        Marker::I16 => { MsgPackValue::I16(reader.read_i16::<BigEndian>()?) }
        Marker::I32 => { MsgPackValue::I32(reader.read_i32::<BigEndian>()?) }
        Marker::I64 => { MsgPackValue::I64(reader.read_i64::<BigEndian>()?) }
        // Float - first 4/8 bytes after the marker represent the value
        Marker::F32 => { MsgPackValue::F32(reader.read_f32::<BigEndian>()?) }
        Marker::F64 => { MsgPackValue::F64(reader.read_f64::<BigEndian>()?) }
        // String
        Marker::FixStr(_)|Marker::Str8|Marker::Str16|Marker::Str32 => { read_str(reader, marker)? },
        // Binary
        Marker::Bin8|Marker::Bin16|Marker::Bin32 => { read_bin(reader, marker)? },
        // Array
        Marker::FixArray(_)|Marker::Array16|Marker::Array32 => { read_array(reader, marker)? },
        // Map
        Marker::FixMap(_)|Marker::Map16|Marker::Map32 => { read_map(reader, marker)? },
        // Extension - I don't really care about it, teehee
        Marker::Ext8|Marker::Ext16|Marker::Ext32|
        Marker::FixExt1|Marker::FixExt2|Marker::FixExt4|Marker::FixExt8|Marker::FixExt16 => {
            unimplemented!()
        },
        Marker::Reserved => {
            unreachable!()
        }
    };

    Ok(MsgPackEntry::new(raw_marker, value))
}


/// Reads MessagePack strings
fn read_str<R: Read>(reader: &mut R, marker: Marker) -> Result<MsgPackValue, MsgPackError> {
    let len: usize = match marker {
        // FixStr has the length from 0 to 31 encoded inside of it
        Marker::FixStr(val) => { usize::from(val & 0b0001_1111) } // Lower 5 bits represent the length
        // Otherwise, the first 1/2/4 byte(s) after the marker represent the length
        Marker::Str8  => { reader.read_u8()? as usize },
        Marker::Str16 => { reader.read_u16::<BigEndian>()? as usize },
        Marker::Str32 => { reader.read_u32::<BigEndian>()? as usize },
        _ => unreachable!()
    };

    // After that comes the string data
    let mut buf: Vec<u8> = vec![0u8;len];
    reader.read_exact(&mut buf)?;
    let s=String::from_utf8(buf).map_err(|e| MsgPackError::Custom(format!("Invalid UTF-8: {}", e)))?;

    let res: MsgPackValue = match marker {
        Marker::FixStr(_) => { MsgPackValue::FixStr(s) }
        Marker::Str8  =>     { MsgPackValue::Str8(s)   },
        Marker::Str16 =>     { MsgPackValue::Str16(s)  },
        Marker::Str32 =>     { MsgPackValue::Str32(s)  },
        _ => unreachable!()
    };

    Ok(res)
}

/// Reads MessagePack binary
fn read_bin<R: Read>(reader: &mut R, marker: Marker) -> Result<MsgPackValue, MsgPackError> {
    let len: usize = match marker {
        // The first 1/2/4 byte(s) after the marker represent the length
        Marker::Bin8  => { reader.read_u8()? as usize }
        Marker::Bin16 => { reader.read_u16::<BigEndian>()? as usize }
        Marker::Bin32 => { reader.read_u32::<BigEndian>()? as usize }
        _ => unreachable!()
    };
    
    // After that comes the binary data
    let mut buf: Vec<u8> = vec![0u8;len];
    reader.read_exact(&mut buf)?;

    let res: MsgPackValue = match marker {
        Marker::Bin8  => { MsgPackValue::Bin8(buf)  }
        Marker::Bin16 => { MsgPackValue::Bin16(buf) }
        Marker::Bin32 => { MsgPackValue::Bin32(buf) }
        _ => unreachable!()
    };

    Ok(res)
}

/// Reads MessagePack arrays
fn read_array<R: Read>(reader: &mut R, marker: Marker) -> Result<MsgPackValue, MsgPackError> {
    let len: usize = match marker {
        // FixArray has the length from 0 to 15 encoded inside of it
        Marker::FixArray(val) => { usize::from(val & 0b0000_1111) }, // Lower 4 bits represent the length
        // Otherwise, the first 2/4 bytes after the marker represent the length
        Marker::Array16 => { reader.read_u16::<BigEndian>()? as usize },
        Marker::Array32 => { reader.read_u32::<BigEndian>()? as usize },
        _ => unreachable!()
    };

    // After that comes the array data
    let mut array: Vec<MsgPackEntry> = Vec::with_capacity(len);
    for _ in 0..len { array.push(read_value(reader)?); } // Recursively read each element

    let res: MsgPackValue = match marker {
        Marker::FixArray(_) => { MsgPackValue::FixArray(array) },
        Marker::Array16 =>     { MsgPackValue::Array16(array)  },
        Marker::Array32 =>     { MsgPackValue::Array32(array)  },
        _ => unreachable!()
    };

    Ok(res)
}

/// Reads MessagePack maps
fn read_map<R: Read>(reader: &mut R, marker: Marker) -> Result<MsgPackValue, MsgPackError> {
    let len: usize = match marker {
        // FixMap has the length from 0 to 15 encoded inside of it
        Marker::FixMap(val) => { usize::from(val & 0b0000_1111) }, // Lower 4 bits represent the length
        // Otherwise, the first 2/4 bytes after the marker represent the length
        Marker::Map16 => { reader.read_u16::<BigEndian>()? as usize },
        Marker::Map32 => { reader.read_u32::<BigEndian>()? as usize },
        _ => unreachable!()
    };

    // After that comes the map data
    let mut map: Vec<_> = Vec::with_capacity(len);
    for _ in 0..len { 
        // Recursively read each element
        let k: MsgPackEntry = read_value(reader)?; 
        let v: MsgPackEntry = read_value(reader)?;
        map.push((k, v));
    }

    let res: MsgPackValue = match marker {
        Marker::FixMap(_) => { MsgPackValue::FixMap(map) },
        Marker::Map16 =>     { MsgPackValue::Map16(map)  },
        Marker::Map32 =>     { MsgPackValue::Map32(map)  },
        _ => unreachable!()
    };
    
    Ok(res)
}