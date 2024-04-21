use base64::prelude::*;
use prost_reflect::{DescriptorPool, DynamicMessage, MessageDescriptor};
use serde::Serialize;
use serde_json::Value as JsonValue;
use std::{collections::HashMap, error::Error};

use crate::SERIALIZE_OPTIONS;

#[derive(Debug)]
pub enum MessageType {
    Notify = 1,
    Request = 2,
    Response = 3,
}

#[derive(Debug)]
pub struct LiqiMessage {
    pub id: usize,
    pub msg_type: MessageType,
    pub method_name: String,
    pub data: JsonValue,
}

#[derive(Serialize, Debug)]
pub struct Action {
    pub name: String,
    pub data: JsonValue,
}

pub struct Parser {
    total: usize,
    respond_type: HashMap<usize, (String, MessageDescriptor)>,
    proto_json: JsonValue,
    pub pool: DescriptorPool,
}

pub fn my_serialize(msg: DynamicMessage) -> Result<JsonValue, Box<dyn Error>> {
    let mut serializer = serde_json::Serializer::new(vec![]);
    msg.serialize_with_options(&mut serializer, &SERIALIZE_OPTIONS)?;
    let json_str = String::from_utf8(serializer.into_inner())?;
    Ok(serde_json::from_str(&json_str)?)
}

impl Parser {
    pub fn new() -> Self {
        let json_str = include_str!("liqi.json");
        let proto_json = serde_json::from_str(json_str).expect("Failed to parse liqi.json");
        let pool = DescriptorPool::decode(include_bytes!("liqi.desc").as_ref())
            .expect("Failed to decode liqi.desc");
        Self {
            total: 0,
            respond_type: HashMap::new(),
            proto_json,
            pool,
        }
    }

    pub fn parse(&mut self, buf: &[u8]) -> Result<LiqiMessage, Box<dyn Error>> {
        let msg_type_byte = buf[0];
        if !(1..=3).contains(&msg_type_byte) {
            return Err(format!("Invalid message type: {}", msg_type_byte).into());
        }
        let msg_type = match msg_type_byte {
            1 => MessageType::Notify,
            2 => MessageType::Request,
            3 => MessageType::Response,
            _ => unreachable!(),
        };
        let method_name;
        let mut data_obj: JsonValue;
        let msg_id: usize;
        match msg_type {
            MessageType::Notify => {
                let blocks = buf_to_blocks(&buf[1..])?;
                method_name = String::from_utf8(blocks[0].data.clone())?;
                let method_name_list: Vec<&str> = method_name.split('.').collect();
                let message_name = method_name_list[2];
                let message_type = self
                    .pool
                    .get_message_by_name(&to_fqn(message_name))
                    .ok_or(format!("Invalid message type: {}", message_name))?;
                let dyn_msg = DynamicMessage::decode(message_type, blocks[1].data.as_ref())?;
                data_obj = my_serialize(dyn_msg)?;
                if let Some(b64) = data_obj.get("data") {
                    let action_name = data_obj
                        .get("name")
                        .ok_or("No name field")?
                        .as_str()
                        .ok_or("Name is not a string")?;
                    let b64 = b64.as_str().unwrap_or_default();

                    let decoded = BASE64_STANDARD.decode(b64)?;
                    let my_decoded = decode(&decoded);
                    let action_type = self
                        .pool
                        .get_message_by_name(&to_fqn(action_name))
                        .ok_or(format!("Invalid action type: {}", action_name))?;
                    let action_msg = DynamicMessage::decode(action_type, my_decoded.as_ref())?;
                    let action_obj = my_serialize(action_msg)?;
                    data_obj
                        .as_object_mut()
                        .ok_or("Data is not an object")?
                        .insert("data".to_string(), action_obj);
                }
                msg_id = self.total;
            }
            MessageType::Request => {
                // little endian, msg_id = unpack("<H", buf[1:3])[0]
                msg_id = u16::from_le_bytes([buf[1], buf[2]]) as usize;
                let blocks = buf_to_blocks(&buf[3..])?;
                assert!(msg_id < 1 << 16);
                assert!(blocks.len() == 2);
                // ascii decode into method name, method_name = msg_block[0]["data"].decode()
                let method_block = &blocks[0];
                method_name = String::from_utf8(method_block.data.to_owned())?;
                let method_name_list: Vec<&str> = method_name.split('.').collect();
                let lq = method_name_list[1];
                let service = method_name_list[2];
                let rpc = method_name_list[3];
                let proto_domain =
                    &self.proto_json["nested"][lq]["nested"][service]["methods"][rpc];
                let req_value = &proto_domain["requestType"];
                if let JsonValue::String(name) = req_value {
                    let req_type = self
                        .pool
                        .get_message_by_name(&to_fqn(name))
                        .ok_or(format!("Invalid request type: {}", name))?;
                    let dyn_msg = DynamicMessage::decode(req_type, blocks[1].data.as_ref())?;
                    data_obj = my_serialize(dyn_msg)?;
                    let res_type_name = proto_domain["responseType"]
                        .as_str()
                        .ok_or("Invalid response type")?;
                    let resp_type = self
                        .pool
                        .get_message_by_name(&to_fqn(res_type_name))
                        .ok_or(format!("Invalid response type: {}", res_type_name))?;
                    self.respond_type
                        .insert(msg_id, (method_name.to_owned(), resp_type));
                } else {
                    return Err(format!("Invalid request type in json: {:?}", req_value).into());
                }
            }
            MessageType::Response => {
                msg_id = u16::from_le_bytes([buf[1], buf[2]]) as usize;
                let blocks = buf_to_blocks(&buf[3..])?;
                assert!(blocks[0].data.is_empty());
                let resp_type: MessageDescriptor;
                (method_name, resp_type) = self
                    .respond_type
                    .remove(&msg_id)
                    .ok_or("No response type found")?;
                let dyn_msg =
                    DynamicMessage::decode(resp_type.to_owned(), blocks[1].data.as_ref())?;
                data_obj = my_serialize(dyn_msg)?;
            }
        }
        self.total += 1;
        Ok(LiqiMessage {
            id: msg_id,
            msg_type,
            method_name,
            data: data_obj,
        })
    }
}

