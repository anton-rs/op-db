use alloy_rlp::Decodable;
use anyhow::{anyhow, Result};
use leveldb::{database::Database, kv::KV, options::ReadOptions};
use reth_primitives::{Header, SealedBlock, SealedHeader, TransactionSigned};

mod key;
pub use self::key::DBKey;

/// A wrapper around a [leveldb] [Database] instance to read data from a Geth database into
/// [reth_primitives] types.
pub struct GethDBReader {
    db: Database<DBKey>,
}

impl GethDBReader {
    /// Create a new [GethDBReader] instance.
    ///
    /// ### Takes
    /// - `database`: An open handle to a Geth [leveldb] database.
    pub fn new(database: Database<DBKey>) -> Self {
        Self { db: database }
    }

    /// Retrieve a [SealedHeader] by its block number from a Geth LevelDB
    ///
    /// ### Takes
    /// - `db`: A reference to a [Database] instance
    /// - `number`: The block number of the [SealedHeader] to retrieve
    ///
    /// ### Returns
    /// - Success: A [SealedHeader] instance
    /// - Failure: An [anyhow::Error] if the header could not be found
    pub fn header_by_number(&self, number: u64) -> Result<SealedHeader> {
        // Fetch the header hash
        let header_hash_key = DBKey::hash_by_number(number);
        let header_hash: [u8; 32] = self
            .db
            .get(ReadOptions::new(), header_hash_key)?
            .ok_or(anyhow::anyhow!("Header hash not found"))?
            .try_into()
            .map_err(|_| anyhow!("Header hash received from DB is not 32 bytes in size"))?;

        // Fetch the header RLP
        let header_key = DBKey::header_lookup(header_hash, number);
        let header_rlp = self
            .db
            .get(ReadOptions::new(), header_key)?
            .ok_or(anyhow::anyhow!("Header RLP not found"))?;

        // Decode the header
        let unsealed_header = Header::decode(&mut header_rlp.as_slice())
            .map_err(|e| anyhow!("RLP decode error: {e}"))?;

        // Return the sealed header
        Ok(unsealed_header.seal(header_hash.into()))
    }

    /// Retrieve a [SealedBlock] by its block number from a Geth LevelDB.
    ///
    /// ### Takes
    /// - `db`: A reference to a [Database] instance
    /// - `number`: The block number of the [SealedBlock] to Retrieve
    ///
    /// ### Returns
    /// - Success: A [SealedBlock] instance
    /// - Failure: An [anyhow::Error] if the block could not be found
    pub fn block_by_number(&self, number: u64) -> Result<SealedBlock> {
        let header = self.header_by_number(number)?;

        let body_key = DBKey::body_by_hash(*header.hash, header.number);
        let body_rlp = self
            .db
            .get(ReadOptions::new(), body_key)?
            .ok_or(anyhow::anyhow!("Body RLP not found"))?;

        // Decode the transactions in the block body
        // Why geth? ... Why?
        let mut transactions = <Vec<Vec<TransactionSigned>>>::decode(&mut body_rlp.as_slice())?;

        Ok(SealedBlock {
            header,
            body: transactions.remove(0),
            ommers: Vec::new(),
            withdrawals: None,
        })
    }
}

#[cfg(test)]
mod db_test {
    use super::key::DBKey;
    use crate::leveldb::GethDBReader;
    use leveldb::{database::Database, options::Options};
    use std::path::PathBuf;

    const TEST_BLOCK_NO: u64 = 4_000_000;

    #[test]
    #[ignore]
    fn sanity_read_header() {
        let mut db_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        db_path.push("testdata/geth/chaindata");

        let options = Options::new();
        let database: Database<DBKey> = Database::open(db_path.as_path(), options).unwrap();
        let reader = GethDBReader::new(database);

        dbg!(reader.header_by_number(TEST_BLOCK_NO).unwrap());
    }

    #[test]
    #[ignore]
    fn sanity_read_block() {
        let mut db_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        db_path.push("testdata/geth/chaindata");

        let options = Options::new();
        let database: Database<DBKey> = Database::open(db_path.as_path(), options).unwrap();
        let reader = GethDBReader::new(database);

        dbg!(reader.block_by_number(TEST_BLOCK_NO).unwrap());
    }
}
