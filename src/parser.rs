use anyhow::{Context, Result, ensure};
use base64::prelude::*;
use bytes::Bytes;
use prost::Message;
use prost_reflect::{DescriptorPool, DynamicMessage, MessageDescriptor, SerializeOptions};
use serde_json::{Value as JsonValue, value::Serializer};
use std::{collections::HashMap, sync::Arc};

use crate::proto::base::BaseMessage;

const SERIALIZE_OPTIONS: SerializeOptions = SerializeOptions::new()
    .skip_default_fields(false)
    .use_proto_field_name(true);

#[derive(Debug, Clone)]
pub enum MessageType {
    Notify = 1,
    Request = 2,
    Response = 3,
}

#[derive(Debug, Clone)]
pub struct LiqiMessage {
    pub id: usize,
    pub msg_type: MessageType,
    pub method_name: Arc<str>,
    pub data: JsonValue,
}

impl LiqiMessage {
    pub fn new(id: usize, msg_type: MessageType, method_name: Arc<str>, data: JsonValue) -> Self {
        Self {
            id,
            msg_type,
            method_name,
            data,
        }
    }
}

#[derive(Debug)]
pub struct Parser {
    total: usize,
    pub respond_type: HashMap<usize, (Arc<str>, MessageDescriptor)>,
    proto_json: &'static JsonValue,
    pub pool: &'static DescriptorPool,
}

fn dyn_to_json(msg: &DynamicMessage) -> Result<JsonValue> {
    Ok(msg.serialize_with_options(Serializer, &SERIALIZE_OPTIONS)?)
}

impl Parser {
    pub fn new(proto_json: &'static JsonValue, pool: &'static DescriptorPool) -> Self {
        Self {
            total: 0,
            respond_type: HashMap::new(),
            proto_json,
            pool,
        }
    }

    /// Parses a raw message buffer into a structured LiqiMessage
    ///
    /// # Arguments
    /// * `buf` - The raw message bytes to parse
    ///
    /// # Returns
    /// A Result containing the parsed LiqiMessage or an error
    ///
    /// # Errors
    /// Returns an error if:
    /// - Invalid message type
    /// - Failed to decode protobuf message
    /// - Invalid message structure
    pub fn parse(&mut self, buf: Bytes) -> Result<LiqiMessage> {
        // Validate message type
        let msg_type = match buf[0] {
            1 => MessageType::Notify,
            2 => MessageType::Request,
            3 => MessageType::Response,
            t => anyhow::bail!("Invalid message type: {}", t),
        };

        // Parse based on message type
        let (msg_id, method_name, data_obj) = match msg_type {
            MessageType::Notify => self.parse_notify(&buf)?,
            MessageType::Request => self.parse_request(&buf)?,
            MessageType::Response => self.parse_response(&buf)?,
        };

        self.total += 1;
        Ok(LiqiMessage::new(msg_id, msg_type, method_name, data_obj))
    }

    fn parse_notify(&self, buf: &[u8]) -> Result<(usize, Arc<str>, JsonValue)> {
        let msg_block = BaseMessage::decode(&buf[1..])?;
        let method_name: Arc<str> = Arc::from(msg_block.method_name);

        // Extract message name from method (e.g. "lq.Notify.RoomMessage" -> "RoomMessage")
        let message_name = method_name
            .split('.')
            .nth(2)
            .context("Invalid method name format")?;

        // Decode and convert to JSON
        let message_type = self
            .pool
            .get_message_by_name(&to_fqn(message_name))
            .context(format!("Invalid message type: {}", message_name))?;
        let dyn_msg = DynamicMessage::decode(message_type, msg_block.data.as_ref())?;
        let mut data_obj = dyn_to_json(&dyn_msg)?;

        // Handle nested action data if present
        if let Some(b64) = data_obj.get("data") {
            let action_name = data_obj["name"].as_str().context("name field invalid")?;
            let b64 = b64.as_str().unwrap_or_default();
            let action_obj = decode_action(action_name, b64, self.pool)?;
            data_obj
                .as_object_mut()
                .context("data is not an object")?
                .insert("data".to_string(), action_obj);
        }

        Ok((self.total, method_name, data_obj))
    }

