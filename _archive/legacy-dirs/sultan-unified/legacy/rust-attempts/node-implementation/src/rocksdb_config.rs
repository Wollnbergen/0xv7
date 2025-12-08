use rocksdb::{DB, Options, WriteBatch};
use std::path::Path;

pub struct SultanStateDB {
    db: DB,
    zero_fees: bool,  // Always true!
}

impl SultanStateDB {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, rocksdb::Error> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.set_max_write_buffer_number(16);
        opts.set_write_buffer_size(256 * 1024 * 1024); // 256MB
        opts.set_target_file_size_base(256 * 1024 * 1024);
        opts.set_max_bytes_for_level_base(1024 * 1024 * 1024); // 1GB
        opts.set_compression_type(rocksdb::DBCompressionType::Lz4);
        
        let db = DB::open(&opts, path)?;
        Ok(Self { 
            db,
            zero_fees: true  // Sultan Chain always has zero fees
        })
    }
    
    pub fn get_fee(&self) -> u64 {
        0  // Always returns 0 - zero gas fees!
    }
    
    pub fn write_batch(&self, batch: WriteBatch) -> Result<(), rocksdb::Error> {
        self.db.write(batch)
    }
}
