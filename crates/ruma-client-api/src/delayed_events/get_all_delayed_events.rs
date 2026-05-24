//! `GET /_matrix/client/*/rooms/{roomId}/delayed_events`
//!
//! Get all of the user's delayed events. [MSC4140](https://github.com/matrix-org/matrix-spec-proposals/pull/4140)

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
        rate_limited: false,
        authentication: AccessToken,
        history: {
            unstable("org.matrix.msc4140") => "/_matrix/client/unstable/org.matrix.msc4140/delayed_events",
        }
    }

    /// Request type for the
    /// [`get_all_delayed_events`](crate::delayed_events::get_all_delayed_events) endpoint
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
        pub from: Option<String>,

        // Note: the MSC specifies a `since_ts` parameter in the query.
        // This parameter is currently not clearly defined by the MSC, and so will be ommited
        // until the MSC is clarified.
        /// If specified, return only events of the specified status
        #[ruma_api(query)]
        pub status: Option<DelayedEventStatus>,

        /// If specified, return only events that were submitted to the specified room
        #[ruma_api(query)]
        pub room_id: Option<OwnedRoomId>,

        /// If specified, return only events of the specified type
        #[ruma_api(query)]
        #[serde(rename = "type")]
        pub event_type: Option<TimelineEventType>,

        /// If specified, return only finalized events that were finalized with the specified
        /// outcome
        #[ruma_api(query)]
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
        #[serde(skip_serializing_if = "std::option::Option::is_none")]
        pub next_batch: Option<String>,

        /// A token that can be passed into a subsequent call to the endpoint to retrieve the
        /// previous page of results. Absent if paginating forward or when there is no previous
        /// page of results.
        #[serde(skip_serializing_if = "std::option::Option::is_none")]
        pub prev_batch: Option<String>,
    }

    impl Response {
        /// Create a new Response.
        pub fn new(
            delayed_events: Vec<DelayedEventData>,
            next_batch: Option<String>,
            prev_batch: Option<String>,
        ) -> Self {
            Self {
                delayed_events,
                next_batch,
                prev_batch,
            }
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

        /// The time when the delayed event was finalised, or its scheduled send time if still
        /// scheduled.
        OriginServerTs,

        /// The time when the delayed event was scheduled or last restarted.
        RunningSince,

        #[doc(hidden)]
        _Custom(PrivOwnedStr),
    }
}
