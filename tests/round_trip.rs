use nota_codec::{Decoder, Encoder, NotaDecode, NotaEncode};
use owner_signal_persona_router::{
    AdjudicationDenial, AdjudicationDenied, AdjudicationRequestIdentifier, ChannelDuration,
    ChannelEndpoint, ChannelExtended, ChannelExtension, ChannelGrant, ChannelGranted,
    ChannelMessageKind, ChannelOrderRejected, ChannelOrderRejectionReason, ChannelRevocation,
    ChannelRevoked, Frame, FrameBody, OperationKind, OwnerRouterChannelRequest, OwnerRouterReply,
    OwnerRouterRequest, RequestUnimplemented, TextBody, TimestampNanoseconds, UnimplementedReason,
};
use signal_frame::{
    ExchangeIdentifier, ExchangeLane, LaneSequence, NonEmpty, Reply, RequestPayload, SessionEpoch,
    SubReply,
};
use signal_persona_auth::{ChannelId, ComponentName, ConnectionClass};

fn exchange() -> ExchangeIdentifier {
    ExchangeIdentifier::new(
        SessionEpoch::new(1),
        ExchangeLane::Connector,
        LaneSequence::first(),
    )
}

fn channel() -> ChannelId {
    ChannelId::new("channel-aab")
}

fn adjudication_request() -> AdjudicationRequestIdentifier {
    AdjudicationRequestIdentifier::new("adjudication-aab")
}

fn internal_endpoint(component: ComponentName) -> ChannelEndpoint {
    ChannelEndpoint::Internal(component)
}

fn external_owner_endpoint() -> ChannelEndpoint {
    ChannelEndpoint::External(ConnectionClass::Owner)
}

fn grant() -> ChannelGrant {
    ChannelGrant {
        source: external_owner_endpoint(),
        destination: internal_endpoint(ComponentName::Router),
        kinds: vec![
            ChannelMessageKind::MessageSubmission,
            ChannelMessageKind::InboxQuery,
        ],
        duration: ChannelDuration::Permanent,
    }
}

fn round_trip_request(request: OwnerRouterRequest) -> OwnerRouterRequest {
    let frame = Frame::new(FrameBody::Request {
        exchange: exchange(),
        request: request.into_request(),
    });
    let bytes = frame.encode_length_prefixed().expect("encode");
    let decoded = Frame::decode_length_prefixed(&bytes).expect("decode");
    match decoded.into_body() {
        FrameBody::Request { request, .. } => request.payloads().head().clone(),
        other => panic!("expected request operation, got {other:?}"),
    }
}

fn round_trip_reply(reply: OwnerRouterReply) -> OwnerRouterReply {
    let frame = Frame::new(FrameBody::Reply {
        exchange: exchange(),
        reply: Reply::committed(NonEmpty::single(SubReply::Ok(reply))),
    });
    let bytes = frame.encode_length_prefixed().expect("encode");
    let decoded = Frame::decode_length_prefixed(&bytes).expect("decode");
    match decoded.into_body() {
        FrameBody::Reply { reply, .. } => match reply {
            Reply::Accepted { per_operation, .. } => match per_operation.into_head() {
                SubReply::Ok(payload) => payload,
                other => panic!("expected accepted reply payload, got {other:?}"),
            },
            other => panic!("expected accepted reply, got {other:?}"),
        },
        other => panic!("expected reply operation, got {other:?}"),
    }
}

fn round_trip_nota<T>(value: T, expected: &str)
where
    T: NotaEncode + NotaDecode + PartialEq + std::fmt::Debug,
{
    let mut encoder = Encoder::new();
    value.encode(&mut encoder).expect("encode nota text");
    let encoded = encoder.into_string();
    assert_eq!(encoded, expected);

    let mut decoder = Decoder::new(&encoded);
    let recovered = T::decode(&mut decoder).expect("decode nota text");
    assert_eq!(recovered, value);
}

