use std::{
    fs::File,
    io::{Cursor, Read},
};

use byteorder::{ByteOrder, LittleEndian, ReadBytesExt};

pub(crate) const FILESIZE_FIELD_SIZE: usize = 4;
pub(crate) const IDX1_INDEX_ENTRY_SIZE: usize = 16;
pub(crate) const IDX1_FOURCC: &[u8; 4] = b"idx1";

// AVIファイルのチャンクヘッダーの構造体
#[derive(Debug)]
pub(crate) struct ChunkHeader {
    fourcc: [u8; 4],
    size: u32,
}

impl ChunkHeader {
    fn new(fourcc: [u8; 4], size: u32) -> Self {
        ChunkHeader { fourcc, size }
    }

    pub(crate) fn get_fourcc(&self) -> [u8; 4] {
        self.fourcc
    }

    pub(crate) fn get_size(&self) -> usize {
        self.size as usize
    }
}

#[derive(Debug)]
pub(crate) struct AVIIndex {
    chunk_id: [u8; 4],
    flags: u32,
    offset: u32,
    size: u32,
}

impl AVIIndex {
    pub(crate) fn get_chunk_id(&self) -> [u8; 4] {
        self.chunk_id
    }

    pub(crate) fn get_flags(&self) -> u32 {
        self.flags
    }

    pub(crate) fn get_offset(&self) -> u64 {
        self.offset as u64
    }

    pub(crate) fn get_size(&self) -> usize {
        self.size as usize
    }
}

pub(crate) trait AVI {
    fn open(&self, filename: &str) -> Result<Cursor<Vec<u8>>, String> {
        let mut file = File::open(&filename).map_err(|e| format!("Failed to open file: {}", e))?;
        let mut buffer = Vec::new();
        let _ = file.read_to_end(&mut buffer);
        let file_reader = std::io::Cursor::new(buffer);
        Ok(file_reader)
    }

    fn read_index_entry<R: Read>(&self, reader: &mut R) -> Option<AVIIndex> {
        let mut chunk_id = [0u8; 4];
        if reader.read_exact(&mut chunk_id).is_err() {
            return None;
        }
        let flags = reader.read_u32::<LittleEndian>().unwrap_or(0);
        let offset = reader.read_u32::<LittleEndian>().unwrap_or(0);
        let size = reader.read_u32::<LittleEndian>().unwrap_or(0);
        Some(AVIIndex {
            chunk_id,
            flags,
            offset,
            size,
        })
    }

    fn parse_chunk_header(&self, data: &[u8]) -> Option<ChunkHeader> {
        if data.len() < 8 {
            return None;
        }
        let fourcc = [data[0], data[1], data[2], data[3]];
        let size = LittleEndian::read_u32(&data[4..8]);
        Some(ChunkHeader { fourcc, size })
    }
}
