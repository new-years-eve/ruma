//! `GET /_matrix/client/*/rooms/{roomId}/delayed_events`
//!
//! Get all of the user's delayed events.

pub mod unstable {
    //! `msc4140` ([MSC])
    //!
    //! [MSC]: https://github.com/matrix-org/matrix-spec-proposals/pull/4140

    use ruma_common::{
        MilliSecondsSinceUnixEpoch, OwnedRoomId,
        api::{Direction, auth_scheme::AccessToken, request, response},
        metadata,
    };
    use ruma_events::TimelineEventType;

    use crate::delayed_events::{DelayedEventData, DelayedEventStatus};

    metadata! {
        method: GET,
        rate_limited: true,
        authentication: AccessToken,
        history: {
            unstable("org.matrix.msc4140") => "/_matrix/client/unstable/org.matrix.msc4140/delayed_events",
        }
    }

    /// Request type for the
    /// [`get_all_delayed_events`](crate::delayed_events::get_all_delayed_events) endpoint
    ///
    /// The details of this endpoint are still under discussion and subject to change.
    /// Some of the fields specified may be removed. For now, the fields `reason`
    /// and `since_ts` in the MSC have not been included.
    #[request]
    pub struct Request {
        /// The direction to return events from.
        #[ruma_api(query)]
        #[serde(default = "Direction::forward")]
        pub dir: Direction,

        /// Pagination token which was returned in the next_batch property
        #[ruma_api(query)]
        #[serde(skip_serializing_if = "Option::is_none")]
        pub from: Option<String>,

        /// If provided, only return events scheduled to be sent after this time
        #[ruma_api(query)]
        #[serde(skip_serializing_if = "Option::is_none")]
        pub from_ts: Option<MilliSecondsSinceUnixEpoch>,

        /// If provided, only return events scheduled to be sent before this time
        #[ruma_api(query)]
        #[serde(skip_serializing_if = "Option::is_none")]
        pub to_ts: Option<MilliSecondsSinceUnixEpoch>,

        /// If specified, return only events of the specified status
        #[ruma_api(query)]
        #[serde(skip_serializing_if = "Option::is_none")]
        pub status: Option<DelayedEventStatus>,

        /// If specified, return only events that were submitted to the specified room
        #[ruma_api(query)]
        #[serde(skip_serializing_if = "Option::is_none")]
        pub room_id: Option<OwnedRoomId>,

        /// If specified, return only events of the specified type
        #[ruma_api(query)]
        #[serde(rename = "type")]
        #[serde(skip_serializing_if = "Option::is_none")]
        pub event_type: Option<TimelineEventType>,
    }

    /// Response type for the
    /// [`get_all_delayed_events`](crate::delayed_events::get_all_delayed_events) endpoint
    #[response]
    pub struct Response {
        /// An array of objects describing delayed events owned by the requesting user that match
        /// the filters provided in the request, in the order specified by the order_by and dir
        /// query parameters.
        pub delayed_events: Vec<DelayedEventData>,

        /// A token that can be passed into a subsequent call to the endpoint to retrieve the next
        /// page of results. Absent if paginating backwards or when there is no next page of
        /// results.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub next_batch: Option<String>,
    }

    impl Request {
        /// Create a new Request
        pub fn new() -> Self {
            Self {
                dir: Direction::Forward,
                from: None,
                from_ts: None,
                to_ts: None,
                status: None,
                room_id: None,
                event_type: None,
            }
        }
    }

    impl Default for Request {
        fn default() -> Self {
            Self::new()
        }
    }

    impl Response {
        /// Create a new Response.
        pub fn new(delayed_events: Vec<DelayedEventData>) -> Self {
            Self { delayed_events, next_batch: None }
        }
    }

    #[cfg(all(test, feature = "client"))]
    mod client_tests {

        use std::borrow::Cow;

        use ruma_common::{
            MilliSecondsSinceUnixEpoch,
            api::{
                MatrixVersion, OutgoingRequest, SupportedVersions, auth_scheme::SendAccessToken,
            },
            owned_room_id,
        };

        use super::Request;
        use crate::delayed_events::DelayedEventStatus;

        #[test]
        fn serialize_get_all_delayed_events_request() {
            let room_id = owned_room_id!("!roomid:example.org");
            let supported = SupportedVersions {
                versions: [MatrixVersion::V1_1].into(),
                features: Default::default(),
            };

            let mut req = Request::new();
            req.from = Some("next_batch_key".to_owned());
            req.status = Some(DelayedEventStatus::Error);
            req.room_id = Some(room_id);
            req.to_ts = Some(MilliSecondsSinceUnixEpoch(555000.try_into().unwrap()));

            let request: http::Request<Vec<u8>> = req
                .try_into_http_request(
                    "https://homeserver.tld",
                    SendAccessToken::IfRequired("auth_tok"),
                    Cow::Owned(supported),
                )
                .unwrap();
            let (parts, _body) = request.into_parts();

            assert_eq!(
                "/_matrix/client/unstable/org.matrix.msc4140/delayed_events",
                parts.uri.path()
            );
            assert_eq!("GET", parts.method.to_string());
            assert_eq!(
                "dir=f&from=next_batch_key&to_ts=555000&status=error&room_id=%21roomid%3Aexample.org",
                parts.uri.query().unwrap()
            );
        }
    }

    #[cfg(all(test, feature = "server"))]
    mod server_tests {

        use std::time::Duration;

        use js_int::UInt;
        use ruma_common::{
            MilliSecondsSinceUnixEpoch, api::OutgoingResponse, owned_event_id, owned_room_id,
            serde::Raw,
        };
        use ruma_events::TimelineEventType;
        use serde_json::{Value as JsonValue, json};

        use super::Response;
        use crate::delayed_events::DelayedEventData;

        #[test]
        fn serialize_get_all_delayed_events_response() {
            let content = json!({
                "topic": "test topic"
            })
            .to_string();

            let mut event0 = DelayedEventData::new(
                "a_delay_id".to_owned(),
                owned_room_id!("!roomid:example.org"),
                TimelineEventType::RoomTopic,
                Some("a_state_key".to_owned()),
                Raw::from_json_string(content).unwrap(),
                Duration::from_millis(103),
                MilliSecondsSinceUnixEpoch(UInt::new(70000).unwrap()),
            );

            event0.event_id = Some(owned_event_id!("$event:imaginary.hs"));
            event0.finalized_ts = Some(MilliSecondsSinceUnixEpoch(UInt::new(70103).unwrap()));

            let mut response = Response::new(vec![event0]);
            response.next_batch = Some("next_batch_key".to_owned());

            let response: http::Response<Vec<u8>> = response.try_into_http_response().unwrap();

            assert_eq!(
                json!({
                    "delayed_events" : [{
                        "content": {
                            "topic": "test topic"
                        },
                        "delay": 103,
                        "delay_id": "a_delay_id",
                        "event_id": "$event:imaginary.hs",
                        "finalised_ts": 70103,
                        "room_id": "!roomid:example.org",
                        "running_since": 70000,
                        "state_key": "a_state_key",
                        "type": "m.room.topic"
                        }],
                    "next_batch" : "next_batch_key"
                }),
                serde_json::from_slice::<JsonValue>(response.body()).unwrap()
            );
        }
    }
}
