use anyhow::Result;
use bytes::Bytes;
use hudsucker::{
    futures::{Sink, SinkExt, Stream, StreamExt},
    tokio_tungstenite::tungstenite::{self, Message},
    *,
};
use std::sync::Arc;
use tokio::sync::{mpsc::Sender, RwLock};
use tracing::*;

use crate::{
    modder::Modder,
    parser::{LiqiMessage, Parser},
    settings::Settings,
};

#[derive(Clone)]
pub struct Handler {
    sender: Option<Sender<(LiqiMessage, char)>>,
    modder: Option<Arc<Modder>>,
    inject_msg: Option<Message>,
    parser: Arc<RwLock<Parser>>,
}

impl Handler {
    pub fn new(
        sender: Option<Sender<(LiqiMessage, char)>>,
        modder: Option<Arc<Modder>>,
        settings: &'static Settings,
    ) -> Self {
        Self {
            sender,
            modder,
            inject_msg: None,
            parser: Arc::new(RwLock::new(Parser::new(
                &settings.proto_json,
                &settings.desc,
            ))),
        }
    }
}

impl WebSocketHandler for Handler {
    async fn handle_websocket(
        mut self,
        ctx: WebSocketContext,
        mut stream: impl Stream<Item = Result<Message, tungstenite::Error>> + Unpin + Send + 'static,
        mut sink: impl Sink<Message, Error = tungstenite::Error> + Unpin + Send + 'static,
    ) {
        if let WebSocketContext::ServerToClient { .. } = ctx {
            if let Some(msg) = self.inject_msg.take() {
                if let Err(e) = sink.send(msg).await {
                    error!("Failed to send injected message: {e}");
                }
            }
        }
        while let Some(message) = stream.next().await {
            match message {
                Ok(message) => {
                    let Some(message) = self.handle_message(&ctx, message).await else {
                        continue;
                    };

                    match sink.send(message).await {
                        Err(tungstenite::Error::ConnectionClosed) => (),
                        Err(e) => error!("WebSocket send error: {e}"),
                        _ => (),
                    }
                }
                Err(e) => {
                    error!("WebSocket message error: {e}");

                    match sink.send(Message::Close(None)).await {
                        Err(tungstenite::Error::ConnectionClosed) => (),
                        Err(e) => error!("WebSocket close error: {e}"),
                        _ => (),
                    };

                    break;
                }
            }
        }
    }

    async fn handle_message(&mut self, _ctx: &WebSocketContext, msg: Message) -> Option<Message> {
        let (direction_char, uri) = match _ctx {
            WebSocketContext::ServerToClient { src, .. } => ('\u{2193}', src),
            WebSocketContext::ClientToServer { dst, .. } => ('\u{2191}', dst),
        };

        if uri.path() == "/ob" {
            // ignore ob messages
            return Some(msg);
        }

        debug!("{direction_char} {uri}");

        let Message::Binary(buf) = msg else {
            return Some(msg);
        };

        let buf: Bytes = buf.into();
        let mut parser = self.parser.write().await;
        let Ok(parsed) = parser.parse(buf.clone()) else {
            error!("Failed to parse message");
            return Some(Message::Binary(buf.into()));
        };
        drop(parser);

        let method_name = parsed.method_name.clone();
        if let Some(tx) = &self.sender {
            if let Err(e) = tx.send((parsed, direction_char)).await {
                error!("Failed to send message to channel: {e}");
            }
        }
        let Some(ref modder) = self.modder else {
            return Some(Message::Binary(buf.into()));
        };
        let parser = self.parser.read().await;
        let res = modder
            .modify(buf, direction_char == '\u{2191}', method_name)
            .await;
        drop(parser);
        if let Some(inj) = res.inject_msg {
            self.inject_msg = Some(Message::Binary(inj.into()));
        }
        res.msg.map(|msg| Message::Binary(msg.into()))
    }
}
