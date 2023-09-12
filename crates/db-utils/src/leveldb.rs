use std::path::PathBuf;

use leveldb::{database::Database, options::Options};

struct LevelDB {
    path: PathBuf,
    database: Database,
}

impl LevelDB {
    pub fn new(path: PathBuf) -> Self {
        let options = Options::new();
        Self {
            path,
            database: Database::open(path.as_path(), options).unwrap(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn print_type_of<T>(_: &T) {
        println!("{}", std::any::type_name::<T>())
    }

    #[test]
    fn test_leveldb() {
        let db = LevelDB::new(PathBuf::from("testdb"));
        let ddb = db.database;
        dbg!(print_type_of(&db));
    }
}
