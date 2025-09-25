use std::io::Write;
use wasm_bindgen::prelude::*;
use crate::types::{MsgValue, MsgPackEntry, MsgPackValue};


/// Turns a json-encoded MsgPackEntry string into a MessagePack-encoded buffer
///
/// # Examples 
/// 
/// ```
/// let json = r###"
/// {
///     "raw_marker": 195,
///     "basic_type": "Bool",
///     "data": {
///         "type": "Bool",
///         "value": true
///     }
/// }
/// "###;
/// 
/// let vec = rmpp::pack_json(json);
/// assert_eq!(vec![0xC3], vec);
/// ```
#[wasm_bindgen]
pub fn pack_json(json: &str) -> Vec<u8> {
    let mut buffer: Vec<u8> = vec![];
    let entry: MsgPackEntry = serde_json::from_str(json).unwrap();
    write_value(&mut buffer, &entry.data).unwrap();
    buffer
}

/// Turns a MsgPackEntry object into a MessagePack-encoded buffer
///
/// # Examples 
/// 
/// ```
/// let entry = rmpp::MsgPackEntry::new(
///     195, rmpp::MsgPackValue::Bool(true)
/// );
/// 
/// let vec = rmpp::pack(&entry);
/// assert_eq!(vec![0xC3], vec);
/// ```
pub fn pack(entry: &MsgPackEntry) -> Vec<u8> {
    let mut buffer: Vec<u8> = vec![];
    write_value(&mut buffer, &entry.data).unwrap();
    buffer
}

