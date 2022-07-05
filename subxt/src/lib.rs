// Copyright 2019-2022 Parity Technologies (UK) Ltd.
// This file is dual-licensed as Apache-2.0 or GPL-3.0.
// see LICENSE for license details.

//! Subxt is a library to **sub**mit e**xt**rinsics to a [substrate](https://github.com/paritytech/substrate) node via RPC.
//!
//! The generated Subxt API exposes the ability to:
//! - [Submit extrinsics](https://docs.substrate.io/v3/concepts/extrinsics/) (Calls)
//! - [Query storage](https://docs.substrate.io/v3/runtime/storage/) (Storage)
//! - [Query constants](https://docs.substrate.io/how-to-guides/v3/basics/configurable-constants/) (Constants)
//! - [Subscribe to events](https://docs.substrate.io/v3/runtime/events-and-errors/) (Events)
//!
//!
//! # Generate the runtime API
//!
//! Subxt generates a runtime API from downloaded static metadata. The metadata can be downloaded using the
//! [subxt-cli](https://crates.io/crates/subxt-cli) tool.
//!
//! To generate the runtime API, use the `subxt` attribute which points at downloaded static metadata.
//!
//! ```ignore
//! #[subxt::subxt(runtime_metadata_path = "metadata.scale")]
//! pub mod node_runtime { }
//! ```
//!
//! The `node_runtime` has the following hierarchy:
//!
//! ```rust
//! pub mod node_runtime {
//!     pub mod PalletName {
//!         pub mod calls { }
//!         pub mod storage { }
//!         pub mod constants { }
//!         pub mod events { }
//!     }
//! }
//! ```
//!
//! For more information regarding the `node_runtime` hierarchy, please visit the
//! [subxt-codegen](https://docs.rs/subxt-codegen/latest/subxt_codegen/) documentation.
//!
//!
//! # Initializing the API client
//!
//! ```no_run
//! use subxt::{ClientBuilder, DefaultConfig, PolkadotExtrinsicParams};
//!
//! #[subxt::subxt(runtime_metadata_path = "../artifacts/polkadot_metadata.scale")]
//! pub mod polkadot {}
//!
//! # #[tokio::main]
//! # async fn main() {
//! let api = ClientBuilder::new()
//!     .set_url("wss://rpc.polkadot.io:443")
//!     .build()
//!     .await
//!     .unwrap()
//!     .to_runtime_api::<polkadot::RuntimeApi<DefaultConfig, PolkadotExtrinsicParams<DefaultConfig>>>();
//! # }
//! ```
//!
//! The `RuntimeApi` type is generated by the `subxt` macro from the supplied metadata. This can be parameterized with user
//! supplied implementations for the `Config` and `Extra` types, if the default implementation differs from the target
//! chain.
//!
//! To ensure that extrinsics are properly submitted, during the build phase of the Client the
//! runtime metadata of the node is downloaded. If the URL is not specified (`set_url`), the local host is used instead.
//!
//!
//! # Submit Extrinsics
//!
//! Extrinsics are obtained using the API's `RuntimeApi::tx()` method, followed by `pallet_name()` and then the
//! `call_item_name()`.
//!
//! Submit an extrinsic, returning success once the transaction is validated and accepted into the pool:
//!
//! Please visit the [balance_transfer](../examples/examples/balance_transfer.rs) example for more details.
//!
//!
//! # Querying Storage
//!
//! The runtime storage is queried via the generated `RuntimeApi::storage()` method, followed by the `pallet_name()` and
//! then the `storage_item_name()`.
//!
//! Please visit the [fetch_staking_details](../examples/examples/fetch_staking_details.rs) example for more details.
//!
//! # Query Constants
//!
//! Constants are embedded into the node's metadata.
//!
//! The subxt offers the ability to query constants from the runtime metadata (metadata downloaded when constructing
//! the client, *not* the one provided for API generation).
//!
//! To query constants use the generated `RuntimeApi::constants()` method, followed by the `pallet_name()` and then the
//! `constant_item_name()`.
//!
//! Please visit the [fetch_constants](../examples/examples/fetch_constants.rs) example for more details.
//!
//! # Subscribe to Events
//!
//! To subscribe to events, use the generated `RuntimeApi::events()` method which exposes:
//! - `subscribe()` - Subscribe to events emitted from blocks. These blocks haven't necessarily been finalised.
//! - `subscribe_finalized()` - Subscribe to events from finalized blocks.
//! - `at()` - Obtain events at a given block hash.
//!
//!
//! *Examples*
//! - [subscribe_all_events](../examples/examples/subscribe_all_events.rs): Subscribe to events emitted from blocks.
//! - [subscribe_one_event](../examples/examples/subscribe_one_event.rs): Subscribe and filter by one event.
//! - [subscribe_some_events](../examples/examples/subscribe_some_events.rs): Subscribe and filter event.
//!
//! # Static Metadata Validation
//!
//! There are two types of metadata that the subxt is aware of:
//! - static metadata: Metadata used for generating the API.
//! - runtime metadata: Metadata downloaded from the target node when a subxt client is created.
//!
//! There are cases when the static metadata is different from the runtime metadata of a node.
//! Such is the case when the node performs a runtime update.
//!
//! To ensure that subxt can properly communicate with the target node the static metadata is validated
//! against the runtime metadata of the node.
//!
//! This validation is performed at the Call, Constant, and Storage levels, as well for the entire metadata.
//! The level of granularity ensures that the users can still submit a given call, even if another
//! call suffered changes.
//!
//! Full metadata validation:
//!
//! ```no_run
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
//! // To make sure that all of our statically generated pallets are compatible with the
//! // runtime node, we can run this check:
//! api.validate_metadata().unwrap();
//! # }
//! ```
//!
//! Call level validation:
//!
//! ```ignore
//! # use sp_keyring::AccountKeyring;
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
//! // Submit the `transfer` extrinsic from Alice's account to Bob's.
//! let dest = AccountKeyring::Bob.to_account_id().into();
//!
//! let extrinsic = api
//!     .tx()
//!     .balances()
//!     // Constructing an extrinsic will fail if the metadata
//!     // is not in sync with the generated API.
//!     .transfer(dest, 123_456_789_012_345)
//!     .unwrap();
//! # }
//! ```
//!
//! # Runtime Updates
//!
//! There are cases when the node would perform a runtime update, and the runtime node's metadata would be
//! out of sync with the subxt's metadata.
//!
//! The `UpdateClient` API keeps the `RuntimeVersion` and `Metadata` of the client synced with the target node.
//!
//! Please visit the [subscribe_runtime_updates](../examples/examples/subscribe_runtime_updates.rs) example for more details.

