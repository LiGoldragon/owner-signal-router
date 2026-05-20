//! OwnerSignal contract for privileged PersonaRouter channel policy.
//!
//! Ordinary router observation traffic lives in `signal-persona-router`.
//! This crate carries owner-only channel authority orders for
//! `persona-router`.

use nota_codec::{Decoder, Encoder, NotaDecode, NotaEncode, NotaEnum, NotaRecord, NotaTransparent};
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};
use signal_frame::signal_channel;
use signal_persona_auth::{ChannelId, ComponentName, ConnectionClass};

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaTransparent, Debug, Clone, PartialEq, Eq, Hash,
)]
pub struct AdjudicationRequestIdentifier(String);

impl AdjudicationRequestIdentifier {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaTransparent, Debug, Clone, PartialEq, Eq)]
pub struct TextBody(String);

impl TextBody {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(
    Archive,
    RkyvSerialize,
    RkyvDeserialize,
    NotaTransparent,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
)]
pub struct TimestampNanoseconds(u64);

impl TimestampNanoseconds {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn value(self) -> u64 {
        self.0
    }
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq)]
pub enum ChannelEndpoint {
    Internal(ComponentName),
    External(ConnectionClass),
}

impl NotaEncode for ChannelEndpoint {
    fn encode(&self, encoder: &mut Encoder) -> nota_codec::Result<()> {
        match self {
            Self::Internal(component) => {
                encoder.start_record("Internal")?;
                component.encode(encoder)?;
                encoder.end_record()
            }
            Self::External(connection) => {
                encoder.start_record("External")?;
                connection.encode(encoder)?;
                encoder.end_record()
            }
        }
    }
}

impl NotaDecode for ChannelEndpoint {
    fn decode(decoder: &mut Decoder<'_>) -> nota_codec::Result<Self> {
        let head = decoder.peek_record_head()?;
        match head.as_str() {
            "Internal" => {
                decoder.expect_record_head("Internal")?;
                let component = ComponentName::decode(decoder)?;
                decoder.expect_record_end()?;
                Ok(Self::Internal(component))
            }
            "External" => {
                decoder.expect_record_head("External")?;
                let connection = ConnectionClass::decode(decoder)?;
                decoder.expect_record_end()?;
                Ok(Self::External(connection))
            }
            other => Err(nota_codec::Error::UnknownKindForVerb {
                verb: "ChannelEndpoint",
                got: other.to_string(),
            }),
        }
    }
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEnum, Debug, Clone, Copy, PartialEq, Eq, Hash,
)]
pub enum ChannelMessageKind {
    MessageIngressSubmission,
    MessageSubmission,
    InboxQuery,
    FocusObservation,
    PromptBufferObservation,
    MessageDelivery,
    TerminalInput,
    TerminalCapture,
    TerminalResize,
    TranscriptEvent,
    AdjudicationRequest,
    DeliveryNotification,
}