/// Serializes and writes a MsgValue-enabled object to a given buffer
/// 
/// It's pretty trivial under the hood: 
///     it just writes a marker, length if any and then the data if any
/// 
/// # Examples 
/// 
/// ```
/// let mut buffer: Vec<u8> = vec![];
/// let value = rmpp::MsgPackValue::Bool(true);
/// rmpp::write_value(&mut buffer, &value);
/// assert_eq!(vec![0xC3], buffer);
/// ```
pub fn write_value<W: Write, V: MsgValue>(writer: &mut W, value: &V) -> std::io::Result<()> {
    match value.get_value() {
        // Null
        MsgPackValue::Null => {
            writer.write_all(&[0xC0])?;
        },
        // Bool
        MsgPackValue::Bool(b) => {
            writer.write_all(&[if !*b { 0xC2 } else { 0xC3 }])?;
        },
        // Fixed Integer
        MsgPackValue::FixPos(n) => {
            writer.write_all(&[(*n) & 0b0111_1111])?; // Lower 7 bits represent the value
        },
        MsgPackValue::FixNeg(n) => {
            // Preserving the signature is important
            writer.write_all(&[(*n as u8) & 0b0001_1111 | 0b1110_0000])?; // Lower 5 bits represent the value
        },
        // Unsigned Integer
        MsgPackValue::U8(n) => {
            writer.write_all(&[0xCC])?;
            writer.write_all(&[*n])?;
        },
        MsgPackValue::U16(n) => {
            writer.write_all(&[0xCD])?;
            writer.write_all(&(*n).to_be_bytes())?;
        },
        MsgPackValue::U32(n) => {
            writer.write_all(&[0xCE])?;
            writer.write_all(&(*n).to_be_bytes())?;
        },
        MsgPackValue::U64(n) => {
            writer.write_all(&[0xCF])?;
            writer.write_all(&(*n).to_be_bytes())?;
        },
        // Signed Integer
        MsgPackValue::I8(n) => {
            writer.write_all(&[0xD0])?;
            writer.write_all(&[*n as u8])?;
        },
        MsgPackValue::I16(n) => {
            writer.write_all(&[0xD1])?;
            writer.write_all(&(*n).to_be_bytes())?;
        },
        MsgPackValue::I32(n) => {
            writer.write_all(&[0xD2])?;
            writer.write_all(&(*n).to_be_bytes())?;
        },
        MsgPackValue::I64(n) => {
            writer.write_all(&[0xD3])?;
            writer.write_all(&(*n).to_be_bytes())?;
        },
        // Float
        MsgPackValue::F32(n) => {
            writer.write_all(&[0xCA])?;
            writer.write_all(&(*n).to_be_bytes())?;
        },
        MsgPackValue::F64(n) => {
            writer.write_all(&[0xCB])?;
            writer.write_all(&(*n).to_be_bytes())?;
        },
        // String
        MsgPackValue::FixStr(s) => {
            let bytes = s.as_bytes();
            // Preserving the signature is important
            writer.write_all(&[(bytes.len() as u8) & 0b0001_1111 | 0b1010_0000])?; // Lower 5 bits represent the length
            writer.write_all(&bytes)?;
        },
        MsgPackValue::Str8(s) => {
            let bytes = s.as_bytes();
            writer.write_all(&[0xD9])?;
            writer.write_all(&[bytes.len() as u8])?;
            writer.write_all(&bytes)?;
        },
        MsgPackValue::Str16(s) => {
            let bytes = s.as_bytes();
            writer.write_all(&[0xDA])?;
            writer.write_all(&(bytes.len() as u16).to_be_bytes())?;
            writer.write_all(&bytes)?;
        },
        MsgPackValue::Str32(s) => {
            let bytes = s.as_bytes();
            writer.write_all(&[0xDB])?;
            writer.write_all(&(bytes.len() as u32).to_be_bytes())?;
            writer.write_all(&bytes)?;
        },
        // Binary
        MsgPackValue::Bin8(b) => {
            writer.write_all(&[0xC4])?;
            writer.write_all(&[b.len() as u8])?;
            writer.write_all(&b)?;
        },
        MsgPackValue::Bin16(b) => {
            writer.write_all(&[0xC5])?;
            writer.write_all(&(b.len() as u16).to_be_bytes())?;
            writer.write_all(&b)?;
        },
        MsgPackValue::Bin32(b) => {
            writer.write_all(&[0xC6])?;
            writer.write_all(&(b.len() as u32).to_be_bytes())?;
            writer.write_all(&b)?;
        },
        // Array
        MsgPackValue::FixArray(values) => {
            // Preserving the signature is important
            writer.write_all(&[(values.len() as u8) & 0b0000_1111 | 0b1001_0000])?; // Lower 4 bits represent the length

            // Recursively write each element
            for v in values {
                write_value(writer, &v.data)?;
            }
        },
        MsgPackValue::Array16(values) => {
            writer.write_all(&[0xDC])?;
            writer.write_all(&(values.len() as u16).to_be_bytes())?;

            // Recursively write each element
            for v in values {
                write_value(writer, &v.data)?;
            }
        },
        MsgPackValue::Array32(values) => {
            writer.write_all(&[0xDD])?;
            writer.write_all(&(values.len() as u32).to_be_bytes())?;

            // Recursively write each element
            for v in values {
                write_value(writer, &v.data)?;
            }
        },
        // Map
        MsgPackValue::FixMap(values) => {
            // Preserving the signature is important
            writer.write_all(&[(values.len() as u8) & 0b0000_1111 | 0b1000_0000])?; // Lower 4 bits represent the length

             // Recursively write each element
             for (k, v) in values {
                write_value(writer, &k.data)?;
                write_value(writer, &v.data)?;
            }
        },
        MsgPackValue::Map16(values) => {
            writer.write_all(&[0xDE])?;
            writer.write_all(&(values.len() as u16).to_be_bytes())?;
            
            // Recursively write each element
            for (k, v) in values {
                write_value(writer, &k.data)?;
                write_value(writer, &v.data)?;
            }
        },
        MsgPackValue::Map32(values) => {
            writer.write_all(&[0xDF])?;
            writer.write_all(&(values.len() as u32).to_be_bytes())?;
            
            // Recursively write each element
            for (k, v) in values {
                write_value(writer, &k.data)?;
                write_value(writer, &v.data)?;
            }
        }
    }
    
    Ok(())
}