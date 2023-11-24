use std::{cell::RefCell, net::SocketAddr, rc::Rc, str::FromStr};

use js_sys::Reflect;
use signals::{IceCandidateData, IceCandidateWebRtcData, Signal, WebRtcData, WebRtcOfferData};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    RtcPeerConnection, RtcPeerConnectionIceEvent, RtcRtpTransceiverInit, RtcSessionDescriptionInit,
};

mod signals;
// mod web_socket;
#[macro_use]
mod common;

#[wasm_bindgen]
pub async fn start(addr: &str) -> Result<(), JsValue> {
    console_log!("initializing WebRTC...");

    let session_id = chrono::Local::now().timestamp_nanos_opt().unwrap() as u64;
    console_log!("{}", addr);
    let address = SocketAddr::from_str(addr).unwrap();
    let ws = match web_sys::WebSocket::new(&format!("ws://{}", address)) {
        Ok(ws) => ws,
        Err(err) => {
            console_warn!("Failed to connection. {:?}", err);
            panic!("Failed to connection.");
        }
    };
    let ws_clone = ws.clone();

    let (tx, rx) = futures::channel::oneshot::channel::<()>();

    let peer = RtcPeerConnection::new()?;
    peer.add_transceiver_with_str_and_init(
        "video",
        RtcRtpTransceiverInit::new().direction(web_sys::RtcRtpTransceiverDirection::Recvonly),
    );

    let on_track_cb = Closure::<dyn FnMut(_)>::new(move |event: web_sys::Event| {
        if let Ok(video) = web_sys::Document::new() {
            if let Some(element) = video.query_selector("video").unwrap() {}
        }
    });

    peer.set_ontrack(Some(on_track_cb.as_ref().unchecked_ref()));
    let peer_clone = peer.clone();
    let tx = Rc::new(RefCell::new(Some(tx)));
    let tx_clone = tx.clone();

    let on_open_cb = Closure::<dyn FnMut()>::new(move || {
        console_log!("[Signals] OPEN");
        let tx = tx_clone.borrow_mut().take().unwrap();
        tx.send(()).unwrap();
    });
    ws.set_onopen(Some(on_open_cb.as_ref().unchecked_ref()));
    on_open_cb.forget();

    let on_msg_cb = Closure::<dyn FnMut(_)>::new(move |event: web_sys::MessageEvent| {
        if let Some(msg) = event.data().as_string() {
            if let Ok(signal) = serde_json::from_str::<Signal>(&msg) {
                // responce
                console_log!("[Signals] Receive message: {:?}", signal);
                on_signal(&peer, signal);
            }
        }
    });
    ws.set_onmessage(Some(on_msg_cb.as_ref().unchecked_ref()));
    on_msg_cb.forget();

    let on_icecandidate_cb =
        Closure::<dyn FnMut(_)>::new(move |event: RtcPeerConnectionIceEvent| {
            if let Some(candidate) = event.candidate() {
                console_log!("[{:?}][local ICE candidate] {:?}", session_id, candidate);
                let signal = Signal::IceCandidate {
                    data: IceCandidateData {
                        session_id,
                        webrtc_data: IceCandidateWebRtcData {
                            candidate: candidate.candidate(),
                            sdp_m_line_index: candidate.sdp_m_line_index(),
                        },
                    },
                };
                let _ = send_signal(&ws, signal);
            }
        });
    peer_clone.set_onicecandidate(Some(on_icecandidate_cb.as_ref().unchecked_ref()));
    on_icecandidate_cb.forget();

    let sdp = make_offer(&peer_clone).await?;
    let offer_signal = Signal::WebRtcOffer {
        data: WebRtcOfferData {
            session_id,
            webrtc_data: WebRtcData {
                data_type: Some("offer".to_string()),
                sdp,
            },
        },
    };

    if let Ok(_) = rx.await {
        send_signal(&ws_clone, offer_signal)?;
    }

    Ok(())
}

// fn send_signal(signal: Signal, data) {}
fn on_signal(peer: &RtcPeerConnection, signal: Signal) {
    match signal {
        Signal::WebRtcOffer { data: _ } => {}
        Signal::WebRtcAnswer { data } => {
            console_log!("[Signals] Receive answer {:?}", data);
            let _ = peer.set_remote_description(
                &RtcSessionDescriptionInit::new(web_sys::RtcSdpType::Answer)
                    .sdp(&data.webrtc_data.sdp),
            );
        }
        Signal::IceCandidate { data } => {
            console_log!("[Signals] Receive ICE candidate {:?}", data);
            let _ = peer.add_ice_candidate_with_opt_rtc_ice_candidate_init(Some(
                web_sys::RtcIceCandidateInit::new(&data.webrtc_data.candidate)
                    .sdp_m_line_index(data.webrtc_data.sdp_m_line_index),
            ));
        }
    }
}

fn send_signal(ws: &web_sys::WebSocket, signal: Signal) -> Result<(), JsValue> {
    if let Ok(msg) = serde_json::to_string(&signal) {
        console_log!("[Signals] SEND {}", msg);
        ws.send_with_str(&msg)?;
    }
    Ok(())
}

async fn make_offer(peer: &RtcPeerConnection) -> Result<String, JsValue> {
    if let Ok(offer) = JsFuture::from(peer.create_offer()).await {
        let sdp = Reflect::get(&offer, &JsValue::from_str("sdp"))?
            .as_string()
            .unwrap();
        let _ = peer.set_local_description(
            web_sys::RtcSessionDescriptionInit::new(web_sys::RtcSdpType::Offer).sdp(&sdp),
        );
        return Ok(sdp);
    } else {
        console_warn!("Failed to create sdp offer.");
        panic!("Failed to create sdp offer.");
    }
}
