use anyhow::{bail, Result};

use crate::cfgbin::TextEntry;

const MAGIC: u32 = 0x4E424452;

fn read_u32(d: &[u8], p: usize) -> u32 {
    u32::from_le_bytes([d[p], d[p+1], d[p+2], d[p+3]])
}
fn read_i32(d: &[u8], p: usize) -> i32 {
    i32::from_le_bytes([d[p], d[p+1], d[p+2], d[p+3]])
}
fn read_i16(d: &[u8], p: usize) -> i16 {
    i16::from_le_bytes([d[p], d[p+1]])
}
fn read_f32(d: &[u8], p: usize) -> f32 {
    f32::from_le_bytes([d[p], d[p+1], d[p+2], d[p+3]])
}

pub fn is_rdbn(data: &[u8]) -> bool {
    data.len() >= 4 && read_u32(data, 0) == MAGIC
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RdbnFieldType {
    AbilityData = 0,
    EnhanceData = 1,
    StatusRate  = 2,
    Bool        = 3,
    Byte        = 4,
    Short       = 5,
    Int         = 6,
    ActType     = 9,
    Flag        = 10,
    Float       = 13,
    Hash        = 15,
    RateMatrix  = 18,
    Position    = 19,
    String      = 20,
    DataTuple   = 21,
}

impl RdbnFieldType {
    fn from_i16(v: i16) -> Option<Self> {
        match v {
            0  => Some(Self::AbilityData),
            1  => Some(Self::EnhanceData),
            2  => Some(Self::StatusRate),
            3  => Some(Self::Bool),
            4  => Some(Self::Byte),
            5  => Some(Self::Short),
            6  => Some(Self::Int),
            9  => Some(Self::ActType),
            10 => Some(Self::Flag),
            13 => Some(Self::Float),
            15 => Some(Self::Hash),
            18 => Some(Self::RateMatrix),
            19 => Some(Self::Position),
            20 => Some(Self::String),
            21 => Some(Self::DataTuple),
            _  => None,
        }
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            Self::AbilityData => "ability_data",
            Self::EnhanceData => "enhance_data",
            Self::StatusRate  => "status_rate",
            Self::Bool        => "bool",
            Self::Byte        => "byte",
            Self::Short       => "short",
            Self::Int         => "int",
            Self::ActType     => "act_type",
            Self::Flag        => "flag",
            Self::Float       => "float",
            Self::Hash        => "hash",
            Self::RateMatrix  => "rate_matrix",
            Self::Position    => "position",
            Self::String      => "string",
            Self::DataTuple   => "data_tuple",
        }
    }
}

#[derive(Debug, Clone)]
pub enum RdbnValue {
    Bytes(Vec<u8>),
    Bool(bool),
    Byte(u8),
    Short(i16),
    Int(i32),
    Float(f32),
    Hash(u32),
    Flag(u32),
    Floats(Vec<f32>),
    String(Option<String>),
    Shorts(Vec<i16>),
}

impl RdbnValue {
    pub fn to_string_repr(&self) -> String {
        match self {
            Self::Bytes(b)   => b.iter().map(|x| format!("{:02X}", x)).collect::<Vec<_>>().join(" "),
            Self::Bool(v)    => v.to_string(),
            Self::Byte(v)    => v.to_string(),
            Self::Short(v)   => v.to_string(),
            Self::Int(v)     => v.to_string(),
            Self::Float(v)   => v.to_string(),
            Self::Hash(v)    => format!("0x{:08X}", v),
            Self::Flag(v)    => format!("0x{:08X}", v),
            Self::Floats(vs) => vs.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(","),
            Self::String(s)  => s.clone().unwrap_or_default(),
            Self::Shorts(vs) => vs.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(","),
        }
    }

