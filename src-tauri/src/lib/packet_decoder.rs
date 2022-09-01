#![allow(dead_code, unused_variables, non_snake_case)]

use bytebuffer::ByteBuffer;
use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use std::{cmp::min, collections::HashMap, fs, sync::atomic};

pub const PRIMITIVES: [&str; 17] = [
    "Boolean",
    "Byte",
    "ByteArray",
    "Double",
    "Float",
    "Int",
    "Short",
    "UTF",
    "UnsignedByte",
    "UnsignedInt",
    "UnsignedShort",
    "VarInt",
    "VarLong",
    "VarShort",
    "VarUhInt",
    "VarUhLong",
    "VarUhShort",
];

struct UInt64 {
    low: usize,
    high: usize,
}

impl UInt64 {
    fn to_number(&self) -> usize {
        self.high * 4294967296 + self.low
    }
}

trait Buffer {
    fn bytes_available(&self) -> usize {
        0 as usize
    }
    fn swap_bytes(&mut self, bytes: &mut ByteBuffer, len: usize) {}
    fn read(&mut self, t: &str) -> AtomicType {
        AtomicType::Boolean(true)
    }
    fn read_var_int(&mut self) -> u32 {
        0
    }
    fn read_var_uh_int(&mut self) -> u32 {
        0
    }
    fn read_var_short(&mut self) -> i16 {
        0
    }
    fn read_var_uh_short(&mut self) -> u16 {
        0
    }
    fn read_uint_64(&mut self) -> UInt64 {
        UInt64 { low: 0, high: 0 }
    }
}

