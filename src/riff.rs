use crate::fourcc::FourCC;
use tokio::io::{AsyncRead, AsyncSeek, AsyncReadExt, SeekFrom, AsyncSeekExt};
use std::error::Error;
use crate::bytes::{BigEndian, LittleEndian};
use std::fmt::{Display, Formatter, Debug};
use std::io::{Read, Seek};
use futures::future::BoxFuture;
use std::sync::{RwLock, Arc};

const RIFF_TYPE: FourCC = FourCC::from_bytes(b"RIFF");
const LIST_TYPE: FourCC = FourCC::from_bytes(b"LIST");

#[derive(Debug)]
pub enum RiffError {
    InvalidRiffHeader,
    InvalidListHeader,
    InvalidChunkHeader,
    InvalidChunkCast,
    InvalidListCast
}

impl Display for RiffError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RiffError::InvalidRiffHeader => {
                write!(f, "Riff header invalid!")
            },
            RiffError::InvalidListHeader => {
                write!(f, "List header invalid!")
            }
            _ => { unreachable!() }
        }
    }
}

impl Error for RiffError {}

pub struct RiffUtil;

impl RiffUtil {

    pub fn read_fourcc<R>(reader: &mut R) -> Result<FourCC, Box<dyn Error>> where R: Read {
        let mut id_buf = [0u8; 4];
        reader.read(&mut id_buf)?;
        Ok(FourCC::from(BigEndian::read_u32(&id_buf, 0)))
    }

    pub async fn read_fourcc_async<R>(reader: &mut R) -> Result<FourCC, Box<dyn Error>> where R: AsyncRead + Unpin + Send + Sync {
        let mut id_buf = [0u8; 4];
        reader.read(&mut id_buf).await?;
        Ok(FourCC::from(BigEndian::read_u32(&id_buf, 0)))
    }
}

#[derive(Debug)]
pub struct RiffTree {
    header: RiffHeader,
    childs: Vec<Box<dyn RiffNode + Send>>
}

pub trait RiffNode: Debug {
    fn id(&self) -> FourCC;
    fn data_pos(&self) -> u64;
    fn data_size(&self) -> u32;
    fn padding(&self) -> u32;
    fn childs(&self) -> &Vec<Box<dyn RiffNode + Send>>;
    fn as_chunk(&self) -> Result<&RiffChunk, Box<dyn Error>>;
    fn as_list(&self) -> Result<&RiffList, Box<dyn Error>>;
}

#[derive(Debug)]
pub struct RiffHeader {
    file_size: u32,
    file_type: FourCC,
}

#[derive(Debug, Clone)]
pub struct RiffChunkHeader {
    ck_id: FourCC,
    ck_size: u32,
    data_pos: u64
}

impl RiffChunkHeader {

    pub fn data_size(&self) -> u32 {
        self.ck_size
    }

    pub fn data_pos(&self) -> u64 {
        self.data_pos
    }
}

#[derive(Debug, Clone)]
pub struct RiffListHeader {
    list_type: FourCC,
    list_size: u32,
    data_pos: u64,
}

impl RiffListHeader {

    pub fn data_size(&self) -> u32 {
        self.list_size - 4
    }

    pub fn data_pos(&self) -> u64 {
        self.data_pos
    }
}

#[derive(Debug)]
pub struct RiffChunk {
    header: RiffChunkHeader,
    childs: Vec<Box<dyn RiffNode + Send>>
}

#[derive(Debug)]
pub struct RiffList {
    header: RiffListHeader,
    childs: Vec<Box<dyn RiffNode + Send>>
}

impl RiffTree {