#![deny(
    bad_style,
    const_err,
    improper_ctypes,
    missing_docs,
    non_shorthand_field_patterns,
    no_mangle_generic_items,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    private_in_public,
    unconditional_recursion,
    unused_allocation,
    unused_comparisons,
    unused_parens,
    while_true,
    trivial_casts,
    trivial_numeric_casts,
    unused_crate_dependencies,
    unused_extern_crates,
    clippy::all
)]
#![allow(clippy::type_complexity)]

pub use frame_metadata::StorageHasher;
pub use subxt_macro::subxt;

pub use bitvec;
pub use codec;
pub use sp_core;
pub use sp_runtime;

use codec::{
    Decode,
    DecodeAll,
    Encode,
};
use core::fmt::Debug;
use derivative::Derivative;

pub mod client;
pub mod config;
pub mod error;
pub mod events;
pub mod extrinsic;
pub mod metadata;
pub mod rpc;

pub use crate::{
    client::{
        OfflineClient,
        OnlineClient,
    },
    config::{
        Config,
        SubstrateConfig,
    },
    error::{
        BasicError,
        Error,
        GenericError,
        HasModuleError,
        ModuleError,
        ModuleErrorData,
        RuntimeError,
        TransactionError,
    },
    events::{
        EventDetails,
        Events,
        RawEventDetails,
    },
    extrinsic::{
        PairSigner,
        PolkadotExtrinsicParams,
        PolkadotExtrinsicParamsBuilder,
        SubstrateExtrinsicParams,
        SubstrateExtrinsicParamsBuilder,
        TransactionEvents,
        TransactionInBlock,
        TransactionProgress,
        TransactionStatus,
    },
    metadata::{
        ErrorMetadata,
        Metadata,
        MetadataError,
        PalletMetadata,
    },
    rpc::{
        BlockNumber,
        ReadProof,
        RpcClient,
        SystemProperties,
    },
};

