use alloy_rlp::Decodable;
use anyhow::{anyhow, Result};
use leveldb::{database::Database, kv::KV, options::ReadOptions};
use reth_primitives::{
    Header, Log, Receipt, ReceiptWithBloom, SealedBlock, SealedHeader, TransactionSigned, TxType,
};

mod key;
pub use self::key::DBKey;

/// A wrapper around a [leveldb] [Database] instance to read data from a legacy l2geth database into
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

    /// Retrieve a header hash by its number from a Geth LevelDB.
    ///
    /// ### Takes
    /// - `number`: The block number of the header hash to retrieve
    ///
    /// ### Returns
    /// - Success: A [u8; 32] containing the header hash
    /// - Failure: An [anyhow::Error] if the header hash could not be found
    pub fn hash_by_number(&self, number: u64) -> Result<[u8; 32]> {
        let key = DBKey::hash_by_number(number);
        let hash = self
            .db
            .get(ReadOptions::new(), key)?
            .ok_or(anyhow!("Header hash not found"))?
            .try_into()
            .map_err(|_| anyhow!("Header hash received from DB is not 32 bytes in size"))?;

        Ok(hash)
    }

    /// Retrieve a [SealedHeader] by its block number from a Geth LevelDB
    ///
    /// ### Takes
    /// - `number`: The block number of the [SealedHeader] to retrieve
    ///
    /// ### Returns
    /// - Success: A [SealedHeader] instance
    /// - Failure: An [anyhow::Error] if the header could not be found
    pub fn header_by_number(&self, number: u64) -> Result<SealedHeader> {
        // Fetch the header hash
        let header_hash = self.hash_by_number(number)?;

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

    /// Reads the receipts for a [SealedBlock] by its number from a Geth LevelDB.
    ///
    /// Geth encodes its receipts in storage without the Bloom Filter, and [reth_primitives]
    /// [Receipt] type does not implement [Decodable]. This function decodes the single
    /// expected [Receipt] and computes the Bloom Filter after decoding, as per the Geth
    /// "ReceiptForStorage" type's behavior.
    ///
    /// ### Takes
    /// - `db`: A reference to a [Database] instance
    ///
    /// ### Returns
    /// - Success: A [Vec] of [ReceiptWithBloom] instances
    /// - Failure: An [anyhow::Error] if the receipts could not be found
    pub fn receipts_by_number(&self, number: u64) -> Result<Vec<ReceiptWithBloom>> {
        let header_hash = self.hash_by_number(number)?;

        let receipts_key = DBKey::receipts_by_hash(header_hash, number);
        let receipts_rlp = self
            .db
            .get(ReadOptions::new(), receipts_key)?
            .ok_or(anyhow::anyhow!("Receipts RLP not found"))?;
        let rlp_buf = &mut receipts_rlp.as_slice();

        // Outer list - consume the header, we don't care about it. The only block that has a
        // zero-length outer list is the Bedrock genesis block.
        let outer_header = alloy_rlp::Header::decode(rlp_buf)?;
        if outer_header.payload_length == 0 {
            return Ok(Vec::default());
        }

        // Inner list
        let rlp_header = alloy_rlp::Header::decode(rlp_buf)?;
        let payload_len = rlp_header.payload_length;
        let start = rlp_buf.len();

        // Decode inner receipt, note that there is no bloom filter in the RLP.
        let success: bool = alloy_rlp::Decodable::decode(rlp_buf)?;
        let cumulative_gas_used: u64 = alloy_rlp::Decodable::decode(rlp_buf)?;
        let logs: Vec<Log> = alloy_rlp::Decodable::decode(rlp_buf)?;

        if start - payload_len != 0 {
            anyhow::bail!(
                "Receipts RLP not fully consumed; Expected 1 receipt per Legacy Optimism block!"
            );
        }

        // All receipts in the legacy Optimism datadir are legacy transactions
        let receipt = Receipt {
            tx_type: TxType::Legacy,
            success,
            cumulative_gas_used,
            logs,
        };

        Ok(vec![receipt.with_bloom()])
    }
}

#[cfg(test)]
mod db_test {
    use super::key::DBKey;
    use crate::leveldb::GethDBReader;
    use leveldb::{database::Database, options::Options};
    use std::path::PathBuf;

    const BEDROCK_TRANSITION: u64 = 4_061_224;
    const FULL_PRUNE_DEPTH: u64 = 90_000;

    #[test]
    #[ignore]
    fn sanity_read_headers() {
        let mut db_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        db_path.push("testdata/bedrock/geth/chaindata");

        let options = Options::new();
        let database: Database<DBKey> = Database::open(db_path.as_path(), options).unwrap();
        let reader = GethDBReader::new(database);

        for i in BEDROCK_TRANSITION - FULL_PRUNE_DEPTH - 1..=BEDROCK_TRANSITION {
            if let Err(e) = reader.header_by_number(i) {
                panic!("Error reading header @ block # {}: {}", i, e);
            }
        }
    }

    #[test]
    #[ignore]
    fn sanity_read_block() {
        let mut db_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        db_path.push("testdata/bedrock/geth/chaindata");

        let options = Options::new();
        let database: Database<DBKey> = Database::open(db_path.as_path(), options).unwrap();
        let reader = GethDBReader::new(database);

        for i in BEDROCK_TRANSITION - FULL_PRUNE_DEPTH - 1..=BEDROCK_TRANSITION {
            if let Err(e) = reader.block_by_number(i) {
                panic!("Error reading block # {}: {}", i, e);
            }
        }
    }

    #[test]
    #[ignore]
    fn sanity_read_receipts() {
        let mut db_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        db_path.push("testdata/bedrock/geth/chaindata");

        let options = Options::new();
        let database: Database<DBKey> = Database::open(db_path.as_path(), options).unwrap();
        let reader = GethDBReader::new(database);

        for i in BEDROCK_TRANSITION - FULL_PRUNE_DEPTH - 1..=BEDROCK_TRANSITION {
            if let Err(e) = reader.receipts_by_number(i) {
                panic!("Error reading receipts in block # {}: {}", i, e);
            }
        }
    }
}