impl ChannelMessageKind {
    pub const ALL: [Self; 12] = [
        Self::MessageIngressSubmission,
        Self::MessageSubmission,
        Self::InboxQuery,
        Self::FocusObservation,
        Self::PromptBufferObservation,
        Self::MessageDelivery,
        Self::TerminalInput,
        Self::TerminalCapture,
        Self::TerminalResize,
        Self::TranscriptEvent,
        Self::AdjudicationRequest,
        Self::DeliveryNotification,
    ];
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChannelDuration {
    OneShot,
    Permanent,
    TimeBound(TimestampNanoseconds),
}

impl NotaEncode for ChannelDuration {
    fn encode(&self, encoder: &mut Encoder) -> nota_codec::Result<()> {
        match self {
            Self::OneShot => {
                encoder.start_record("OneShot")?;
                encoder.end_record()
            }
            Self::Permanent => {
                encoder.start_record("Permanent")?;
                encoder.end_record()
            }
            Self::TimeBound(timestamp) => {
                encoder.start_record("TimeBound")?;
                timestamp.encode(encoder)?;
                encoder.end_record()
            }
        }
    }
}

impl NotaDecode for ChannelDuration {
    fn decode(decoder: &mut Decoder<'_>) -> nota_codec::Result<Self> {
        let head = decoder.peek_record_head()?;
        match head.as_str() {
            "OneShot" => {
                decoder.expect_record_head("OneShot")?;
                decoder.expect_record_end()?;
                Ok(Self::OneShot)
            }
            "Permanent" => {
                decoder.expect_record_head("Permanent")?;
                decoder.expect_record_end()?;
                Ok(Self::Permanent)
            }
            "TimeBound" => {
                decoder.expect_record_head("TimeBound")?;
                let timestamp = TimestampNanoseconds::decode(decoder)?;
                decoder.expect_record_end()?;
                Ok(Self::TimeBound(timestamp))
            }
            other => Err(nota_codec::Error::UnknownKindForVerb {
                verb: "ChannelDuration",
                got: other.to_string(),
            }),
        }
    }
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct ChannelGrant {
    pub source: ChannelEndpoint,
    pub destination: ChannelEndpoint,
    pub kinds: Vec<ChannelMessageKind>,
    pub duration: ChannelDuration,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct ChannelExtension {
    pub channel: ChannelId,
    pub duration: ChannelDuration,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct ChannelRevocation {
    pub channel: ChannelId,
    pub reason: TextBody,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct AdjudicationDenial {
    pub request: AdjudicationRequestIdentifier,
    pub reason: TextBody,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct ChannelGranted {
    pub channel: ChannelId,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct ChannelExtended {
    pub channel: ChannelId,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct ChannelRevoked {
    pub channel: ChannelId,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct AdjudicationDenied {
    pub request: AdjudicationRequestIdentifier,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct ChannelOrderRejected {
    pub operation: OperationKind,
    pub reason: ChannelOrderRejectionReason,
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEnum, Debug, Clone, Copy, PartialEq, Eq, Hash,
)]
pub enum ChannelOrderRejectionReason {
    OwnerAuthorityRequired,
    ChannelAlreadyExists,
    ChannelMissing,
    AdjudicationRequestMissing,
    PolicyRefused,
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEnum, Debug, Clone, Copy, PartialEq, Eq, Hash,
)]
pub enum OperationKind {
    Grant,
    Extend,
    Revoke,
    Deny,
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaEnum, Debug, Clone, Copy, PartialEq, Eq, Hash,
)]
pub enum UnimplementedReason {
    NotBuiltYet,
    DependencyNotReady,
    PolicyStoreUnavailable,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct RequestUnimplemented {
    pub operation: OperationKind,
    pub reason: UnimplementedReason,
}

signal_channel! {
    channel OwnerRouter {
        operation Grant(ChannelGrant),
        operation Extend(ChannelExtension),
        operation Revoke(ChannelRevocation),
        operation Deny(AdjudicationDenial),
    }
    reply OwnerRouterReply {
        ChannelGranted(ChannelGranted),
        ChannelExtended(ChannelExtended),
        ChannelRevoked(ChannelRevoked),
        AdjudicationDenied(AdjudicationDenied),
        ChannelOrderRejected(ChannelOrderRejected),
        RequestUnimplemented(RequestUnimplemented),
    }
}

pub type OwnerRouterRequest = OwnerRouterOperation;
pub type Frame = OwnerRouterFrame;
pub type FrameBody = OwnerRouterFrameBody;
pub type ChannelRequest = OwnerRouterChannelRequest;
pub type ChannelReply = OwnerRouterChannelReply;
pub type RequestBuilder = OwnerRouterRequestBuilder;

impl OwnerRouterOperation {
    pub fn operation_kind(&self) -> OperationKind {
        match self {
            Self::Grant(_) => OperationKind::Grant,
            Self::Extend(_) => OperationKind::Extend,
            Self::Revoke(_) => OperationKind::Revoke,
            Self::Deny(_) => OperationKind::Deny,
        }
    }
}

impl From<ChannelGrant> for OwnerRouterRequest {
    fn from(payload: ChannelGrant) -> Self {
        Self::Grant(payload)
    }
}

impl From<ChannelExtension> for OwnerRouterRequest {
    fn from(payload: ChannelExtension) -> Self {
        Self::Extend(payload)
    }
}

impl From<ChannelRevocation> for OwnerRouterRequest {
    fn from(payload: ChannelRevocation) -> Self {
        Self::Revoke(payload)
    }
}

impl From<AdjudicationDenial> for OwnerRouterRequest {
    fn from(payload: AdjudicationDenial) -> Self {
        Self::Deny(payload)
    }
}

impl From<ChannelGranted> for OwnerRouterReply {
    fn from(payload: ChannelGranted) -> Self {
        Self::ChannelGranted(payload)
    }
}

impl From<ChannelExtended> for OwnerRouterReply {
    fn from(payload: ChannelExtended) -> Self {
        Self::ChannelExtended(payload)
    }
}

impl From<ChannelRevoked> for OwnerRouterReply {
    fn from(payload: ChannelRevoked) -> Self {
        Self::ChannelRevoked(payload)
    }
}

impl From<AdjudicationDenied> for OwnerRouterReply {
    fn from(payload: AdjudicationDenied) -> Self {
        Self::AdjudicationDenied(payload)
    }
}

impl From<ChannelOrderRejected> for OwnerRouterReply {
    fn from(payload: ChannelOrderRejected) -> Self {
        Self::ChannelOrderRejected(payload)
    }
}

impl From<RequestUnimplemented> for OwnerRouterReply {
    fn from(payload: RequestUnimplemented) -> Self {
        Self::RequestUnimplemented(payload)
    }
}
