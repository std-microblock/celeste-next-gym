use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;

#[derive(Clone, Debug, PartialEq)]
pub enum BinaryValue {
    Bool(bool),
    Byte(u8),
    Short(i16),
    Int(i32),
    Float(f32),
    String(String),
}

#[derive(Clone, Debug, PartialEq)]
pub struct BinaryElement {
    pub package: Option<String>,
    pub name: String,
    pub attributes: BTreeMap<String, BinaryValue>,
    pub children: Vec<BinaryElement>,
}

#[derive(Debug, Error)]
pub enum BinaryPackerError {
    #[error("unexpected end of file")]
    Eof,
    #[error("invalid UTF-8 string")]
    Utf8,
    #[error("invalid string table index {0}")]
    StringIndex(i16),
    #[error("unsupported BinaryPacker value type {0}")]
    ValueType(u8),
    #[error("invalid BinaryPacker header {0:?}")]
    Header(String),
    #[error("invalid length or recursion limit")]
    Limit,
    #[error("invalid RLE payload")]
    Rle,
}

#[derive(Debug, Error, PartialEq)]
pub enum BinaryPackerWriteError {
    #[error("root element is missing its package name")]
    MissingPackage,
    #[error("BinaryPacker string, attribute, or child count exceeds the format limit")]
    Limit,
    #[error("BinaryPacker element or attribute name is absent from the string table")]
    MissingString,
}