#[test]
fn owner_router_requests_round_trip() {
    let requests = vec![
        OwnerRouterRequest::Grant(grant()),
        OwnerRouterRequest::Extend(ChannelExtension {
            channel: channel(),
            duration: ChannelDuration::TimeBound(TimestampNanoseconds::new(
                1_730_000_000_000_000_000,
            )),
        }),
        OwnerRouterRequest::Revoke(ChannelRevocation {
            channel: channel(),
            reason: TextBody::new("operator closed the path"),
        }),
        OwnerRouterRequest::Deny(AdjudicationDenial {
            request: adjudication_request(),
            reason: TextBody::new("destination unavailable"),
        }),
    ];

    for request in requests {
        assert_eq!(round_trip_request(request.clone()), request);
    }
}

#[test]
fn owner_router_replies_round_trip() {
    let replies = vec![
        OwnerRouterReply::ChannelGranted(ChannelGranted { channel: channel() }),
        OwnerRouterReply::ChannelExtended(ChannelExtended { channel: channel() }),
        OwnerRouterReply::ChannelRevoked(ChannelRevoked { channel: channel() }),
        OwnerRouterReply::AdjudicationDenied(AdjudicationDenied {
            request: adjudication_request(),
        }),
        OwnerRouterReply::ChannelOrderRejected(ChannelOrderRejected {
            operation: OperationKind::Grant,
            reason: ChannelOrderRejectionReason::PolicyRefused,
        }),
        OwnerRouterReply::RequestUnimplemented(RequestUnimplemented {
            operation: OperationKind::Grant,
            reason: UnimplementedReason::NotBuiltYet,
        }),
    ];

    for reply in replies {
        assert_eq!(round_trip_reply(reply.clone()), reply);
    }
}

#[test]
fn owner_router_operations_encode_as_contract_local_nota_heads() {
    let operation = OwnerRouterRequest::Grant(grant());
    let mut encoder = Encoder::new();
    operation
        .into_request()
        .encode(&mut encoder)
        .expect("encode");
    let text = encoder.into_string();

    assert_eq!(
        text,
        "(Grant ((External (Owner)) (Internal Router) [MessageSubmission InboxQuery] (Permanent)))"
    );
    assert!(!text.contains("Mutate"));
    assert!(!text.contains("Retract"));
    assert!(!text.contains("Assert"));

    let mut decoder = Decoder::new(&text);
    let decoded = OwnerRouterChannelRequest::decode(&mut decoder).expect("decode");
    assert_eq!(
        decoded.payloads().head().operation_kind(),
        OperationKind::Grant
    );
}

#[test]
fn owner_router_request_exposes_contract_owned_operation_kind() {
    let cases = vec![
        (OwnerRouterRequest::Grant(grant()), OperationKind::Grant),
        (
            OwnerRouterRequest::Extend(ChannelExtension {
                channel: channel(),
                duration: ChannelDuration::OneShot,
            }),
            OperationKind::Extend,
        ),
        (
            OwnerRouterRequest::Revoke(ChannelRevocation {
                channel: channel(),
                reason: TextBody::new("operator closed the path"),
            }),
            OperationKind::Revoke,
        ),
        (
            OwnerRouterRequest::Deny(AdjudicationDenial {
                request: adjudication_request(),
                reason: TextBody::new("destination unavailable"),
            }),
            OperationKind::Deny,
        ),
    ];

    for (request, operation) in cases {
        assert_eq!(request.operation_kind(), operation);
    }
}

#[test]
fn owner_order_names_are_not_channel_message_kinds() {
    for kind in ChannelMessageKind::ALL {
        let mut encoder = Encoder::new();
        kind.encode(&mut encoder).expect("encode");
        let encoded = encoder.into_string();

        assert_ne!(encoded, "ChannelGrant");
        assert_ne!(encoded, "ChannelExtend");
        assert_ne!(encoded, "ChannelRetract");
        assert_ne!(encoded, "AdjudicationDeny");
        assert_ne!(encoded, "AdjudicationDenial");
    }

    round_trip_nota(
        ChannelMessageKind::MessageIngressSubmission,
        "MessageIngressSubmission",
    );
    round_trip_nota(ChannelMessageKind::MessageSubmission, "MessageSubmission");
}
