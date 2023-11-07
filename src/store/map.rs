use std::path::Path;

use bytesize::ByteSize;

use crate::map::{BlockMap, Flags};

use super::*;

/// MapStore implements a store on a mmap file.

pub struct MapStore {
    map: BlockMap,
    size: usize,
}

impl MapStore {
    pub fn new<P: AsRef<Path>>(path: P, size: ByteSize, bs: ByteSize) -> Result<Self> {
        Ok(Self {
            map: BlockMap::new(path, size, bs).map_err(IoError::from)?,
            size: size.as_u64() as usize,
        })
    }
}

#[async_trait::async_trait]
impl Store for MapStore {
    async fn set(&mut self, index: u32, data: &[u8]) -> Result<()> {
        if data.len() != self.map.block_size() {
            return Err(Error::InvalidBlockSize);
        }

        let mut block = self.map.at_mut(index as usize);
        block.data_mut().copy_from_slice(data);
        block.set_header(block.header().with_flag(Flags::Occupied, true));
        block.flush().map_err(IoError::from)?;

        Ok(())
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

    fn size(&self) -> usize {
        self.size
    }
}