impl Buffer for ByteBuffer {
    fn bytes_available(&self) -> usize {
        return self.len() - self.get_rpos();
    }
    fn swap_bytes(&mut self, bytes: &mut ByteBuffer, len: usize) {
        let mut length = len.clone();
        if length == 0 {
            length = self.bytes_available();
        }

        if length > self.bytes_available() {
            println!("End of buffer was encountered.");
            return;
        }

        bytes.write_bytes(&self.read_bytes(length));

        self.set_rpos(min(self.get_rpos() + length, self.len()));
    }
    fn read_var_int(&mut self) -> u32 {
        const INT_SIZE: isize = 32;

        const UNSIGNED_SHORT_MAX_VALUE: isize = 65536;

        const CHUNCK_BIT_SIZE: isize = 7;

        const MASK_10000000: isize = 128;

        const MASK_01111111: isize = 127;

        let mut b: isize;
        let mut value: isize = 0;
        let mut offset: isize = 0;
        let mut hasNext: bool;
        while offset < INT_SIZE {
            b = self.read_i8() as isize;
            hasNext = (b & MASK_10000000) == MASK_10000000;
            if offset > 0 {
                value += (b & MASK_01111111) << offset;
            } else {
                value += b & MASK_01111111;
            }
            offset += CHUNCK_BIT_SIZE;
            if !hasNext {
                return value.try_into().expect("read_var_int failed to try to i32");
            }
        }
        println!("Too much data, {}", value);
        value
            .try_into()
            .expect("read_var_int failed to try to i32 - Too much data")
    }
    fn read_var_uh_int(&mut self) -> u32 {
        self.read_var_int()
            .try_into()
            .expect("read_var_uh_int failed to try to u32")
    }
    fn read_var_short(&mut self) -> i16 {
        const SHORT_MAX_VALUE: isize = 32767;

        const UNSIGNED_SHORT_MAX_VALUE: isize = 65536;

        const CHUNCK_BIT_SIZE: isize = 7;

        const MASK_10000000: isize = 128;

        const MASK_01111111: isize = 127;

        let mut b: isize;
        let mut value: isize = 0;
        let mut offset: isize = 0;
        let mut hasNext: bool;
        while offset < 16 {
            b = self.read_i8() as isize;
            hasNext = (b & MASK_10000000) == MASK_10000000;
            if offset > 0 {
                value += (b & MASK_01111111) << offset;
            } else {
                value += b & MASK_01111111;
            }
            offset += CHUNCK_BIT_SIZE;
            if !hasNext {
                if value > SHORT_MAX_VALUE {
                    value -= UNSIGNED_SHORT_MAX_VALUE;
                }
                return value
                    .try_into()
                    .expect("read_var_short failed to try to i16");
            }
        }
        value
            .try_into()
            .expect("read_var_short failed to try to i16 - Too much data")
    }
    fn read_var_uh_short(&mut self) -> u16 {
        self.read_var_short()
            .try_into()
            .expect("read_var_uh_short failed to try to u16")
    }
    fn read_uint_64(&mut self) -> UInt64 {
        let mut b: usize;
        //  let result:u64 = u64::from_be(0); //= new UInt64();
        let mut result = UInt64 { low: 0, high: 0 };

        let mut i: usize = 0;
        loop {
            b = self.read_u8().try_into().unwrap();
            if i == 28 {
                break;
            }
            if b < 128 {
                result.low |= b << i;
                return result;
            }
            result.low |= (b & 127) << i;
            i += 7;
        }
        if b >= 128 {
            b &= 127;
            result.low |= b << i;
            result.high = b >> 4;
            i = 3;
            loop {
                b = self.read_u8().try_into().unwrap();
                if i < 32 {
                    if b < 128 {
                        break;
                    }
                    result.high |= (b & 127) << i;
                }
                i += 7;
            }
            result.high |= b << i;
            return result;
        }
        result.low |= b << i;
        result.high = b >> 4;
        return result;
    }
    fn read(&mut self, t: &str) -> AtomicType {
        match t {
            "UnsignedByte" => AtomicType::UnsignedByte(self.read_u8()),
            "Byte" => AtomicType::Byte(self.read_i8()),
            "UnsignedShort" => AtomicType::UnsignedShort(self.read_u16()),
            "Short" => AtomicType::Short(self.read_i16()),
            "Int" => AtomicType::Int(self.read_i32()),
            "Boolean" => {
                let b = self.read_i8();
                if b == 0 {
                    return AtomicType::Boolean(false);
                }
                AtomicType::Boolean(true)
            }
            "UTF" => AtomicType::UTF({
                let n_of_bytes = self.read_u16();
                let v = self.read_bytes(n_of_bytes.clone().try_into().expect("Error in readUTF"));
                std::str::from_utf8(&v)
                    .expect("Error in readUTF")
                    .to_string()
            }),
            "Double" => AtomicType::Double(self.read_f64().abs()),
            "VarUhLong" => AtomicType::VarUhLong({
                let var = self.read_uint_64().to_number();
                var.try_into().expect("test failed")
            }),
            "VarLong" => AtomicType::VarLong({
                let var = self.read_uint_64().to_number();
                var.try_into().expect("test failed")
            }),
            "VarUhInt" => AtomicType::VarUhInt(self.read_var_uh_int()),
            "VarInt" => AtomicType::VarInt(self.read_var_int()),
            "VarShort" => AtomicType::VarShort(self.read_var_short()),
            "VarUhShort" => AtomicType::VarUhShort(self.read_var_uh_short()),
            _ => {
                println!("{} type is not implemented", t);
                AtomicType::Boolean(false)
            }
        }
    }
}

#[derive(Debug)]
enum AtomicType {
    UnsignedByte(u8),
    Byte(i8),
    UnsignedShort(u16),
    Short(i16),
    Int(i32),
    Boolean(bool),
    UTF(String),
    Double(f64),
    VarUhLong(u64),
    VarLong(u64),
    VarUhInt(u32),
    VarInt(u32),
    VarShort(i16),
    VarUhShort(u16),
}

#[derive(Debug, Clone, Serialize)]
pub struct DofusPacket {
    source: String,
    time: String,
    id: u16,
    name: String,
    raw: String,
    body: Map<String, Value>,
}

