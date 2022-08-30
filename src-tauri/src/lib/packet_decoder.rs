#![allow(dead_code, unused_variables, non_snake_case)]
// use std::any::Any;

use anyhow::Result;
use bytebuffer::ByteBuffer;
use serde_json::{Map, Value};

use std::{cmp::min, collections::HashMap, fs};

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

trait Buffer {
    fn bytes_available(&self) -> usize {
        0 as usize
    }
    fn swap_bytes(&mut self, bytes: &mut ByteBuffer, offset: usize, len: usize) {}
}

impl Buffer for ByteBuffer {
    fn bytes_available(&self) -> usize {
        return self.len() - self.get_rpos();
    }

    fn swap_bytes(&mut self, bytes: &mut ByteBuffer, offset: usize, len: usize) {
        let mut length = len.clone();
        if length == 0 {
            length = self.bytes_available();
        }

        if length > self.bytes_available() {
            println!("End of buffer was encountered.");
            return;
        }

        bytes.write_bytes(&self.read_bytes(self.bytes_available()));

        self.set_rpos(min(self.get_rpos() + length, self.len()));
    }
}
struct DofusPacket {
    source: String,
    time: String,
    id: u32,
    name: String,
    raw: String,
    body: String,
}

impl DofusPacket {
    fn new() -> DofusPacket {
        DofusPacket {
            source: "".to_string(),
            time: "".to_string(),
            id: 0,
            name: "".to_string(),
            raw: "".to_string(),
            body: "".to_string(),
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
    // messages:
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
                    let offset = self.sba.len().clone();
                    ba.swap_bytes(&mut self.sba, offset, ba.bytes_available());
                } else {
                    let offset = self.sba.len().clone();
                    ba.swap_bytes(&mut self.sba, offset, self.split_packet_length - offset);

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
                    ba.clear();

                    if self.split_packet_length - consumed != 0 {
                        // println!("warning: forced to trim a packet !");
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
                    println!("- {}", name);
                } else {
                    println!("Packet with unknown Id: {}", packet_id);
                    return 0;
                }

                // println!("Before, bytes: {:?}", ba.to_bytes());
                if length_type == 0 {
                    length = 0;
                } else if length_type == 1 {
                    length = ba.read_u8() as usize;
                } else if length_type == 2 {
                    length = ba.read_u16() as usize;
                } else if length_type == 3 {
                    length = (((ba.read_u8() as u32 & 255) << 16)
                        + ((ba.read_u8() as u32 & 255) << 8)
                        + (ba.read_u8() as u32 & 255))
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
                    println!("Set split packet");
                    self.split_packet = true;
                    self.split_packet_port = port;
                    self.split_packet_length = length;
                    self.split_packet_id = packet_id;

                    let offset = self.sba.len().clone();
                    ba.swap_bytes(&mut self.sba, offset, ba.bytes_available());
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
                        ba.clear();

                        if length - consumed != 0 {
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
        let dofus_packet = DofusPacket::new();

        let msg = msg_from_types.get(&packet_id.to_string());

        if let Some(message_type) = msg {
            let message_type = message_type.as_object().unwrap();
            let name = message_type
                .get("name")
                .expect("Message has no name")
                .as_str()
                .unwrap();

            PacketDecoder::deserialize(packet_content, name, message_type, types);
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
    ) -> Map<String, Value> {
        let mut result: Map<String, Value> = Map::new();
        let msgSpec = types_from_name
            .get(type_name)
            .expect(format!("msgSpec missing ! typeName: {}", type_name).as_str())
            .as_object()
            .unwrap();

        if let Some(parent) = msgSpec.get("parent") {
            if let Some(parent_name) = parent.as_str() {
                let mut res =
                    PacketDecoder::deserialize(ba, type_name, message_type, types_from_name);
                result.append(&mut res);
            } // else means it's Null
        }

        if let Some(bool_vars) = msgSpec.get("boolVars") {
            if let Some(bool_vars_arr) = bool_vars.as_array() {
                for item in bool_vars_arr.iter() {
                    // let var = item.as_object().unwrap();
                    // let name = vars.get("name").unwrap().as_str().unwrap();
                    // let length = vars.get("length").unwrap().as_str().unwrap();
                    // let var_type = vars.get("type").unwrap().as_str().unwrap();
                    // let optional = vars.get("optional").unwrap().as_str().unwrap();

                    let box0 = ba.read_u8();
                    // for (let i = 0; i < 8 && i < msgSpec.boolVars.length / (j + 1); i++) {
                    //     let bool1 = msgSpec?.boolVars[i];
                    //     result = {
                    //       ...result,
                    //       [bool1.name]: BooleanByteWrapper.getFlag(_box0, i),
                    //     };
                    //   }
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
                        PacketDecoder::read_atomic_types(ba, length, var_type);
                    } else {
                        match length {
                            Value::Null =>
                            // let type = v.type;

                            // if (type == "ID") {
                            //   let id = data.readUnsignedShort();
                            //   type = getTypeFromId[id]?.name;
                            // }
                            // if (type) {
                            //   const r = deserialize(data, type);
                            //   console.log(r);
                            //   result = { ...result, ...r };
                            // }
                            {
                                todo!()
                            }

                            Value::String(_) =>
                            // const length = data.read(v.length);
                            // const res = [];
                            // for (let i = 0; i < length; i++) {
                            //   let type = v.type;

                            //   if (type === "ID") {
                            //     let id = data.readUnsignedShort();

                            //     type = types_from_id[id]?.name;
                            //     // if (!type) console.log(id);
                            //   }
                            //   if (type) {
                            //     const r = deserialize(data, type);
                            //     console.log(r);
                            //     res.push(r);
                            //   }
                            // }
                            // result = { ...result, [v.name]: res };
                            {
                                todo!()
                            }
                            _ => todo!(),
                        }
                    }
                }
            }
        }

        result
    }

    fn read_atomic_types(
        ba: &mut ByteBuffer,
        var_length: &Value,
        var_type: &str,
    ) -> Map<String, Value> {
        // if (desc.optional) {
        // }
        // let result = {};
        // try {
        //   if (desc.length) {
        //     let length = data.read(desc.length);
        //     if (typeof length != "number") {
        //       throw new Error("length not number : " + length);
        //     }
        //     let res = [];
        //     for (let i = 0; i < length; i++) {
        //       res.push(data.read(desc.type));
        //     }
        //     result = res;
        //   } else {
        //     result = data.read(desc.type);
        //   }
        // } catch (ex) {
        //   console.log("error!", ex);
        //   return "ERROR";
        // }

        // return result;
        Map::new()
    }
}
