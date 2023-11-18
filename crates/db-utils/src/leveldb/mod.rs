mod key;

#[cfg(test)]
mod db_test {
    use super::key::DBKey;
    use alloy_primitives::hex;
    use alloy_rlp::Decodable;
    use leveldb::{
        database::Database,
        kv::KV,
        options::{Options, ReadOptions},
    };
    use reth_primitives::Header;
    use std::path::PathBuf;

    const HEADER_PREFIX: u8 = b"h"[0];
    const NUM_SUFFIX: u8 = b"n"[0];

    #[test]
    #[ignore]
    fn sanity_read_header() {
        let mut db_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        db_path.push("testdata/geth/chaindata");

        let options = Options::new();
        let database: Database<DBKey> = Database::open(db_path.as_path(), options).unwrap();

        let block_number = 4_000_000u64;

        // Formulate the block hash key
        // Table 1: header prefix ++ number ++ num suffix
        let mut block_hash_key = vec![HEADER_PREFIX];
        block_hash_key.extend_from_slice(&block_number.to_be_bytes());
        block_hash_key.push(NUM_SUFFIX);

        // Get blockhash first
        let read_opts = ReadOptions::new();
        let block_hash = database
            .get(read_opts, &block_hash_key.into())
            .unwrap()
            .unwrap();

        println!(
            "Found block hash for block #{block_number}: 0x{}",
            hex::encode(&block_hash)
        );

        // Formulate the header key
        // Table 2: header prefix ++ number ++ hash
        let mut header_key = vec![HEADER_PREFIX];
        header_key.extend_from_slice(&block_number.to_be_bytes());
        header_key.extend(block_hash);

        let read_opts = ReadOptions::new();
        let header = database
            .get(read_opts, &header_key.into())
            .unwrap()
            .unwrap();

        // RLP Decode the header
        let header = Header::decode(&mut header.as_slice()).unwrap();
        dbg!(header);
    }
}
