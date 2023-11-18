use leveldb::database::key::Key;

/// Wrapper around a [Vec<u8>] to implement the [Key] trait.
pub struct DBKey(Vec<u8>);

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
