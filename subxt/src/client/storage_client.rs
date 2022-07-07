// Copyright 2019-2022 Parity Technologies (UK) Ltd.
// This file is dual-licensed as Apache-2.0 or GPL-3.0.
// see LICENSE for license details.

use codec::{
    Decode,
    Encode,
};
use sp_core::storage::{
    StorageChangeSet,
    StorageData,
    StorageKey,
};
pub use sp_runtime::traits::SignedExtension;
use std::{
    marker::PhantomData,
};
use crate::{
    error::BasicError,
    metadata::{
        MetadataError,
    },
    client::{
        OnlineClientT,
    },
    Config,
    StorageHasher,
};

/// Query the runtime storage using [StorageClient].
///
/// This module is the core of performing runtime storage queries. While you can
/// work with it directly, it's prefer to use the generated `storage()` interface where
/// possible.
///
/// The exposed API is performing RPC calls to `state_getStorage` and `state_getKeysPaged`.
///
/// A runtime storage entry can be of type:
/// - [StorageEntryKey::Plain] for keys constructed just from the prefix
///   `twox_128(pallet) ++ twox_128(storage_item)`
/// - [StorageEntryKey::Map] for mapped keys constructed from the prefix,
///   plus other arguments `twox_128(pallet) ++ twox_128(storage_item) ++ hash(arg1) ++ arg1`
///
/// # Examples
///
/// ## Fetch Storage Keys
///
/// ```no_run
/// # use subxt::{ClientBuilder, DefaultConfig, PolkadotExtrinsicParams};
/// # use subxt::storage::StorageClient;
///
/// #[subxt::subxt(runtime_metadata_path = "../artifacts/polkadot_metadata.scale")]
/// pub mod polkadot {}
///
/// # #[tokio::main]
/// # async fn main() {
/// # let api = ClientBuilder::new()
/// #     .build()
/// #     .await
/// #     .unwrap()
/// #     .to_runtime_api::<polkadot::RuntimeApi<DefaultConfig, PolkadotExtrinsicParams<DefaultConfig>>>();
/// # // Obtain the storage client wrapper from the API.
/// # let storage: StorageClient<_> = api.client.storage();
/// // Fetch just the keys, returning up to 10 keys.
/// let keys = storage
///     .fetch_keys::<polkadot::xcm_pallet::storage::VersionNotifiers>(10, None, None)
///     .await
///     .unwrap();
/// // Iterate over each key
/// for key in keys.iter() {
///     println!("Key: 0x{}", hex::encode(&key));
/// }
/// # }
/// ```
///
/// ## Iterate over Storage
///
/// ```no_run
/// # use subxt::{ClientBuilder, DefaultConfig, PolkadotExtrinsicParams};
/// # use subxt::storage::StorageClient;
///
/// #[subxt::subxt(runtime_metadata_path = "../artifacts/polkadot_metadata.scale")]
/// pub mod polkadot {}
///
/// # #[tokio::main]
/// # async fn main() {
/// # let api = ClientBuilder::new()
/// #     .build()
/// #     .await
/// #     .unwrap()
/// #     .to_runtime_api::<polkadot::RuntimeApi<DefaultConfig, PolkadotExtrinsicParams<DefaultConfig>>>();
/// # // Obtain the storage client wrapper from the API.
/// # let storage: StorageClient<_> = api.client.storage();
/// // Iterate over keys and values.
/// let mut iter = storage
///     .iter::<polkadot::xcm_pallet::storage::VersionNotifiers>(None)
///     .await
///     .unwrap();
/// while let Some((key, value)) = iter.next().await.unwrap() {
///     println!("Key: 0x{}", hex::encode(&key));
///     println!("Value: {}", value);
/// }
/// # }
/// ```
pub struct StorageClient<T, Client> {
    client: Client,
    _marker: PhantomData<T>
}

impl<T, Client: Clone> Clone for StorageClient<T, Client> {
    fn clone(&self) -> Self {
        Self {
            client: self.client.clone(),
            _marker: PhantomData,
        }
    }
}