impl DofusPacket {
    fn new(source: String, time: i64, id: u16, raw: String) -> DofusPacket {
        DofusPacket {
            source,
            time: time.to_string(),
            id,
            name: "".to_string(),
            raw,
            body: Map::new(),
        }
    }
}

pub struct PacketDecoder {
    sba: ByteBuffer,
    split_packet: bool,
    split_packet_id: u16,
    split_packet_length: usize,
    split_packet_port: u16,
    queue: Vec<DofusPacket>,
    msg_from_types: Map<String, Value>,
    types_from_id: Map<String, Value>,
    types: Map<String, Value>,
}

impl PacketDecoder {
    pub fn new() -> PacketDecoder {
        let data = fs::read_to_string("./src/utils/network-message/2.63/messages.json")
            .expect("Unable to open messages.json file");

        let json: HashMap<String, Value> =
            serde_json::from_str(&data).expect("Unable to JSON the file");

        PacketDecoder {
            sba: ByteBuffer::new(),
            split_packet: false,
            split_packet_id: 0,
            split_packet_length: 0,
            split_packet_port: 0,
            queue: Vec::<DofusPacket>::new(),
            msg_from_types: json
                .get("msg_from_id")
                .expect("Unable to get msg from types")
                .as_object()
                .unwrap()
                .clone(),
            types_from_id: json
                .get("types_from_id")
                .expect("Unable to get types from id")
                .as_object()
                .unwrap()
                .clone(),
            types: json
                .get("types")
                .expect("Unable to get types")
                .as_object()
                .unwrap()
                .clone(),
        }
    }

    pub fn decode_packet(&mut self, tcp_content: &[u8], port: u16) -> u32 {
        let t = tcp_content.clone();
        let mut ba = ByteBuffer::from_bytes(t);
        while ba.bytes_available() > 0 {
            if self.split_packet {
                println!(
                    "We're split, at {} + {}, looking for {}",
                    self.sba.len(),
                    ba.bytes_available(),
                    self.split_packet_length
                );

                if self.sba.len() + ba.bytes_available() < self.split_packet_length {
                    ba.swap_bytes(&mut self.sba, ba.bytes_available());
                } else {
                    let offset = self.sba.len().clone();
                    ba.swap_bytes(&mut self.sba, self.split_packet_length - offset);

                    self.sba.set_rpos(0);

                    let initial_pos = self.sba.get_rpos();

                    // Parse the message
                    let message_object = PacketDecoder::parse_ba_to_object(
                        &mut self.sba,
                        self.split_packet_id,
                        &self.msg_from_types,
                        &self.types_from_id,
                        &self.types,
                    );

                    match message_object {
                        Ok(obj) => self.queue.push(obj),
                        Err(err) => println!("{}", err),
                    }

                    let consumed = self.sba.get_rpos() - initial_pos;

                    if self.split_packet_length - consumed != 0 {
                        println!("warning: forced to trim a packet !");
                        ba.set_rpos(min(initial_pos + self.split_packet_length, self.sba.len()));
                    }

                    // reset
                    self.split_packet = false;
                    self.sba = ByteBuffer::new();
                    self.split_packet_id = 0;
                    self.split_packet_port = 0;
                }
            } else {
                if ba.bytes_available() < 2 {
                    println!("Empty packet");
                    return 0;
                }

                let hi_header = ba.read_u16();
                let packet_id = hi_header >> 2;
                let length_type = hi_header & 0b11;

                let mut length: usize = 0;
                let mut _instance_id = 0;

                if port != 5555 {
                    _instance_id = ba.read_u8();
                }

                let msg = self.msg_from_types.get(&packet_id.to_string());

                if let Some(message_type) = msg {
                    let message_type = message_type.as_object().unwrap();
                    let name = message_type
                        .get("name")
                        .expect("Message has no name")
                        .as_str()
                        .unwrap();
                } else {
                    println!("Packet with unknown Id: {}", packet_id);
                    return 0;
                }

                if length_type == 0 {
                    length = 0;
                } else if length_type == 1 {
                    length = ba.read_u8() as usize;
                } else if length_type == 2 {
                    length = ba.read_u16() as usize;
                } else if length_type == 3 {
                    length = (((ba.read_i8() as i32 & 255) << 16)
                        + ((ba.read_i8() as i32 & 255) << 8)
                        + (ba.read_i8() as i32 & 255))
                        .try_into()
                        .expect("Error at length type 3");
                }

                println!(
                    "length {} | available {} | packetId {}",
                    length,
                    ba.bytes_available(),
                    packet_id,
                );

                if length > ba.bytes_available() {
                    self.split_packet = true;
                    self.split_packet_port = port;
                    self.split_packet_length = length;
                    self.split_packet_id = packet_id;

                    ba.swap_bytes(&mut self.sba, ba.bytes_available());
                } else {
                    let initial_pos = ba.get_rpos();

                    // Parse the message
                    if packet_id == 0 {
                        ba.clear();
                    } else {
                        let message_object = PacketDecoder::parse_ba_to_object(
                            &mut ba,
                            packet_id,
                            &self.msg_from_types,
                            &self.types_from_id,
                            &self.types,
                        );

                        match message_object {
                            Ok(obj) => self.queue.push(obj),
                            Err(err) => println!("{}", err),
                        }

                        let consumed = ba.get_rpos() - initial_pos;

                        if length as isize - consumed as isize != 0 {
                            println!("warning: forced to trim a packet !");
                            ba.set_rpos(min(initial_pos + length, ba.len()));
                        }
                    }
                }
            }
        }
        1
    }

