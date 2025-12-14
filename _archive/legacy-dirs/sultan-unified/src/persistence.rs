use rocksdb::{DB, Options};
use anyhow::Result;

pub struct Storage {
    db: DB,
}

impl Storage {
    pub fn new(path: &str) -> Result<Self> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        let db = DB::open(&opts, path)?;
        Ok(Storage { db })
    }
    
    pub fn save_block(&self, height: u64, block_data: &[u8]) -> Result<()> {
        self.db.put(format!("block:{}", height), block_data)?;
        Ok(())
    }
    
    pub fn get_block(&self, height: u64) -> Result<Option<Vec<u8>>> {
        Ok(self.db.get(format!("block:{}", height))?)
    }
    
    pub fn save_state(&self, key: &str, value: &[u8]) -> Result<()> {
        self.db.put(format!("state:{}", key), value)?;
        Ok(())
    }
}