impl<T, Client> StorageClient<T, Client>{
    /// Create a new [`StorageClient`]
    pub fn new(
        client: Client,
    ) -> Self {
        Self {
            client,
            _marker: PhantomData,
        }
    }
}

impl<T, Client> StorageClient<T, Client>
where
    T: Config,
    Client: OnlineClientT<T>,
{
    /// Fetch the raw encoded value under the raw storage key given.
    pub async fn fetch_raw_key(
        &self,
        key: &StorageKey,
        hash: Option<T::Hash>,
    ) -> Result<Option<Vec<u8>>, BasicError> {
        let data = self.client.rpc().storage(&key, hash).await?;
        Ok(data.map(|d| d.0))
    }

    /// Fetch the raw encoded value at the address given.
    pub async fn fetch_raw<K: Into<StorageKey>>(
        &self,
        key: K,
        hash: Option<T::Hash>,
    ) -> Result<Option<Vec<u8>>, BasicError> {
        let key = key.into();
        self.fetch_raw_key(&key, hash).await
    }

    /// Fetch a decoded value from storage at a given address and optional block hash.
    pub async fn fetch<ReturnTy: Decode>(
        &self,
        address: StorageAddress<'_, ReturnTy>,
        hash: Option<T::Hash>,
    ) -> Result<Option<ReturnTy>, BasicError> {
        if let Some(data) = self.fetch_raw(&address, hash).await? {
            Ok(Some(Decode::decode(&mut &*data)?))
        } else {
            Ok(None)
        }
    }

    /// Fetch a StorageKey that has a default value with an optional block hash.
    pub async fn fetch_or_default<ReturnTy: Decode>(
        &self,
        address: StorageAddress<'_, ReturnTy>,
        hash: Option<T::Hash>,
    ) -> Result<ReturnTy, BasicError> {
        let pallet_name = address.pallet_name;
        let storage_name = address.storage_name;
        if let Some(data) = self.fetch(address, hash).await? {
            Ok(data)
        } else {
            let metadata = self.client.metadata();
            let pallet_metadata = metadata.pallet(pallet_name)?;
            let storage_metadata = pallet_metadata.storage(storage_name)?;
            let default = Decode::decode(&mut &storage_metadata.default[..])
                .map_err(MetadataError::DefaultError)?;
            Ok(default)
        }
    }

    /// Fetch up to `count` keys for a storage map in lexicographic order.
    ///
    /// Supports pagination by passing a value to `start_key`.
    pub async fn fetch_keys<K: Into<StorageKey>>(
        &self,
        key: StorageKey,
        count: u32,
        start_key: Option<StorageKey>,
        hash: Option<T::Hash>,
    ) -> Result<Vec<StorageKey>, BasicError> {
        let keys = self
            .client
            .rpc()
            .storage_keys_paged(key.into(), count, start_key, hash)
            .await?;
        Ok(keys)
    }

    /// Returns an iterator of key value pairs.
    pub async fn iter<F: StorageEntry>(
        &self,
        page_size: u32,
        hash: Option<T::Hash>,
    ) -> Result<KeyIter<T, Client, F>, BasicError> {
        let hash = if let Some(hash) = hash {
            hash
        } else {
            self.client
                .rpc()
                .block_hash(None)
                .await?
                .expect("didn't pass a block number; qed")
        };
        Ok(KeyIter {
            client: self.clone(),
            hash,
            count: page_size,
            start_key: None,
            buffer: Default::default(),
            _marker: PhantomData,
        })
    }
}


/// WHAT DO WE ACTUALLY NEED?
///
/// - Iterate entries
/// - Access specific entry
///   - Return raw bytes, Value, or decoded type.
///
/// Accessing a single entry:
/// - PALLET NAME
/// - STORAGE ENTRY NAME
/// - for a "plain" key, that's it. For a "map" key, we need
///   a list of 1 or more values + hashers, which we'll append to the
///   hashed key to access the specific item.
///
/// Iterating entries:
/// - how many entries per page to obtain?
/// - PALLET NAME
/// - STORAGE ENTRY name
/// - Key to start from (allows pagination)


















