// Copyright 2019-2022 Parity Technologies (UK) Ltd.
// This file is dual-licensed as Apache-2.0 or GPL-3.0.
// see LICENSE for license details.

use codec::Decode;
use futures::{
    future::Either,
    stream,
    Stream,
    StreamExt,
};
use sp_runtime::traits::Header;
use sp_core::{
    storage::StorageKey,
    twox_128,
};
use std::future::Future;
use crate::{
    client::{
        OnlineClientT
    },
    error::{
        BasicError,
    },
    Config,
};
use crate::events::{
    EventSubscription,
    Events,
    EventSub,
    FinalizedEventSub,
};
use derivative::Derivative;

/// A client for working with events.
#[derive(Derivative)]
#[derivative(Clone(bound = "Client: Clone"))]
pub struct EventsClient<T, Client> {
    client: Client,
    _marker: std::marker::PhantomData<T>
}

impl <T, Client> EventsClient<T, Client> {
    /// Create a new [`EventsClient`].
    pub fn new(client: Client) -> Self {
        Self {
            client,
            _marker: std::marker::PhantomData
        }
    }
}

impl <T, Client> EventsClient<T, Client>
where
    T: Config,
    Client: OnlineClientT<T>
{
    /// Obtain events at some block hash. The generic parameter is what we
    /// will attempt to decode each event into if using [`crate::events::Events::iter()`],
    /// and is expected to be the outermost event enum that contains all of
    /// the possible events across all pallets.
    pub fn at<Evs>(
        &self,
        block_hash: T::Hash,
    ) -> impl Future<Output = Result<Events<T, Evs>, BasicError>> + Send + 'static
    where
        Evs: Decode,
    {
        // Clone and pass the client in like this so that we can explicitly
        // return a Future that's Send + 'static, rather than tied to &self.
        let client = self.client.clone();
        async move {
            at(client, block_hash).await
        }
    }

    /// Subscribe to events from blocks.
    ///
    /// **Note:** these blocks haven't necessarily been finalised yet; prefer
    /// [`Events::subscribe_finalized()`] if that is important.
    pub fn subscribe<Evs>(
        &self
    ) -> impl Future<Output = Result<EventSubscription<T, Client, EventSub<T::Header>, Evs>, BasicError>> + Send + 'static
    where
        Evs: Decode + 'static
    {
        let client = self.client.clone();
        async move {
            subscribe(client).await
        }
    }

    /// Subscribe to events from finalized blocks.
    pub async fn subscribe_finalized<Evs>(
        &self,
    ) -> impl Future<Output = Result<EventSubscription<T, Client, FinalizedEventSub<T::Header>, Evs>, BasicError>> + Send + 'static
    where
        Client: Send + Sync + 'static,
        Evs: Decode + 'static
    {
        let client = self.client.clone();
        async move {
            subscribe_finalized(client).await
        }
    }

    /// Take a subscription that returns block headers, and if any block numbers are missed out
    /// betweem the block number provided and what's returned from the subscription, we fill in
    /// the gaps and get hold of all intermediate block headers.
    pub fn subscribe_to_block_headers_filling_in_gaps<S, E>(
        &self,
        last_block_num: Option<u64>,
        sub: S,
    ) -> impl Stream<Item = Result<T::Header, BasicError>> + Send
    where
        T: Config,
        Client: OnlineClientT<T>,
        S: Stream<Item = Result<T::Header, E>> + Send,
        E: Into<BasicError> + Send + 'static,
    {
        let client = self.client.clone();
        subscribe_to_block_headers_filling_in_gaps(client, last_block_num, sub)
    }
}

async fn at<T, Client, Evs>(
    client: Client,
    block_hash: T::Hash,
) -> Result<Events<T, Evs>, BasicError>
where
    T: Config,
    Client: OnlineClientT<T>,
    Evs: Decode,
{
    let event_bytes = client
        .rpc()
        .storage(&system_events_key(), Some(block_hash))
        .await?
        .map(|e| e.0)
        .unwrap_or_else(Vec::new);

    Ok(Events::new(
        client.metadata(),
        block_hash,
        event_bytes,
    ))
}

async fn subscribe<T, Client, Evs>(
    client: Client
) -> Result<EventSubscription<T, Client, EventSub<T::Header>, Evs>, BasicError>
where
    T: Config,
    Client: OnlineClientT<T>,
    Evs: Decode + 'static
{
    let block_subscription = client.rpc().subscribe_blocks().await?;
    Ok(EventSubscription::new(client, block_subscription))
}

/// Subscribe to events from finalized blocks.
async fn subscribe_finalized<T, Client, Evs>(
    client: Client,
) -> Result<EventSubscription<T, Client, FinalizedEventSub<T::Header>, Evs>, BasicError>
where
    T: Config,
    Client: OnlineClientT<T>,
    Evs: Decode + 'static
{
    // fetch the last finalised block details immediately, so that we'll get
    // events for each block after this one.
    let last_finalized_block_hash = client.rpc().finalized_head().await?;
    let last_finalized_block_number = client
        .rpc()
        .header(Some(last_finalized_block_hash))
        .await?
        .map(|h| (*h.number()).into());

    let sub = client.rpc().subscribe_finalized_blocks().await?;

    // Fill in any gaps between the block above and the finalized blocks reported.
    let block_subscription = subscribe_to_block_headers_filling_in_gaps(
        client.clone(),
        last_finalized_block_number,
        sub,
    );

    Ok(EventSubscription::new(client, Box::pin(block_subscription)))
}

fn subscribe_to_block_headers_filling_in_gaps<T, Client, S, E>(
    client: Client,
    mut last_block_num: Option<u64>,
    sub: S,
) -> impl Stream<Item = Result<T::Header, BasicError>> + Send
where
    T: Config,
    Client: OnlineClientT<T> + Send + Sync,
    S: Stream<Item = Result<T::Header, E>> + Send,
    E: Into<BasicError> + Send + 'static,
{
    sub.flat_map(move |s| {
        let client = client.clone();

        // Get the header, or return a stream containing just the error. Our EventSubscription
        // stream will return `None` as soon as it hits an error like this.
        let header = match s {
            Ok(header) => header,
            Err(e) => return Either::Left(stream::once(async { Err(e.into()) })),
        };

        // We want all previous details up to, but not including this current block num.
        let end_block_num = (*header.number()).into();

        // This is one after the last block we returned details for last time.
        let start_block_num = last_block_num.map(|n| n + 1).unwrap_or(end_block_num);

        // Iterate over all of the previous blocks we need headers for, ignoring the current block
        // (which we already have the header info for):
        let previous_headers = stream::iter(start_block_num..end_block_num)
            .then(move |n| {
                let client = client.clone();
                async move {
                    let hash = client.rpc().block_hash(Some(n.into())).await?;
                    let header = client.rpc().header(hash).await?;
                    Ok::<_, BasicError>(header)
                }
            })
            .filter_map(|h| async { h.transpose() });

        // On the next iteration, we'll get details starting just after this end block.
        last_block_num = Some(end_block_num);

        // Return a combination of any previous headers plus the new header.
        Either::Right(previous_headers.chain(stream::once(async { Ok(header) })))
    })
}

// The storage key needed to access events.
fn system_events_key() -> StorageKey {
    let mut storage_key = twox_128(b"System").to_vec();
    storage_key.extend(twox_128(b"Events").to_vec());
    StorageKey(storage_key)
}