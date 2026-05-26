//! `GET /_matrix/client/*/rooms/{roomId}/delayed_events`
//!
//! Get all of the user's delayed events.

pub mod unstable {
    //! `msc4140` ([MSC])
    //!
    //! [MSC]: https://github.com/matrix-org/matrix-spec-proposals/pull/4140

    use ruma_common::{
        OwnedRoomId,
        api::{Direction, auth_scheme::AccessToken, request, response},
        metadata,
        serde::StringEnum,
    };
    use ruma_events::TimelineEventType;

    use crate::{
        PrivOwnedStr,
        delayed_events::{DelayedEventData, DelayedEventOutcome, DelayedEventStatus},
    };

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
        /// The order in which to display the events.
        #[ruma_api(query)]
        pub order_by: OrderBy,

        /// The direction to return events from.
        #[ruma_api(query)]
        #[serde(default = "Direction::forward")]
        pub dir: Direction,

        /// Pagination token which was returned in the next_batch property
        #[ruma_api(query)]
        #[serde(skip_serializing_if = "Option::is_none")]
        pub from: Option<String>,

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

        /// If specified, return only finalized events that were finalized with the specified
        /// outcome
        #[ruma_api(query)]
        #[serde(skip_serializing_if = "Option::is_none")]
        pub outcome: Option<DelayedEventOutcome>,
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

        /// A token that can be passed into a subsequent call to the endpoint to retrieve the
        /// previous page of results. Absent if paginating forward or when there is no previous
        /// page of results.
        #[serde(skip_serializing_if = "Option::is_none")]
        pub prev_batch: Option<String>,
    }

    impl Request {
        /// Create a new Request
        pub fn new() -> Self {
            Self {
                order_by: OrderBy::default(),
                dir: Direction::Forward,
                from: None,
                status: None,
                room_id: None,
                event_type: None,
                outcome: None,
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
            Self { delayed_events, next_batch: None, prev_batch: None }
        }
    }

    /// The order in which to display the events.
    #[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
    #[derive(Clone, StringEnum, Default)]
    #[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
    #[ruma_enum(rename_all = "snake_case")]
    pub enum OrderBy {
        /// The intended scheduled send time (running_since + delay) of the delayed event.
        #[default]
        SendTs,

        /// The time when the delayed event was finalized, or its scheduled send time if still
        /// scheduled.
        FinalizedTs,

        /// The time when the delayed event was scheduled or last restarted.
        RunningSince,

        #[doc(hidden)]
        _Custom(PrivOwnedStr),
    }

    #[cfg(all(test, feature = "client"))]
    mod client_tests {

        use std::borrow::Cow;

        use ruma_common::{
            api::{
                MatrixVersion, OutgoingRequest, SupportedVersions, auth_scheme::SendAccessToken,
            },
            owned_room_id,
        };

        use super::{OrderBy, Request};
        use crate::delayed_events::DelayedEventStatus;

        #[test]
        fn serialize_get_all_delayed_events_request() {
            let room_id = owned_room_id!("!roomid:example.org");
            let supported = SupportedVersions {
                versions: [MatrixVersion::V1_1].into(),
                features: Default::default(),
            };

            let mut req = Request::new();
            req.order_by = OrderBy::RunningSince;
            req.from = Some("next_batch_key".to_owned());
            req.status = Some(DelayedEventStatus::Finalized);
            req.room_id = Some(room_id);
            req.outcome = Some(crate::delayed_events::DelayedEventOutcome::Send);

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
                "order_by=running_since&dir=f&from=next_batch_key&status=finalised&room_id=%21roomid%3Aexample.org&outcome=send",
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
            response.prev_batch = Some("prev_batch_key".to_owned());

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
                    "next_batch" : "next_batch_key",
                    "prev_batch" : "prev_batch_key"
                }),
                serde_json::from_slice::<JsonValue>(response.body()).unwrap()
            );
        }
    }
}