    /**
    * Reads and parses a riff file structure
    */
    pub async fn read_async<R>(reader: &mut R) -> Result<Self, Box<dyn Error>> where R: AsyncRead + AsyncSeek + Unpin + Send + Sync {

        let riff_file_len: u64 = reader.seek(SeekFrom::End(0)).await?;
        reader.seek(SeekFrom::Start(0)).await?;

        let mut riff_header_buf = [0u8;12];
        reader.read_exact(&mut riff_header_buf).await?;

        let riff_type = FourCC::from(BigEndian::read_u32(&riff_header_buf, 0));
        let riff_file_size = LittleEndian::read_u32(&riff_header_buf, 4);
        let riff_file_type = FourCC::from(BigEndian::read_u32(&riff_header_buf, 8));
        if riff_type != RIFF_TYPE || riff_file_size as u64 > riff_file_len || riff_file_size < 4 {
            return Err(RiffError::InvalidRiffHeader.into());
        }

        let pos: u64 = reader.seek(SeekFrom::Current(0)).await?;

        Ok(RiffTree {
            header: RiffHeader {
                file_size: riff_file_size,
                file_type: riff_file_type
            },
            childs: RiffTree::read_childs_async(reader, pos, riff_file_size - 4, riff_file_len).await?
        })
    }

    fn read_childs_async<R>(reader: &mut R, pos: u64, size: u32, file_size: u64) -> BoxFuture<Result<Vec<Box<dyn RiffNode + Send>>, Box<dyn Error>>> where R: AsyncRead + AsyncSeek + Unpin + Send + Sync {
        Box::pin(async move {
            let mut childs: Vec<Box<dyn RiffNode + Send>> = Vec::new();
            while reader.seek(SeekFrom::Current(0)).await? < pos + size as u64 {
                let next = RiffUtil::read_fourcc_async(reader).await?;
                if next == LIST_TYPE {
                    let mut list_header_buf = [0u8;8];
                    reader.read_exact(&mut list_header_buf).await?;

                    let list_size = LittleEndian::read_u32(&list_header_buf, 0);
                    let list_type = FourCC::from(BigEndian::read_u32(&list_header_buf, 4));
                    if pos + (12 - 4) + list_size as u64 >= file_size {
                        return Err(RiffError::InvalidListHeader.into());
                    }
                    let data_pos = reader.seek(SeekFrom::Current(0)).await?;
                    childs.push(Box::new(RiffList {
                        header: RiffListHeader {
                            list_type,
                            list_size,
                            data_pos,
                        },
                        childs: RiffTree::read_childs_async(reader, data_pos, list_size - 4, file_size).await?
                    }));
                } else {
                    let mut chunk_size_buf = [0u8;4];
                    reader.read_exact(&mut chunk_size_buf).await?;
                    let chunk_size = LittleEndian::read_u32(&chunk_size_buf, 0);
                    if pos + 8 + chunk_size as u64 >= file_size {
                        return Err(RiffError::InvalidChunkHeader.into());
                    }
                    let data_pos = reader.seek(SeekFrom::Current(0)).await?;
                    childs.push(Box::new(RiffChunk {
                        header: RiffChunkHeader {
                            ck_id: next,
                            ck_size: chunk_size,
                            data_pos
                        },
                        childs: vec![]
                    }));
                    let padding = RiffChunk::padding(chunk_size);
                    reader.seek(SeekFrom::Current((chunk_size + padding) as i64)).await?;
                }
            }
            Ok(childs)
        })
    }

    /**
    * Reads and parses a riff file structure
    */
    pub fn read<R>(reader: &mut R) -> Result<Self, Box<dyn Error>> where R: Read + Seek {
        let riff_file_len: u64 = reader.seek(SeekFrom::End(0))?;
        reader.seek(SeekFrom::Start(0))?;

        let mut riff_header_buf = [0u8;12];
        reader.read_exact(&mut riff_header_buf)?;

        let riff_type = FourCC::from(BigEndian::read_u32(&riff_header_buf, 0));
        let riff_file_size = LittleEndian::read_u32(&riff_header_buf, 4);
        let riff_file_type = FourCC::from(BigEndian::read_u32(&riff_header_buf, 8));
        if riff_type != RIFF_TYPE || riff_file_size as u64 > riff_file_len || riff_file_size < 4 {
            return Err(RiffError::InvalidRiffHeader.into());
        }

        let pos: u64 = reader.seek(SeekFrom::Current(0))?;

        Ok(RiffTree {
            header: RiffHeader {
                file_size: riff_file_size,
                file_type: riff_file_type
            },
            childs: RiffTree::read_childs(reader, pos, riff_file_size - 4, riff_file_len)?
        })
    }