    fn parse_ba_to_object(
        packet_content: &mut ByteBuffer,
        packet_id: u16,
        msg_from_types: &Map<String, Value>,
        types_from_id: &Map<String, Value>,
        types: &Map<String, Value>,
    ) -> Result<DofusPacket, &'static str> {
        let mut dofus_packet = DofusPacket::new(
            "Server".to_owned(),
            Local::now().timestamp(),
            packet_id,
            packet_content.to_string(),
        );

        let msg = msg_from_types.get(&packet_id.to_string());
        if let Some(message_type) = msg {
            let message_type = message_type.as_object().unwrap();
            let name = message_type
                .get("name")
                .expect("Message has no name")
                .as_str()
                .unwrap();

            dofus_packet.name = name.to_string();

            let decoded = PacketDecoder::deserialize(
                packet_content,
                name,
                message_type,
                types,
                types_from_id,
            );

            dofus_packet.body = decoded;
        } else {
            return Err("Error when parsing the ba to object");
        }

        Ok(dofus_packet)
    }

    fn deserialize(
        ba: &mut ByteBuffer,
        type_name: &str,
        message_type: &Map<String, Value>,
        types_from_name: &Map<String, Value>,
        types_from_id: &Map<String, Value>,
    ) -> Map<String, Value> {
        let mut result: Map<String, Value> = Map::new();
        let msgSpec = types_from_name
            .get(type_name)
            .expect(format!("msgSpec missing ! typeName: {}", type_name).as_str())
            .as_object()
            .unwrap();

        if let Some(parent) = msgSpec.get("parent") {
            if let Some(parent_name) = parent.as_str() {
                let mut res = PacketDecoder::deserialize(
                    ba,
                    parent_name,
                    message_type,
                    types_from_name,
                    types_from_id,
                );
                result.append(&mut res);
            } // else means it's Null
        }

        if let Some(bool_vars) = msgSpec.get("boolVars") {
            if let Some(bool_vars_arr) = bool_vars.as_array() {
                if bool_vars_arr.len() > 0 {
                    let mut j = 0;

                    loop {
                        let box0: i16 = ba.read_i8().try_into().expect("box0 to i16 failed");

                        let mut i = 0;
                        while i < 8 && i < bool_vars_arr.len() / (j + 1) {
                            let bool_obj = bool_vars_arr
                                .get(i)
                                .expect("bool obj")
                                .as_object()
                                .expect("to be an object");

                            let bool_name = bool_obj
                                .get("name")
                                .expect("bool obj has name prop")
                                .as_str()
                                .unwrap();

                            let res = getFlagBooleanByte(&box0, i);

                            result.insert((&bool_name).to_string(), Value::Bool(res));

                            i = i + 1;
                        }

                        j = j + 8;
                        if j >= bool_vars_arr.len() {
                            break;
                        }
                    }
                }
            }
        }

        if let Some(vars) = msgSpec.get("vars") {
            if let Some(vars_arr) = vars.as_array() {
                for item in vars_arr.iter() {
                    let var = item.as_object().unwrap();
                    let name = item.get("name").unwrap().as_str().unwrap();
                    let length = item.get("length").unwrap();
                    let var_type = item.get("type").unwrap().as_str().unwrap();
                    let optional = item.get("optional").unwrap().as_bool().unwrap();

                    if PRIMITIVES.contains(&var_type) {
                        // println!("value: {}", name);
                        let res = PacketDecoder::read_atomic_types(ba, length, var_type);
                        let mut map_res = Map::new();
                        map_res.insert((&name).to_string(), res);
                        result.append(&mut map_res);
                    } else {
                        match length {
                            Value::Null => {
                                if var_type == "ID" {
                                    let id_num = ba.read_u16();
                                    let id_type = types_from_id.get(&id_num.to_string());

                                    if let Some(id) = id_type {
                                        let name = id
                                            .get("name")
                                            .expect("Types from id has no name")
                                            .as_str()
                                            .unwrap();
                                        let mut res = PacketDecoder::deserialize(
                                            ba,
                                            name,
                                            message_type,
                                            types_from_name,
                                            types_from_id,
                                        );
                                        result.append(&mut res);
                                    }
                                } else {
                                    let mut res = PacketDecoder::deserialize(
                                        ba,
                                        var_type,
                                        message_type,
                                        types_from_name,
                                        types_from_id,
                                    );
                                    result.append(&mut res);
                                }
                            }
                            Value::String(len_type) => {
                                let data_len = ba.read(len_type);
                                let atomic_length = match data_len {
                                    AtomicType::Boolean(v) => v.to_string(),
                                    AtomicType::UnsignedByte(v) => v.to_string(),
                                    AtomicType::Byte(v) => v.to_string(),
                                    AtomicType::UnsignedShort(v) => v.to_string(),
                                    AtomicType::Short(v) => v.to_string(),
                                    AtomicType::Int(v) => v.to_string(),
                                    AtomicType::UTF(v) => v.to_string(),
                                    AtomicType::Double(v) => v.to_string(),
                                    AtomicType::VarUhLong(v) => v.to_string(),
                                    AtomicType::VarLong(v) => v.to_string(),
                                    AtomicType::VarUhInt(v) => v.to_string(),
                                    AtomicType::VarInt(v) => v.to_string(),
                                    AtomicType::VarShort(v) => v.to_string(),
                                    AtomicType::VarUhShort(v) => v.to_string(),
                                };

                                let atomic_res = match atomic_length.parse::<u64>() {
                                    Ok(size) => {
                                        let mut arr_temp = Vec::<Map<String, Value>>::new();
                                        for i in 0..size {
                                            if var_type == "ID" {
                                                //
                                                let id_num = ba.read_u16();
                                                let id_type =
                                                    types_from_id.get(&id_num.to_string());
                                                if let Some(id) = id_type {
                                                    let name = id
                                                        .get("name")
                                                        .expect("Types from id has no name")
                                                        .as_str()
                                                        .unwrap();
                                                    let res = PacketDecoder::deserialize(
                                                        ba,
                                                        name,
                                                        message_type,
                                                        types_from_name,
                                                        types_from_id,
                                                    );
                                                    arr_temp.push(res);
                                                    // result.append(&mut res);
                                                }
                                            } else {
                                                let res = PacketDecoder::deserialize(
                                                    ba,
                                                    var_type,
                                                    message_type,
                                                    types_from_name,
                                                    types_from_id,
                                                );
                                                arr_temp.push(res);
                                            }
                                        }
                                        let mut res_map = Map::new();
                                        res_map.insert((&name).to_string(), json!(arr_temp));
                                        result.append(&mut res_map);
                                    }
                                    Err(_) => (),
                                };
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        result
    }

    pub fn get_messages(&mut self) -> Vec<DofusPacket> {
        let queue = self.queue.clone();
        self.queue.clear();
        queue
    }
    fn read_atomic_types(ba: &mut ByteBuffer, var_length: &Value, var_type: &str) -> Value {
        // if (desc.optional) {
        // }
        match var_length {
            Value::String(length) => {
                let ba_len = ba.read(&length);
                let atomic_length = match ba_len {
                    AtomicType::Boolean(v) => v.to_string(),
                    AtomicType::UnsignedByte(v) => v.to_string(),
                    AtomicType::Byte(v) => v.to_string(),
                    AtomicType::UnsignedShort(v) => v.to_string(),
                    AtomicType::Short(v) => v.to_string(),
                    AtomicType::Int(v) => v.to_string(),
                    AtomicType::UTF(v) => v.to_string(),
                    AtomicType::Double(v) => v.to_string(),
                    AtomicType::VarUhLong(v) => v.to_string(),
                    AtomicType::VarLong(v) => v.to_string(),
                    AtomicType::VarUhInt(v) => v.to_string(),
                    AtomicType::VarInt(v) => v.to_string(),
                    AtomicType::VarShort(v) => v.to_string(),
                    AtomicType::VarUhShort(v) => v.to_string(),
                };

                let atomic_res = match atomic_length.parse::<u64>() {
                    Ok(size) => {
                        let mut arr_temp = Vec::<Value>::new();
                        for i in 0..size {
                            let atomic = ba.read(var_type);
                            let json_value = atomic_to_serde_value(&atomic);
                            arr_temp.push(json_value);
                        }
                        Value::Array(arr_temp)
                    }
                    Err(_) => Value::Null,
                };

                atomic_res
            }
            _ => {
                let atomic = ba.read(var_type);
                let json_value = atomic_to_serde_value(&atomic);
                json_value
            }
        }
    }
}

fn atomic_to_serde_value(atomic: &AtomicType) -> Value {
    let res = match atomic {
        AtomicType::Boolean(v) => json!(v),
        AtomicType::UnsignedByte(v) => json!(v),
        AtomicType::Byte(v) => json!(v),
        AtomicType::UnsignedShort(v) => json!(v),
        AtomicType::Short(v) => json!(v),
        AtomicType::Int(v) => json!(v),
        AtomicType::UTF(v) => json!(v),
        AtomicType::Double(v) => json!(v),
        AtomicType::VarUhLong(v) => json!(v),
        AtomicType::VarLong(v) => json!(v),
        AtomicType::VarUhInt(v) => json!(v),
        AtomicType::VarInt(v) => json!(v),
        AtomicType::VarShort(v) => json!(v),
        AtomicType::VarUhShort(v) => json!(v),
    };
    // println!("atomic to serde res: {}, type {:?}", res, atomic);
    res
}

fn getFlagBooleanByte(a: &i16, pos: usize) -> bool {
    let b = false;
    match pos {
        0 => (a & 1) != 0,
        1 => (a & 2) != 0,
        2 => (a & 4) != 0,
        3 => (a & 8) != 0,
        4 => (a & 16) != 0,
        5 => (a & 32) != 0,
        6 => (a & 64) != 0,
        7 => (a & 128) != 0,
        _ => false,
    };
    b
}
