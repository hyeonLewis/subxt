// Copyright 2019-2022 Parity Technologies (UK) Ltd.
// This file is dual-licensed as Apache-2.0 or GPL-3.0.
// see LICENSE for license details.

//! This module exposes the ability to work with events generated by a given block.
//! Subxt can either attempt to statically decode events into known types, or it
//! can hand back details of the raw event without knowing what shape its contents
//! are (this may be useful if we don't know what exactly we're looking for).
//!
//! This module is wrapped by the generated API in `RuntimeAPI::EventsApi`.
//!
//! # Examples
//!
//! ## Subscribe to all events
//!
//! Users can subscribe to all emitted events from blocks using `subscribe()`.
//!
//! To subscribe to all events from just the finalized blocks use `subscribe_finalized()`.
//!
//! To obtain the events from a given block use `at()`.
//!
//! ```no_run
//! # use futures::StreamExt;
//! # use subxt::{ClientBuilder, DefaultConfig, PolkadotExtrinsicParams};
//! # #[subxt::subxt(runtime_metadata_path = "../artifacts/polkadot_metadata.scale")]
//! # pub mod polkadot {}
//! # #[tokio::main]
//! # async fn main() {
//! # let api = ClientBuilder::new()
//! #     .build()
//! #     .await
//! #     .unwrap()
//! #     .to_runtime_api::<polkadot::RuntimeApi<DefaultConfig, PolkadotExtrinsicParams<DefaultConfig>>>();
//! let mut events = api.events().subscribe().await.unwrap();
//!
//! while let Some(ev) = events.next().await {
//!     // Obtain all events from this block.
//!     let ev: subxt::Events<_, _> = ev.unwrap();
//!     // Print block hash.
//!     println!("Event at block hash {:?}", ev.block_hash());
//!     // Iterate over all events.
//!     let mut iter = ev.iter();
//!     while let Some(event_details) = iter.next() {
//!         println!("Event details {:?}", event_details);
//!     }
//! }
//! # }
//! ```
//!
//! ## Filter events
//!
//! The subxt exposes the ability to filter events via the `filter_events()` function.
//!
//! The function filters events from the provided tuple. If 1-tuple is provided, the events are
//! returned directly. Otherwise, we'll be given a corresponding tuple of `Option`'s, with exactly
//! one variant populated each time.
//!
//! ```no_run
//! # use futures::StreamExt;
//! # use subxt::{ClientBuilder, DefaultConfig, PolkadotExtrinsicParams};
//!
//! #[subxt::subxt(runtime_metadata_path = "../artifacts/polkadot_metadata.scale")]
//! pub mod polkadot {}
//!
//! # #[tokio::main]
//! # async fn main() {
//! # let api = ClientBuilder::new()
//! #     .build()
//! #     .await
//! #     .unwrap()
//! #     .to_runtime_api::<polkadot::RuntimeApi<DefaultConfig, PolkadotExtrinsicParams<DefaultConfig>>>();
//!
//! let mut transfer_events = api
//!     .events()
//!     .subscribe()
//!     .await
//!     .unwrap()
//!     .filter_events::<(polkadot::balances::events::Transfer,)>();
//!
//! while let Some(transfer_event) = transfer_events.next().await {
//!     println!("Balance transfer event: {transfer_event:?}");
//! }
//! # }
//! ```

mod event_subscription;
mod events_type;
mod filter_events;

pub use event_subscription::{
    EventSub,
    EventSubscription,
    FinalizedEventSub,
};
pub use events_type::{
    DecodedValue,
    EventDetails,
    Events,
    RawEventDetails,
};
pub use filter_events::{
    EventFilter,
    FilterEvents,
    FilteredEventDetails,
};
