use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Signal {
    #[serde(rename = "webrtc_offer")]
    WebRtcOffer { data: WebRtcOfferData },
    #[serde(rename = "webrtc_answer")]
    WebRtcAnswer { data: WebRtcAnswerData },
    #[serde(rename = "ice_candidate")]
    IceCandidate { data: IceCandidateData },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WebRtcOfferData {
    pub session_id: u64,
    pub webrtc_data: WebRtcData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WebRtcAnswerData {
    pub session_id: u64,
    pub webrtc_data: WebRtcData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IceCandidateData {
    pub session_id: u64,
    pub webrtc_data: IceCandidateWebRtcData,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WebRtcData {
    #[serde(rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_type: Option<String>,
    pub sdp: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IceCandidateWebRtcData {
    pub candidate: String,
    #[serde(rename = "sdpMLineIndex")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sdp_m_line_index: Option<u16>,
}