/// Trait to uniquely identify the call (extrinsic)'s identity from the runtime metadata.
///
/// Generated API structures that represent each of the different possible
/// calls to a node each implement this trait.
///
/// When encoding an extrinsic, we use this information to know how to map
/// the call to the specific pallet and call index needed by a particular node.
pub trait Call: Encode {
    /// Pallet name.
    const PALLET: &'static str;
    /// Function name.
    const FUNCTION: &'static str;

    /// Returns true if the given pallet and function names match this call.
    fn is_call(pallet: &str, function: &str) -> bool {
        Self::PALLET == pallet && Self::FUNCTION == function
    }
}

/// Trait to uniquely identify the events's identity from the runtime metadata.
///
/// Generated API structures that represent an event implement this trait.
///
/// The trait is utilized to decode emitted events from a block, via obtaining the
/// form of the `Event` from the metadata.
pub trait Event: Decode {
    /// Pallet name.
    const PALLET: &'static str;
    /// Event name.
    const EVENT: &'static str;

    /// Returns true if the given pallet and event names match this event.
    fn is_event(pallet: &str, event: &str) -> bool {
        Self::PALLET == pallet && Self::EVENT == event
    }
}

/// Wraps an already encoded byte vector, prevents being encoded as a raw byte vector as part of
/// the transaction payload
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Encoded(pub Vec<u8>);

impl codec::Encode for Encoded {
    fn encode(&self) -> Vec<u8> {
        self.0.to_owned()
    }
}

/// A phase of a block's execution.
#[derive(Clone, Debug, Eq, PartialEq, Decode, Encode)]
pub enum Phase {
    /// Applying an extrinsic.
    ApplyExtrinsic(u32),
    /// Finalizing the block.
    Finalization,
    /// Initializing the block.
    Initialization,
}

/// A wrapper for any type `T` which implement encode/decode in a way compatible with `Vec<u8>`.
///
/// [`WrapperKeepOpaque`] stores the type only in its opaque format, aka as a `Vec<u8>`. To
/// access the real type `T` [`Self::try_decode`] needs to be used.
#[derive(Derivative, Encode, Decode)]
#[derivative(
    Debug(bound = ""),
    Clone(bound = ""),
    PartialEq(bound = ""),
    Eq(bound = ""),
    Default(bound = ""),
    Hash(bound = "")
)]
pub struct WrapperKeepOpaque<T> {
    data: Vec<u8>,
    _phantom: PhantomDataSendSync<T>,
}

impl<T: Decode> WrapperKeepOpaque<T> {
    /// Try to decode the wrapped type from the inner `data`.
    ///
    /// Returns `None` if the decoding failed.
    pub fn try_decode(&self) -> Option<T> {
        T::decode_all(&mut &self.data[..]).ok()
    }

    /// Returns the length of the encoded `T`.
    pub fn encoded_len(&self) -> usize {
        self.data.len()
    }

    /// Returns the encoded data.
    pub fn encoded(&self) -> &[u8] {
        &self.data
    }

    /// Create from the given encoded `data`.
    pub fn from_encoded(data: Vec<u8>) -> Self {
        Self {
            data,
            _phantom: PhantomDataSendSync::new(),
        }
    }
}

/// A version of [`std::marker::PhantomData`] that is also Send and Sync (which is fine
/// because regardless of the generic param, it is always possible to Send + Sync this
/// 0 size type).
#[derive(Derivative, Encode, Decode, scale_info::TypeInfo)]
#[derivative(
    Clone(bound = ""),
    PartialEq(bound = ""),
    Debug(bound = ""),
    Eq(bound = ""),
    Default(bound = ""),
    Hash(bound = "")
)]
#[scale_info(skip_type_params(T))]
#[doc(hidden)]
pub struct PhantomDataSendSync<T>(core::marker::PhantomData<T>);

impl<T> PhantomDataSendSync<T> {
    pub(crate) fn new() -> Self {
        Self(core::marker::PhantomData)
    }
}

unsafe impl<T> Send for PhantomDataSendSync<T> {}
unsafe impl<T> Sync for PhantomDataSendSync<T> {}

/// This represents a key-value collection and is SCALE compatible
/// with collections like BTreeMap. This has the same type params
/// as `BTreeMap` which allows us to easily swap the two during codegen.
pub type KeyedVec<K, V> = Vec<(K, V)>;