pub fn to_fqn(method_name: &str) -> String {
    format!("lq.{}", method_name)
}

struct Block {
    _id: usize,
    _blk_type: usize,
    data: Vec<u8>,
    _begin: usize,
}

fn buf_to_blocks(buf: &[u8]) -> Result<Vec<Block>, String> {
    let mut blocks = Vec::new();
    let mut i = 0;
    let l = buf.len();
    while i < l {
        let begin = i;
        let blk_type = (buf[i] & 0x07) as usize;
        let id = (buf[i] >> 3) as usize;
        i += 1;
        let data: Vec<u8>;
        match blk_type {
            0 => {
                let (int, p) = parse_var_int(buf, i);
                // convert int to bytes
                data = int.to_be_bytes().to_vec();
                i = p;
            }
            2 => {
                let (len, p) = parse_var_int(buf, i);
                data = buf[p..p + len].to_vec();
                i = p + len;
            }
            _ => return Err(format!("Unknown block type: {}", blk_type)),
        }
        blocks.push(Block {
            _id: id,
            _blk_type: blk_type,
            data,
            _begin: begin,
        });
    }
    Ok(blocks)
}

fn parse_var_int(buf: &[u8], p: usize) -> (usize, usize) {
    let mut data = 0;
    let mut shift = 0;
    let l = buf.len();
    let mut i = p;
    while i < l {
        data += ((buf[i] & 0x7f) as usize) << shift;
        shift += 7;
        i += 1;
        if buf[i - 1] >> 7 == 0 {
            break;
        }
    }
    (data, i)
}

fn decode(data: &[u8]) -> Vec<u8> {
    let keys = [0x84, 0x5E, 0x4E, 0x42, 0x39, 0xA2, 0x1F, 0x60, 0x1C];
    let mut data = data.to_vec();
    let k = keys.len();
    let d = data.len();
    for i in 0..d {
        let u = ((23 ^ d) + 5 * i + keys[i % k]) & 255;
        data[i] ^= u as u8;
    }
    data
}
