use serde::de::DeserializeOwned;
use std::net::SocketAddr;
use wasm_bindgen::prelude::*;
use web_sys::WebSocket;

use crate::{console_log, console_warn};

pub struct Client {
    ws: WebSocket,
}

impl Client {
    pub fn connect<T>(addr: SocketAddr) -> Result<Self, JsValue>
    where
        T: DeserializeOwned + 'static,
    {
        let ws = match WebSocket::new(&format!("ws://{}", addr)) {
            Ok(ws) => ws,
            Err(err) => {
                console_warn!("Failed to connection. {:?}", err);
                panic!("Failed to connection.");
            }
        };

        set_onopen(&ws);
        set_onmsg(&ws);

        Ok(Self { ws })
    }

    pub fn send_msg<T>(&self, msg: T) -> Result<(), JsValue>
    where
        T: serde::Serialize,
    {
        if let Ok(msg) = serde_json::to_string(&msg) {
            self.ws.send_with_str(&msg)?;
            console_log!("Success to send message: {}", msg);
        }
        Ok(())
    }
}

fn set_onopen(ws: &WebSocket) {
    let callback = Closure::<dyn FnMut()>::new(move || {
        console_log!("[Signals] OPEN");
    });
    ws.set_onopen(Some(callback.as_ref().unchecked_ref()));
    callback.forget();
}

fn set_onmsg<T>(ws: &WebSocket)
where
    T: DeserializeOwned + 'static,
{
    let callback = Closure::<dyn FnMut(_)>::new(move |event: web_sys::MessageEvent| {
        if let Some(msg) = event.data().as_string() {
            if let Ok(msg) = serde_json::from_str::<T>(&msg) {
                // if let Err(err) = signal_incoming_tx.send(msg) {
                //     console_warn!("Failed to send message: {}", err);
                // }
            }
        }
    });
    ws.set_onmessage(Some(callback.as_ref().unchecked_ref()));
    callback.forget();
}
