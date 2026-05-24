//! `GET /_matrix/client/*/rooms/{roomId}/delayed_events/{delay_id}`
//!
//! Get the information about a delayed event. [MSC4140](https://github.com/matrix-org/matrix-spec-proposals/pull/4140)

pub mod unstable {
    //! `msc4140` ([MSC])
    //!
    //! [MSC]: https://github.com/matrix-org/matrix-spec-proposals/pull/4140

    use std::time::Duration;

    use ruma_common::{
        MilliSecondsSinceUnixEpoch, OwnedEventId, OwnedRoomId,
        api::{auth_scheme::AccessToken, error::StandardErrorBody, request, response},
        metadata,
        serde::Raw,
    };
    use ruma_events::{AnyTimelineEventContent, TimelineEventType};

    use crate::delayed_events::DelayedEventData;

    metadata! {
        method: PUT,
        rate_limited: false,
        authentication: AccessToken,
        history: {
            unstable("org.matrix.msc4140") => "/_matrix/client/unstable/org.matrix.msc4140/delayed_events/{delay_id}",
        }
    }

    /// Request type for the [`get_delayed_event`](crate::delayed_events::get_delayed_event)
    /// endpoint
    #[request]
    pub struct Request {
        /// The ID of the requested delayed event.
        #[ruma_api(path)]
        pub delay_id: String,
    }

    /// Response type for the [`get_delayed_event`](crate::delayed_events::get_delayed_event)
    /// endpoint
    #[response]
    pub struct Response {
        /// The returned delayed event information
        #[ruma_api(body)]
        pub body: DelayedEventData,
    }

    impl Request {
        /// Create a new Request object
        pub fn new(delay_id: String) -> Self {
            Self { delay_id }
        }
    }

    impl Response {
        /// Create a new Response object
        pub fn new(
            delay_id: String,
            room_id: OwnedRoomId,
            event_type: TimelineEventType,
            state_key: Option<String>,
            delay: Duration,
            running_since: MilliSecondsSinceUnixEpoch,
            content: Raw<AnyTimelineEventContent>,
            error: Option<StandardErrorBody>,
            event_id: Option<OwnedEventId>,
            origin_server_ts: Option<MilliSecondsSinceUnixEpoch>,
        ) -> Self {
            Self {
                body: DelayedEventData {
                    delay_id,
                    room_id,
                    event_type,
                    state_key,
                    delay,
                    running_since,
                    content,
                    error,
                    event_id,
                    origin_server_ts,
                },
            }
        }
    }

    impl From<DelayedEventData> for Response {
        fn from(body: DelayedEventData) -> Self {
            Self { body }
        }
    }

    #[cfg(all(test, feature = "client"))]
    mod tests {
        use std::time::Duration;

        use js_int::UInt;
        use ruma_common::{
            MilliSecondsSinceUnixEpoch,
            api::{IncomingResponse, OutgoingResponse},
            owned_event_id, owned_room_id,
            serde::Raw,
        };
        use ruma_events::TimelineEventType;
        use serde_json::{Value as JsonValue, json};

        use super::Response;

        #[test]
        fn serialize_get_delayed_event_response() {
            let content = json!({
                "topic": "test topic"
            })
            .to_string();
            let response: http::Response<Vec<u8>> = Response::new(
                "a_delay_id".to_string(),
                owned_room_id!("!roomid:example.org"),
                TimelineEventType::RoomTopic,
                Some("a_state_key".to_string()),
                Duration::from_millis(103),
                MilliSecondsSinceUnixEpoch(UInt::new(70000).unwrap()),
                Raw::from_json_string(content).unwrap(),
                None,
                Some(owned_event_id!("$event:imaginary.hs")),
                Some(MilliSecondsSinceUnixEpoch(UInt::new(70103).unwrap())),
            )
            .try_into_http_response()
            .unwrap();

            assert_eq!(
                json!({
                    "content": {
                        "topic": "test topic"
                    },
                    "delay": 103,
                    "delay_id": "a_delay_id",
                    "event_id": "$event:imaginary.hs",
                    "origin_server_ts": 70103,
                    "room_id": "!roomid:example.org",
                    "running_since": 70000,
                    "state_key": "a_state_key",
                    "type": "m.room.topic"
                }),
                serde_json::from_str::<JsonValue>(std::str::from_utf8(response.body()).unwrap())
                    .unwrap()
            );
        }

        #[test]
        fn deserialize_update_delayed_events_request() {
            let body = json!({
                "content": {
                    "topic": "test topic"
                },
                "delay": 103,
                "delay_id": "a_delay_id",
                "event_id": "$event:imaginary.hs",
                "origin_server_ts": 70103,
                "room_id": "!roomid:example.org",
                "running_since": 70000,
                "state_key": "a_state_key",
                "type": "m.room.topic"
            })
            .to_string();

            let res =
                Response::try_from_http_response(http::Response::builder().body(body).unwrap())
                    .unwrap()
                    .body;

            let content = json!({
                "topic": "test topic"
            });

            assert_eq!(res.delay_id, "a_delay_id".to_string());
            assert_eq!(res.room_id, owned_room_id!("!roomid:example.org"));
            assert_eq!(res.event_type, TimelineEventType::RoomTopic);
            assert_eq!(res.state_key, Some("a_state_key".to_string()));
            assert_eq!(res.delay, Duration::from_millis(103));
            assert_eq!(res.running_since, MilliSecondsSinceUnixEpoch(UInt::new(70000).unwrap()));
            assert_eq!(
                serde_json::from_str::<JsonValue>(res.content.json().get()).unwrap(),
                content
            );
            assert!(res.error.is_none());
            assert_eq!(res.event_id, Some(owned_event_id!("$event:imaginary.hs")));
            assert_eq!(
                res.origin_server_ts,
                Some(MilliSecondsSinceUnixEpoch(UInt::new(70103).unwrap()))
            );
        }
    }
}