/// This is returned from storage accesses in the statically generated
/// code, and contains the information needed to find, validate and decode
/// the storage entry.
pub struct StorageAddress <'a, ReturnTy> {
    pallet_name: &'a str,
    storage_name: &'a str,
    // How to access the specific value at that storage address.
    storage_entry_key: StorageEntryKey,
    // Hash provided from static code for validation.
    storage_hash: Option<[u8; 32]>,
    _marker: std::marker::PhantomData<ReturnTy>
}

impl <'a, ReturnTy> StorageAddress<'a, ReturnTy> {
    /// Create a new [`StorageAddress`] that will be validated
    /// against node metadata using the hash given.
    pub fn new_with_validation(
        pallet_name: &'a str,
        storage_name: &'a str,
        storage_entry_key: StorageEntryKey,
        hash: [u8; 32]
    ) -> Self {
        Self {
            pallet_name,
            storage_name,
            storage_entry_key,
            storage_hash: Some(hash),
            _marker: std::marker::PhantomData
        }
    }

    /// Do not validate this storage prior to accessing it.
    pub fn unvalidated(self) -> Self {
        Self {
            pallet_name: self.pallet_name,
            storage_name: self.storage_name,
            storage_entry_key: self.storage_entry_key,
            storage_hash: None,
            _marker: self._marker
        }
    }

    // Convert this address into bytes that we can pass to a node to look up
    // the associated value at this address.
    pub fn to_bytes(&self) -> Vec<u8> {
        // First encode the pallet/name part:
        let mut bytes = sp_core::twox_128(self.pallet_name.as_bytes()).to_vec();
        bytes.extend(&sp_core::twox_128(self.storage_name.as_bytes())[..]);

        // Then, if we need to further dig into this entry, we do so,
        // hashing additional fields as specified:
        if let StorageEntryKey::Map(map) = &self.storage_entry_key {
            for entry in map {
                entry.to_bytes(&mut bytes);
            }
        }

        bytes
    }
}

impl <'a, R> From<&StorageAddress<'a, R>> for StorageKey {
    fn from(address: &StorageAddress<'a, R>) -> Self {
        StorageKey(address.to_bytes())
    }
}









/// Storage entry trait.
pub trait StorageEntry {
    /// Pallet name.
    const PALLET: &'static str;
    /// Storage name.
    const STORAGE: &'static str;
    /// Type of the storage entry value.
    type Value: Decode;
    /// Get the key data for the storage.
    fn key(&self) -> StorageEntryKey;
}

/// The prefix of the key to a [`StorageEntry`]
pub struct StorageKeyPrefix(Vec<u8>);

impl StorageKeyPrefix {
    /// Create the storage key prefix for a [`StorageEntry`]
    pub fn new<T: StorageEntry>() -> Self {
        let mut bytes = sp_core::twox_128(T::PALLET.as_bytes()).to_vec();
        bytes.extend(&sp_core::twox_128(T::STORAGE.as_bytes())[..]);
        Self(bytes)
    }

    /// Convert the prefix into a [`StorageKey`]
    pub fn to_storage_key(self) -> StorageKey {
        StorageKey(self.0)
    }
}

/// Storage key.
pub enum StorageEntryKey {
    /// Plain key.
    Plain,
    /// Map key(s).
    Map(Vec<StorageMapKey>),
}

impl StorageEntryKey {
    /// Construct the final [`sp_core::storage::StorageKey`] for the storage entry.
    pub fn final_key(&self, prefix: StorageKeyPrefix) -> sp_core::storage::StorageKey {
        let mut bytes = prefix.0;
        if let Self::Map(map_keys) = self {
            for map_key in map_keys {
                bytes.extend(Self::hash(&map_key.hasher, &map_key.value))
            }
        }
        sp_core::storage::StorageKey(bytes)
    }