    fn parse_request(&mut self, buf: &[u8]) -> Result<(usize, Arc<str>, JsonValue)> {
        // Extract message ID (little-endian u16)
        let msg_id = u16::from_le_bytes([buf[1], buf[2]]) as usize;

        let msg_block = BaseMessage::decode(&buf[3..])?;
        let method_name: Arc<str> = Arc::from(msg_block.method_name);

        // Split method name into components (e.g. "lq.Lobby.oauth2Login")
        let parts: Vec<&str> = method_name.split('.').collect();
        ensure!(parts.len() >= 4, "Invalid method name format");

        // Lookup method details in proto JSON
        let proto_domain =
            &self.proto_json["nested"][parts[1]]["nested"][parts[2]]["methods"][parts[3]];

        // Decode request
        let req_type_name = proto_domain["requestType"]
            .as_str()
            .context("Invalid request type")?;
        let req_type = self
            .pool
            .get_message_by_name(&to_fqn(req_type_name))
            .context(format!("Invalid request type: {}", req_type_name))?;
        let dyn_msg = DynamicMessage::decode(req_type, msg_block.data.as_ref())?;
        let data_obj = dyn_to_json(&dyn_msg)?;

        // Store response type for later
        let res_type_name = proto_domain["responseType"]
            .as_str()
            .context("Invalid response type")?;
        let resp_type = self
            .pool
            .get_message_by_name(&to_fqn(res_type_name))
            .context(format!("Invalid response type: {}", res_type_name))?;
        self.respond_type
            .insert(msg_id, (method_name.clone(), resp_type));

        Ok((msg_id, method_name, data_obj))
    }

    fn parse_response(&mut self, buf: &[u8]) -> Result<(usize, Arc<str>, JsonValue)> {
        let msg_id = u16::from_le_bytes([buf[1], buf[2]]) as usize;

        let msg_block = BaseMessage::decode(&buf[3..])?;
        ensure!(
            msg_block.method_name.is_empty(),
            "Response should have empty method name"
        );

        // Retrieve stored method info
        let (method_name, resp_type) = self
            .respond_type
            .remove(&msg_id)
            .context("No corresponding request")?;

        // Decode response
        let dyn_msg = DynamicMessage::decode(resp_type, msg_block.data.as_ref())?;
        let data_obj = dyn_to_json(&dyn_msg)?;

        Ok((msg_id, method_name, data_obj))
    }
}

/// Converts a method name to its fully qualified name (FQN) by prefixing with "lq."
///
/// # Arguments
/// * `method_name` - The method name to convert
///
/// # Returns
/// A new String with the fully qualified name
fn to_fqn(method_name: &str) -> String {
    // Use format! for better readability and performance
    format!("lq.{}", method_name)
}

pub fn decode_action(name: &str, data: &str, pool: &DescriptorPool) -> Result<JsonValue> {
    let mut decoded = BASE64_STANDARD.decode(data)?;
    wtf_decode(&mut decoded);
    let action_type = pool
        .get_message_by_name(&to_fqn(name))
        .context(format!("Invalid action type: {name}"))?;
    let action_msg = DynamicMessage::decode(action_type, decoded.as_ref())?;
    dyn_to_json(&action_msg)
}

fn wtf_decode(data: &mut [u8]) {
    const KEYS: [u8; 9] = [0x84, 0x5E, 0x4E, 0x42, 0x39, 0xA2, 0x1F, 0x60, 0x1C];
    let base = 23 ^ data.len();
    KEYS.iter()
        .cycle()
        .zip(data.iter_mut())
        .enumerate()
        .for_each(|(i, (key, b))| *b ^= (base + 5 * i + *key as usize) as u8);
}
