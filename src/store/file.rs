use std::path::Path;

use bytesize::ByteSize;

use crate::map::{BlockMap, Flags};

use super::*;

/// MapStore implements a store on a mmap file.

pub struct FileStore {
    map: BlockMap,
    size: ByteSize,
}

impl FileStore {
    pub fn new<P: AsRef<Path>>(path: P, size: ByteSize, bs: ByteSize) -> Result<Self> {
        Ok(Self {
            map: BlockMap::new(path, size, bs).map_err(IoError::from)?,
            size,
        })
    }
}

#[async_trait::async_trait]
impl Store for FileStore {
    async fn set(&mut self, index: u32, data: &[u8]) -> Result<()> {
        if data.len() != self.map.block_size() {
            return Err(Error::InvalidBlockSize);
        }

        let mut block = self.map.at_mut(index as usize);
        block.data_mut().copy_from_slice(data);
        block.header_mut().set(Flags::Occupied, true);

        // this flushes the block immediately, may
        // be for performance improvements we shouldn't
        // do that or use async way
        self.map.flush_block(index as usize)
    }

    async fn get(&self, index: u32) -> Result<Option<Data>> {
        // we access the map directly to avoid a borrow problem
        let header = self.map.header_at(index as usize);
        if !header.flag(Flags::Occupied) {
            return Ok(None);
        }

        let data = self.map.data_at(index as usize);

        Ok(Some(Data::Borrowed(data)))
    }

    fn size(&self) -> ByteSize {
        self.size
    }

    fn block_size(&self) -> usize {
        self.map.block_size()
    }
}