    pub fn from_string_repr(s: &str, field_type: RdbnFieldType) -> Option<Self> {
        match field_type {
            RdbnFieldType::Bool        => Some(Self::Bool(s == "true" || s == "1")),
            RdbnFieldType::Byte        => s.parse::<u8>().ok().map(Self::Byte),
            RdbnFieldType::Short | RdbnFieldType::ActType => s.parse::<i16>().ok().map(Self::Short),
            RdbnFieldType::Int         => s.parse::<i32>().ok().map(Self::Int),
            RdbnFieldType::Float       => s.parse::<f32>().ok().map(Self::Float),
            RdbnFieldType::Hash | RdbnFieldType::Flag => {
                let trimmed = s.trim_start_matches("0x").trim_start_matches("0X");
                u32::from_str_radix(trimmed, 16).ok().or_else(|| s.parse::<u32>().ok()).map(|v| {
                    if field_type == RdbnFieldType::Hash { Self::Hash(v) } else { Self::Flag(v) }
                })
            }
            RdbnFieldType::RateMatrix | RdbnFieldType::Position => {
                let parts: Vec<f32> = s.split(',').filter_map(|x| x.trim().parse().ok()).collect();
                if parts.is_empty() { None } else { Some(Self::Floats(parts)) }
            }
            RdbnFieldType::String => Some(Self::String(if s.is_empty() { None } else { Some(s.to_string()) })),
            RdbnFieldType::DataTuple => {
                let parts: Vec<i16> = s.split(',').filter_map(|x| x.trim().parse().ok()).collect();
                if parts.is_empty() { None } else { Some(Self::Shorts(parts)) }
            }
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RdbnField {
    pub name_hash: u32,
    pub field_type: RdbnFieldType,
    pub value_size: i32,
    pub value_count: i32,
}

#[derive(Debug, Clone)]
pub struct RdbnType {
    #[allow(dead_code)]
    pub name_hash: u32,
    pub fields: Vec<RdbnField>,
}

#[derive(Debug, Clone)]
pub struct RdbnInstance {
    pub values: Vec<Vec<RdbnValue>>,
}

#[derive(Debug, Clone)]
pub struct RdbnRoot {
    pub name_hash: u32,
    pub type_index: usize,
    pub instances: Vec<RdbnInstance>,
}

pub struct RdbnFile {
    #[allow(dead_code)]
    pub version: i32,
    pub types: Vec<RdbnType>,
    pub roots: Vec<RdbnRoot>,
    raw: Vec<u8>,
}

struct Header {
    #[allow(dead_code)]
    version: i32,
    type_offset: usize,
    type_count: usize,
    field_offset: usize,
    root_offset: usize,
    root_count: usize,
    #[allow(dead_code)]
    value_offset: usize,
    string_offset: usize,
}

fn parse_header(data: &[u8]) -> Result<Header> {
    if data.len() < 0x3C {
        bail!("RDBN file too small");
    }
    let magic = read_u32(data, 0);
    if magic != MAGIC {
        bail!("Not an RDBN file");
    }

    let version      = read_i32(data, 6);
    let data_base    = (read_i16(data, 10) as usize) << 2;
    let type_offset  = data_base + ((read_i16(data, 0x24) as usize) << 2);
    let type_count   = read_i16(data, 0x26) as usize;
    let field_offset = data_base + ((read_i16(data, 0x28) as usize) << 2);
    let root_offset  = data_base + ((read_i16(data, 0x2C) as usize) << 2);
    let root_count   = read_i16(data, 0x2E) as usize;
    let value_offset = data_base + ((read_i16(data, 0x36) as usize) << 2);
    let string_offset = read_i32(data, 0x38) as usize;

    Ok(Header {
        version,
        type_offset,
        type_count,
        field_offset,
        root_offset,
        root_count,
        value_offset,
        string_offset,
    })
}

fn read_string_table(data: &[u8], string_offset: usize) -> Vec<u8> {
    if string_offset >= data.len() { return vec![]; }
    data[string_offset..].to_vec()
}

fn read_cstring(table: &[u8], offset: usize) -> Option<String> {
    if offset >= table.len() { return None; }
    let end = table[offset..].iter().position(|&b| b == 0).unwrap_or(table.len() - offset);
    String::from_utf8(table[offset..offset + end].to_vec()).ok()
}

impl RdbnFile {
    pub fn open(data: &[u8]) -> Result<Self> {
        let h = parse_header(data)?;
        let types = parse_types(data, &h)?;
        let roots = parse_roots(data, &h, &types)?;

        Ok(RdbnFile {
            version: h.version,
            types,
            roots,
            raw: data.to_vec(),
        })
    }

    pub fn extract_fields(&self) -> Vec<TextEntry> {
        let mut out = Vec::new();
        let mut global_index = 0usize;

        for root in &self.roots {
            if root.type_index >= self.types.len() { continue; }
            let rdbn_type = &self.types[root.type_index];

            for instance in &root.instances {
                for (field_idx, field_values) in instance.values.iter().enumerate() {
                    if field_idx >= rdbn_type.fields.len() { continue; }
                    let field = &rdbn_type.fields[field_idx];

                    for value in field_values {
                        let field_type = field.field_type.type_name().to_string();
                        let value_str = value.to_string_repr();
                        let entry_name = format!("0x{:08X}", root.name_hash);

                        out.push(TextEntry {
                            index: global_index,
                            entry: entry_name,
                            variable_index: field_idx,
                            field_type,
                            value: value_str,
                        });
                        global_index += 1;
                    }
                }
            }
        }
        out
    }

    pub fn update_fields(&mut self, entries: &[TextEntry]) {
        let mut global_index = 0usize;

        for root in &mut self.roots {
            for instance in &mut root.instances {
                for field_values in &mut instance.values {
                    for value in field_values.iter_mut() {
                        if let Some(te) = entries.iter().find(|t| t.index == global_index) {
                            let field_type = rdbn_field_type_from_name(&te.field_type);
                            if let Some(ft) = field_type {
                                if let Some(new_val) = RdbnValue::from_string_repr(&te.value, ft) {
                                    *value = new_val;
                                }
                            }
                        }
                        global_index += 1;
                    }
                }
            }
        }
    }

    pub fn save(&self) -> Vec<u8> {
        let h = match parse_header(&self.raw) {
            Ok(h) => h,
            Err(_) => return self.raw.clone(),
        };

        let mut out = self.raw.clone();
        let str_table = read_string_table(&self.raw, h.string_offset);

        for (root_idx, root) in self.roots.iter().enumerate() {
            let root_entry_pos = h.root_offset + root_idx * 0x20;
            if root_entry_pos + 0x10 > self.raw.len() { break; }

            let value_base = (read_i32(&self.raw, root_entry_pos + 4) as usize) << 2;
            let instance_size = read_i32(&self.raw, root_entry_pos + 8) as usize;

            if root.type_index >= self.types.len() { continue; }
            let rdbn_type = &self.types[root.type_index];

            for (inst_idx, instance) in root.instances.iter().enumerate() {
                let inst_base = value_base + inst_idx * instance_size;

                for (field_idx, field_values) in instance.values.iter().enumerate() {
                    if field_idx >= rdbn_type.fields.len() { continue; }
                    let field = &rdbn_type.fields[field_idx];

                    let field_entry_base = h.field_offset + (get_field_start(&self.types[root.type_index], field_idx)) * 0x20;
                    if field_entry_base + 0x14 > self.raw.len() {
                        continue;
                    }

                    let field_val_offset = (read_i32(&self.raw, field_entry_base + 0x10) as usize) << 2;
                    let field_start = inst_base + field_val_offset;

                    for (sub_idx, value) in field_values.iter().enumerate() {
                        let byte_pos = field_start + sub_idx * (field.value_size as usize);
                        write_value(&mut out, byte_pos, value, field.field_type, &str_table, h.string_offset);
                    }
                }
            }
        }

        out
    }
}

fn get_field_start(_rdbn_type: &RdbnType, field_idx: usize) -> usize {
    field_idx
}

fn write_value(out: &mut Vec<u8>, pos: usize, value: &RdbnValue, ft: RdbnFieldType, _str_table: &[u8], _str_base: usize) {
    if pos >= out.len() { return; }

    match (value, ft) {
        (RdbnValue::Bool(v), _)    => { if pos < out.len() { out[pos] = if *v { 1 } else { 0 }; } }
        (RdbnValue::Byte(v), _)    => { if pos < out.len() { out[pos] = *v; } }
        (RdbnValue::Short(v), _)   => { if pos + 2 <= out.len() { out[pos..pos+2].copy_from_slice(&v.to_le_bytes()); } }
        (RdbnValue::Int(v), _)     => { if pos + 4 <= out.len() { out[pos..pos+4].copy_from_slice(&v.to_le_bytes()); } }
        (RdbnValue::Float(v), _)   => { if pos + 4 <= out.len() { out[pos..pos+4].copy_from_slice(&v.to_le_bytes()); } }
        (RdbnValue::Hash(v), _)    => { if pos + 4 <= out.len() { out[pos..pos+4].copy_from_slice(&v.to_le_bytes()); } }
        (RdbnValue::Flag(v), _)    => { if pos + 4 <= out.len() { out[pos..pos+4].copy_from_slice(&v.to_le_bytes()); } }
        (RdbnValue::Floats(vs), _) => {
            for (i, f) in vs.iter().enumerate() {
                let p = pos + i * 4;
                if p + 4 <= out.len() { out[p..p+4].copy_from_slice(&f.to_le_bytes()); }
            }
        }
        (RdbnValue::Shorts(vs), _) => {
            for (i, s) in vs.iter().enumerate() {
                let p = pos + i * 2;
                if p + 2 <= out.len() { out[p..p+2].copy_from_slice(&s.to_le_bytes()); }
            }
        }
        _ => {}
    }
}

fn rdbn_field_type_from_name(name: &str) -> Option<RdbnFieldType> {
    match name {
        "ability_data" => Some(RdbnFieldType::AbilityData),
        "enhance_data" => Some(RdbnFieldType::EnhanceData),
        "status_rate"  => Some(RdbnFieldType::StatusRate),
        "bool"         => Some(RdbnFieldType::Bool),
        "byte"         => Some(RdbnFieldType::Byte),
        "short"        => Some(RdbnFieldType::Short),
        "int"          => Some(RdbnFieldType::Int),
        "act_type"     => Some(RdbnFieldType::ActType),
        "flag"         => Some(RdbnFieldType::Flag),
        "float"        => Some(RdbnFieldType::Float),
        "hash"         => Some(RdbnFieldType::Hash),
        "rate_matrix"  => Some(RdbnFieldType::RateMatrix),
        "position"     => Some(RdbnFieldType::Position),
        "string"       => Some(RdbnFieldType::String),
        "data_tuple"   => Some(RdbnFieldType::DataTuple),
        _              => None,
    }
}

fn parse_types(data: &[u8], h: &Header) -> Result<Vec<RdbnType>> {
    let mut types = Vec::with_capacity(h.type_count);

    for i in 0..h.type_count {
        let pos = h.type_offset + i * 0x20;
        if pos + 8 > data.len() { bail!("Type entry out of bounds"); }

        let name_hash   = read_u32(data, pos);
        let field_index = read_i16(data, pos + 8) as usize;
        let field_count = read_i16(data, pos + 10) as usize;

        let mut fields = Vec::with_capacity(field_count);
        for j in 0..field_count {
            let fpos = h.field_offset + (field_index + j) * 0x20;
            if fpos + 0x18 > data.len() { bail!("Field entry out of bounds"); }

            let fname_hash   = read_u32(data, fpos);
            let ftype_raw    = read_i16(data, fpos + 4);
            let value_size   = read_i32(data, fpos + 8);
            let value_count  = read_i32(data, fpos + 0x14);

            let field_type = RdbnFieldType::from_i16(ftype_raw)
                .unwrap_or(RdbnFieldType::Int);

            fields.push(RdbnField {
                name_hash: fname_hash,
                field_type,
                value_size,
                value_count,
            });
        }

        types.push(RdbnType { name_hash, fields });
    }

    Ok(types)
}

fn parse_roots(data: &[u8], h: &Header, types: &[RdbnType]) -> Result<Vec<RdbnRoot>> {
    let mut roots = Vec::with_capacity(h.root_count);

    for i in 0..h.root_count {
        let pos = h.root_offset + i * 0x20;
        if pos + 0x1C > data.len() { bail!("Root entry out of bounds"); }

        let type_index    = read_i16(data, pos) as usize;
        let value_offset  = (read_i32(data, pos + 4) as usize) << 2;
        let instance_size = read_i32(data, pos + 8) as usize;
        let instance_count = read_i32(data, pos + 12) as usize;
        let name_hash     = read_u32(data, pos + 16);

        let rdbn_type = types.get(type_index)
            .ok_or_else(|| anyhow::anyhow!("Type index {} out of range", type_index))?;

        let mut instances = Vec::with_capacity(instance_count);
        for inst_idx in 0..instance_count {
            let inst_base = value_offset + inst_idx * instance_size;
            let mut field_values_list = Vec::with_capacity(rdbn_type.fields.len());

            for field in &rdbn_type.fields {
                let field_entry_pos = h.field_offset + field_global_index(types, type_index, rdbn_type.fields.iter().position(|f| f.name_hash == field.name_hash).unwrap_or(0)) * 0x20;
                if field_entry_pos + 0x14 > data.len() {
                    field_values_list.push(vec![]);
                    continue;
                }

                let field_val_offset = (read_i32(data, field_entry_pos + 0x10) as usize) << 2;
                let field_start = inst_base + field_val_offset;
                let count = field.value_count.max(1) as usize;

                let values = read_field_values(data, field_start, field, count, h)?;
                field_values_list.push(values);
            }

            instances.push(RdbnInstance { values: field_values_list });
        }

        roots.push(RdbnRoot { name_hash, type_index, instances });
    }

    Ok(roots)
}

fn field_global_index(types: &[RdbnType], type_idx: usize, local_field_idx: usize) -> usize {
    let mut base = 0;
    for t in &types[..type_idx] {
        base += t.fields.len();
    }
    base + local_field_idx
}

fn read_field_values(data: &[u8], start: usize, field: &RdbnField, count: usize, h: &Header) -> Result<Vec<RdbnValue>> {
    let mut values = Vec::new();

    for i in 0..count {
        let pos = start + i * (field.value_size as usize);
        if pos > data.len() { break; }

        let value = match field.field_type {
            RdbnFieldType::Bool => {
                RdbnValue::Bool(data.get(pos).copied().unwrap_or(0) != 0)
            }
            RdbnFieldType::Byte => {
                RdbnValue::Byte(data.get(pos).copied().unwrap_or(0))
            }
            RdbnFieldType::Short | RdbnFieldType::ActType => {
                if pos + 2 > data.len() { break; }
                RdbnValue::Short(read_i16(data, pos))
            }
            RdbnFieldType::Int => {
                if pos + 4 > data.len() { break; }
                RdbnValue::Int(read_i32(data, pos))
            }
            RdbnFieldType::Float => {
                if pos + 4 > data.len() { break; }
                RdbnValue::Float(read_f32(data, pos))
            }
            RdbnFieldType::Hash | RdbnFieldType::Flag => {
                if pos + 4 > data.len() { break; }
                let v = read_u32(data, pos);
                if field.field_type == RdbnFieldType::Hash { RdbnValue::Hash(v) } else { RdbnValue::Flag(v) }
            }
            RdbnFieldType::RateMatrix | RdbnFieldType::Position => {
                if pos + 16 > data.len() { break; }
                let floats = (0..4).map(|k| read_f32(data, pos + k * 4)).collect();
                RdbnValue::Floats(floats)
            }
            RdbnFieldType::DataTuple => {
                if pos + 4 > data.len() { break; }
                let shorts = vec![read_i16(data, pos), read_i16(data, pos + 2)];
                RdbnValue::Shorts(shorts)
            }
            RdbnFieldType::String => {
                if pos + 4 > data.len() { break; }
                let str_offset = read_i32(data, pos);
                if str_offset < 0 {
                    RdbnValue::String(None)
                } else {
                    let str_data = &data[h.string_offset..];
                    let s = read_cstring(str_data, str_offset as usize);
                    RdbnValue::String(s)
                }
            }
            RdbnFieldType::AbilityData | RdbnFieldType::EnhanceData | RdbnFieldType::StatusRate => {
                let size = (field.value_size as usize).min(data.len().saturating_sub(pos));
                RdbnValue::Bytes(data[pos..pos + size].to_vec())
            }
        };

        values.push(value);
    }

    Ok(values)
}
