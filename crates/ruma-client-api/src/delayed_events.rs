//! Endpoints for sending and interacting with delayed events.

pub mod get_all_delayed_events;
pub mod get_delayed_event;
pub mod send_delayed_event;
pub mod update_delayed_event;

use std::time::Duration;

use ruma_common::{
    MilliSecondsSinceUnixEpoch, OwnedEventId, OwnedRoomId,
    api::error::StandardErrorBody,
    serde::{Raw, StringEnum},
};
use ruma_events::{AnyTimelineEventContent, TimelineEventType};
use serde::{Deserialize, Serialize};

use crate::PrivOwnedStr;

/// The structure of the data for returning a delayed event from a GET endpoint
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
pub struct DelayedEventData {
    /// The ID of the delayed event.
    pub delay_id: String,

    /// The ID of the room that the delayed event was scheduled to be sent in.
    pub room_id: OwnedRoomId,

    /// The event type of the delayed event.
    #[serde(rename = "type")]
    pub event_type: TimelineEventType,

    /// The State Key if the event is a state event, nothing otherwise
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    pub state_key: Option<String>,

    /// The duration that the server should wait before sending this event
    #[serde(with = "ruma_common::serde::duration::ms")]
    pub delay: Duration,

    /// The timestamp when the delayed event was scheduled or last restarted.
    pub running_since: MilliSecondsSinceUnixEpoch,

    /// The event content to send.
    /// This is the content that was submitted to the send endpoint, not the content of the final
    /// event
    pub content: Raw<AnyTimelineEventContent>,

    /// Present only for finalised events that were cancelled due to an error.
    /// The error that prevented the delayed event from being sent.
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    pub error: Option<StandardErrorBody>,

    /// Present only for events that were sent succesfully.
    /// The event_id this event got when it was sent.
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    pub event_id: Option<OwnedEventId>,

    /// Present only for events that were finalized (sent, failed to send, or cancelled).
    /// The timestamp when the event was finalised;
    #[serde(skip_serializing_if = "std::option::Option::is_none")]
    pub origin_server_ts: Option<MilliSecondsSinceUnixEpoch>,
}

impl DelayedEventData {
    /// Create a new delayed event data object with the given parameters
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
        }
    }
}

/// The status that a delayed event stored on the server can have.
#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
#[derive(Clone, StringEnum)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
#[ruma_enum(rename_all = "snake_case")]
pub enum DelayedEventStatus {
    /// The event is currently scheduled to be submitted at a later date.
    /// It may be restarted, sent or cancelled via the management endpoint.
    Scheduled,

    /// The event has been sent, canceled, or has failed to send.
    /// No further action will be taken with this event.
    Finalized,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}

/// The outcome that a finalized delayed event can have.
#[doc = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/doc/string_enum.md"))]
#[derive(Clone, StringEnum)]
#[cfg_attr(not(ruma_unstable_exhaustive_types), non_exhaustive)]
#[ruma_enum(rename_all = "snake_case")]
pub enum DelayedEventOutcome {
    /// The event has been sent succesfully
    Send,

    /// The event has been cancelled
    Cancel,

    /// The event has encountered an error when trying to send
    Error,

    #[doc(hidden)]
    _Custom(PrivOwnedStr),
}