struct Reader<'a> {
    bytes: &'a [u8],
    pos: usize,
}
impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], BinaryPackerError> {
        let end = self.pos.checked_add(n).ok_or(BinaryPackerError::Limit)?;
        let v = self
            .bytes
            .get(self.pos..end)
            .ok_or(BinaryPackerError::Eof)?;
        self.pos = end;
        Ok(v)
    }
    fn u8(&mut self) -> Result<u8, BinaryPackerError> {
        Ok(self.take(1)?[0])
    }
    fn i16(&mut self) -> Result<i16, BinaryPackerError> {
        Ok(i16::from_le_bytes(self.take(2)?.try_into().unwrap()))
    }
    fn i32(&mut self) -> Result<i32, BinaryPackerError> {
        Ok(i32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn f32(&mut self) -> Result<f32, BinaryPackerError> {
        Ok(f32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn string(&mut self) -> Result<String, BinaryPackerError> {
        let mut len = 0usize;
        for shift in (0..35).step_by(7) {
            let b = self.u8()?;
            len |= ((b & 0x7f) as usize) << shift;
            if b & 0x80 == 0 {
                if len > 16 * 1024 * 1024 {
                    return Err(BinaryPackerError::Limit);
                }
                return String::from_utf8(self.take(len)?.to_vec())
                    .map_err(|_| BinaryPackerError::Utf8);
            }
        }
        Err(BinaryPackerError::Limit)
    }
}

pub fn parse_celeste_bin(bytes: &[u8]) -> Result<BinaryElement, BinaryPackerError> {
    let mut r = Reader { bytes, pos: 0 };
    let header = r.string()?;
    if header != "CELESTE MAP" {
        return Err(BinaryPackerError::Header(header));
    }
    let package = r.string()?;
    let count = r.i16()?;
    if !(0..=32767).contains(&count) {
        return Err(BinaryPackerError::Limit);
    }
    let mut lookup = Vec::with_capacity(count as usize);
    for _ in 0..count {
        lookup.push(r.string()?);
    }
    let mut root = read_element(&mut r, &lookup, 0)?;
    root.package = Some(package);
    Ok(root)
}

pub fn encode_celeste_bin(root: &BinaryElement) -> Result<Vec<u8>, BinaryPackerWriteError> {
    let package = root
        .package
        .as_deref()
        .ok_or(BinaryPackerWriteError::MissingPackage)?;
    let mut strings = BTreeSet::new();
    collect_strings(root, &mut strings);
    if strings.len() > i16::MAX as usize {
        return Err(BinaryPackerWriteError::Limit);
    }
    let table: Vec<String> = strings.into_iter().collect();
    let lookup: BTreeMap<&str, i16> = table
        .iter()
        .enumerate()
        .map(|(index, value)| (value.as_str(), index as i16))
        .collect();

    let mut out = Vec::new();
    write_string(&mut out, "CELESTE MAP")?;
    write_string(&mut out, package)?;
    out.extend_from_slice(&(table.len() as i16).to_le_bytes());
    for value in &table {
        write_string(&mut out, value)?;
    }
    write_element(&mut out, root, &lookup)?;
    Ok(out)
}

fn collect_strings(element: &BinaryElement, strings: &mut BTreeSet<String>) {
    strings.insert(element.name.clone());
    strings.extend(element.attributes.keys().cloned());
    for child in &element.children {
        collect_strings(child, strings);
    }
}

fn write_element(
    out: &mut Vec<u8>,
    element: &BinaryElement,
    lookup: &BTreeMap<&str, i16>,
) -> Result<(), BinaryPackerWriteError> {
    let name = lookup
        .get(element.name.as_str())
        .ok_or(BinaryPackerWriteError::MissingString)?;
    out.extend_from_slice(&name.to_le_bytes());
    let attribute_count =
        u8::try_from(element.attributes.len()).map_err(|_| BinaryPackerWriteError::Limit)?;
    out.push(attribute_count);
    for (key, value) in &element.attributes {
        let key = lookup
            .get(key.as_str())
            .ok_or(BinaryPackerWriteError::MissingString)?;
        out.extend_from_slice(&key.to_le_bytes());
        match value {
            BinaryValue::Bool(value) => {
                out.push(0);
                out.push(u8::from(*value));
            }
            BinaryValue::Byte(value) => {
                out.push(1);
                out.push(*value);
            }
            BinaryValue::Short(value) => {
                out.push(2);
                out.extend_from_slice(&value.to_le_bytes());
            }
            BinaryValue::Int(value) => {
                out.push(3);
                out.extend_from_slice(&value.to_le_bytes());
            }
            BinaryValue::Float(value) => {
                out.push(4);
                out.extend_from_slice(&value.to_le_bytes());
            }
            BinaryValue::String(value) => {
                out.push(6);
                write_string(out, value)?;
            }
        }
    }
    let child_count =
        i16::try_from(element.children.len()).map_err(|_| BinaryPackerWriteError::Limit)?;
    out.extend_from_slice(&child_count.to_le_bytes());
    for child in &element.children {
        write_element(out, child, lookup)?;
    }
    Ok(())
}

fn write_string(out: &mut Vec<u8>, value: &str) -> Result<(), BinaryPackerWriteError> {
    let mut length = u32::try_from(value.len()).map_err(|_| BinaryPackerWriteError::Limit)?;
    while length >= 0x80 {
        out.push((length as u8) | 0x80);
        length >>= 7;
    }
    out.push(length as u8);
    out.extend_from_slice(value.as_bytes());
    Ok(())
}

fn lookup(table: &[String], index: i16) -> Result<String, BinaryPackerError> {
    table
        .get(index as usize)
        .cloned()
        .ok_or(BinaryPackerError::StringIndex(index))
}

fn read_element(
    r: &mut Reader<'_>,
    table: &[String],
    depth: usize,
) -> Result<BinaryElement, BinaryPackerError> {
    if depth > 256 {
        return Err(BinaryPackerError::Limit);
    }
    let name = lookup(table, r.i16()?)?;
    let attr_count = r.u8()?;
    let mut attributes = BTreeMap::new();
    for _ in 0..attr_count {
        let key = lookup(table, r.i16()?)?;
        let ty = r.u8()?;
        let value = match ty {
            0 => BinaryValue::Bool(r.u8()? != 0),
            1 => BinaryValue::Byte(r.u8()?),
            2 => BinaryValue::Short(r.i16()?),
            3 => BinaryValue::Int(r.i32()?),
            4 => BinaryValue::Float(r.f32()?),
            5 => BinaryValue::String(lookup(table, r.i16()?)?),
            6 => BinaryValue::String(r.string()?),
            7 => {
                let len = r.i16()?;
                if len < 0 || len % 2 != 0 {
                    return Err(BinaryPackerError::Rle);
                }
                let data = r.take(len as usize)?;
                let mut s = String::new();
                for pair in data.chunks_exact(2) {
                    s.extend(std::iter::repeat_n(pair[1] as char, pair[0] as usize));
                }
                BinaryValue::String(s)
            }
            other => return Err(BinaryPackerError::ValueType(other)),
        };
        attributes.insert(key, value);
    }
    let child_count = r.i16()?;
    if child_count < 0 {
        return Err(BinaryPackerError::Limit);
    }
    let mut children = Vec::with_capacity(child_count as usize);
    for _ in 0..child_count {
        children.push(read_element(r, table, depth + 1)?);
    }
    Ok(BinaryElement {
        package: None,
        name,
        attributes,
        children,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_non_map() {
        assert!(matches!(
            parse_celeste_bin(&[3, b'f', b'o', b'o']),
            Err(BinaryPackerError::Header(_))
        ));
    }

    #[test]
    fn binary_packer_round_trip() {
        let root = BinaryElement {
            package: Some("TestMap".to_owned()),
            name: "Map".to_owned(),
            attributes: BTreeMap::from([
                ("enabled".to_owned(), BinaryValue::Bool(true)),
                ("title".to_owned(), BinaryValue::String("雪山".to_owned())),
            ]),
            children: vec![BinaryElement {
                package: None,
                name: "levels".to_owned(),
                attributes: BTreeMap::new(),
                children: vec![],
            }],
        };
        let bytes = encode_celeste_bin(&root).unwrap();
        assert_eq!(parse_celeste_bin(&bytes).unwrap(), root);
    }
}