    fn hash(hasher: &StorageHasher, bytes: &[u8]) -> Vec<u8> {
        match hasher {
            StorageHasher::Identity => bytes.to_vec(),
            StorageHasher::Blake2_128 => sp_core::blake2_128(bytes).to_vec(),
            StorageHasher::Blake2_128Concat => {
                // copied from substrate Blake2_128Concat::hash since StorageHasher is not public
                sp_core::blake2_128(bytes)
                    .iter()
                    .chain(bytes)
                    .cloned()
                    .collect()
            }
            StorageHasher::Blake2_256 => sp_core::blake2_256(bytes).to_vec(),
            StorageHasher::Twox128 => sp_core::twox_128(bytes).to_vec(),
            StorageHasher::Twox256 => sp_core::twox_256(bytes).to_vec(),
            StorageHasher::Twox64Concat => {
                sp_core::twox_64(bytes)
                    .iter()
                    .chain(bytes)
                    .cloned()
                    .collect()
            }
        }
    }
}

/// Storage key for a Map.
pub struct StorageMapKey {
    value: Vec<u8>,
    hasher: StorageHasher,
}

impl StorageMapKey {
    /// Create a new [`StorageMapKey`] with the encoded data and the hasher.
    pub fn new<T: Encode>(value: &T, hasher: StorageHasher) -> Self {
        Self {
            value: value.encode(),
            hasher,
        }
    }

    /// Convert this [`StorageMapKey`] into bytes and append it to some existing bytes.
    pub fn to_bytes(&self, bytes: &mut Vec<u8>) {
        match &self.hasher {
            StorageHasher::Identity => bytes.extend(&self.value),
            StorageHasher::Blake2_128 => bytes.extend(sp_core::blake2_128(bytes)),
            StorageHasher::Blake2_128Concat => {
                // adapted from substrate Blake2_128Concat::hash since StorageHasher is not public
                let v = sp_core::blake2_128(&self.value)
                    .iter()
                    .chain(&self.value)
                    .cloned();
                bytes.extend(v);
            }
            StorageHasher::Blake2_256 => bytes.extend(sp_core::blake2_256(&self.value)),
            StorageHasher::Twox128 => bytes.extend(sp_core::twox_128(&self.value)),
            StorageHasher::Twox256 => bytes.extend(sp_core::twox_256(&self.value)),
            StorageHasher::Twox64Concat => {
                let v = sp_core::twox_64(&self.value)
                    .iter()
                    .chain(&self.value)
                    .cloned();
                bytes.extend(v);
            }
        }
    }
}

/// Iterates over key value pairs in a map.
pub struct KeyIter<T: Config, Client, F: StorageEntry> {
    client: StorageClient<T, Client>,
    _marker: PhantomData<F>,
    count: u32,
    hash: T::Hash,
    start_key: Option<StorageKey>,
    buffer: Vec<(StorageKey, StorageData)>,
}

impl<'a, T: Config, Client: OnlineClientT<T>, F: StorageEntry> KeyIter<T, Client, F> {
    /// Returns the next key value pair from a map.
    pub async fn next(&mut self) -> Result<Option<(StorageKey, F::Value)>, BasicError> {
        loop {
            if let Some((k, v)) = self.buffer.pop() {
                return Ok(Some((k, Decode::decode(&mut &v.0[..])?)))
            } else {
                let keys = self
                    .client
                    .fetch_keys::<F>(self.count, self.start_key.take(), Some(self.hash))
                    .await?;

                if keys.is_empty() {
                    return Ok(None)
                }

                self.start_key = keys.last().cloned();

                let change_sets = self
                    .client
                    .client
                    .rpc()
                    .query_storage_at(&keys, Some(self.hash))
                    .await?;
                for change_set in change_sets {
                    for (k, v) in change_set.changes {
                        if let Some(v) = v {
                            self.buffer.push((k, v));
                        }
                    }
                }
                debug_assert_eq!(self.buffer.len(), keys.len());
            }
        }
    }
}