    fn read_childs<R>(reader: &mut R, pos: u64, size: u32, file_size: u64) -> Result<Vec<Box<dyn RiffNode + Send>>, Box<dyn Error>> where R: Read + Seek {
        let mut childs: Vec<Box<dyn RiffNode + Send>> = Vec::new();
        while reader.seek(SeekFrom::Current(0))? < pos + size as u64 {
            let next = RiffUtil::read_fourcc(reader)?;

            if next == LIST_TYPE {
                let mut list_header_buf = [0u8;8];
                reader.read_exact(&mut list_header_buf)?;

                let list_size = LittleEndian::read_u32(&list_header_buf, 0);
                let list_type = FourCC::from(BigEndian::read_u32(&list_header_buf, 4));
                if pos + (12 - 4) + list_size as u64 >= file_size {
                    return Err(RiffError::InvalidListHeader.into());
                }
                let data_pos = reader.seek(SeekFrom::Current(0))?;
                childs.push(Box::new(RiffList {
                    header: RiffListHeader {
                        list_type,
                        list_size,
                        data_pos,
                    },
                    childs: RiffTree::read_childs(reader, data_pos, list_size - 4, file_size)?
                }));
            } else {
                let mut chunk_size_buf = [0u8;4];
                reader.read_exact(&mut chunk_size_buf)?;
                let chunk_size = LittleEndian::read_u32(&chunk_size_buf, 0);
                if pos + 8 + chunk_size as u64 >= file_size {
                    return Err(RiffError::InvalidChunkHeader.into());
                }
                let data_pos = reader.seek(SeekFrom::Current(0))?;
                childs.push(Box::new(RiffChunk {
                    header: RiffChunkHeader {
                        ck_id: next,
                        ck_size: chunk_size,
                        data_pos
                    },
                    childs: vec![]
                }));
                let padding = RiffChunk::padding(chunk_size);
                reader.seek(SeekFrom::Current((chunk_size + padding) as i64))?;
            }
        }
        Ok(childs)
    }

    pub fn header(&self) -> &RiffHeader {
        &self.header
    }

    pub fn childs(&self) -> &Vec<Box<dyn RiffNode + Send>> {
        &self.childs
    }
}

impl RiffHeader {

    pub fn file_size(&self) -> u32 {
        self.file_size
    }

    pub fn file_type(&self) -> FourCC {
        self.file_type
    }
}

impl RiffChunk {
    fn padding(ck_size: u32) -> u32 {
        let m = ck_size % 2;
        return if m != 0 {
            2 - m
        } else {
            0
        } as u32;
    }

    pub fn header(&self) -> RiffChunkHeader {
        self.header.clone()
    }
}

impl RiffList {
    pub fn header(&self) -> RiffListHeader {
        self.header.clone()
    }
}

impl RiffNode for RiffChunk {

    fn id(&self) -> FourCC {
        self.header.ck_id
    }

    fn data_pos(&self) -> u64 {
        self.header.data_pos
    }

    fn data_size(&self) -> u32 {
        self.header.ck_size
    }

    fn padding(&self) -> u32 {
        RiffChunk::padding(self.header.ck_size)
    }

    fn childs(&self) -> &Vec<Box<dyn RiffNode + Send>> {
        &self.childs
    }

    fn as_chunk(&self) -> Result<&RiffChunk, Box<dyn Error>> {
        Ok(self)
    }

    fn as_list(&self) -> Result<&RiffList, Box<dyn Error>> {
        Err(RiffError::InvalidListCast.into())
    }
}

impl RiffNode for RiffList {

    fn id(&self) -> FourCC {
        self.header.list_type
    }

    fn data_pos(&self) -> u64 {
        self.header.data_pos
    }

    fn data_size(&self) -> u32 {
        self.header.list_size - 4
    }

    fn padding(&self) -> u32 {
        0
    }

    fn childs(&self) -> &Vec<Box<dyn RiffNode + Send>> {
        &self.childs
    }

    fn as_chunk(&self) -> Result<&RiffChunk, Box<dyn Error>> {
        Err(RiffError::InvalidChunkCast.into())
    }

    fn as_list(&self) -> Result<&RiffList, Box<dyn Error>> {
        Ok(self)
    }
}