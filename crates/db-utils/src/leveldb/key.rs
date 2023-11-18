//! Helpers for dealing with Geth's rawdb schema and constructing keys.
//!
//! See: https://github.com/ethereum/go-ethereum/blob/c8a22020287e0260e2310a1b91a1aa9b795ca445/core/rawdb/schema.go

use leveldb::database::key::Key;

pub const HEADER_PREFIX: u8 = b"h"[0];
pub const HEADER_HASH_SUFFIX: u8 = b"n"[0];
pub const BLOCK_BODY_PREFIX: u8 = b"b"[0];

/// Wrapper around a [Vec<u8>] to implement the [Key] trait.
pub struct DBKey(Vec<u8>);

impl DBKey {
    /// Get a key for the `hash by number` table in the Geth leveldb.
    ///
    /// Format: `header_hash_prefix ++ number ++ header_hash_suffix -> header_hash`
    pub fn hash_by_number(number: u64) -> Self {
        const KEY_SIZE: usize = 1 + 8 + 1;

        let mut key = Vec::with_capacity(KEY_SIZE);
        key.push(HEADER_PREFIX);
        key.extend_from_slice(&number.to_be_bytes());
        key.push(HEADER_HASH_SUFFIX);
        Self(key)
    }

    /// Get a key for the `header by number + hash` table in the Geth leveldb.
    ///
    /// Format: `header_prefix ++ number ++ hash -> header_rlp`
    pub fn header_lookup(hash: [u8; 32], number: u64) -> Self {
        const KEY_SIZE: usize = 1 + 8 + 32;

        let mut key = Vec::with_capacity(KEY_SIZE);
        key.push(HEADER_PREFIX);
        key.extend_from_slice(&number.to_be_bytes());
        key.extend_from_slice(&hash);
        Self(key)
    }

    /// Get a key for the `block by number + hash` table in the Geth leveldb.
    ///
    /// Format: `block body prefix ++ number ++ hash -> block_body_rlp`
    pub fn body_by_hash(hash: [u8; 32], number: u64) -> Self {
        const KEY_SIZE: usize = 1 + 8 + 32;

        let mut key = Vec::with_capacity(KEY_SIZE);
        key.push(BLOCK_BODY_PREFIX);
        key.extend_from_slice(&number.to_be_bytes());
        key.extend_from_slice(&hash);
        Self(key)
    }
}

impl From<Vec<u8>> for DBKey {
    fn from(key: Vec<u8>) -> Self {
        Self(key)
    }
}

impl Key for DBKey {
    fn from_u8(key: &[u8]) -> Self {
        Self(key.to_vec())
    }

    fn as_slice<T, F: Fn(&[u8]) -> T>(&self, f: F) -> T {
        f(self.0.as_slice())
    }
}
