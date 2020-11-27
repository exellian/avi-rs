use tokio::io::{AsyncRead, AsyncSeek, AsyncReadExt, SeekFrom, AsyncSeekExt};
use std::error::Error;
use crate::fourcc::FourCC;
use std::marker::PhantomData;
use std::fmt::{Debug, Display, Formatter};
use crate::bytes::{BigEndian, LittleEndian};
use crate::riff::{RiffHeader, RiffUtil, RiffTree, RiffChunk, RiffList, RiffListHeader, RiffChunkHeader};
use std::io::{Cursor, Read};
use std::ffi::CString;
use std::collections::HashMap;
use crate::AviError::InvalidMoviList;
use std::fmt;
use crate::mmreg::{WAVE_FORMAT_PCM, WAVE_FORMAT_EXTENSIBLE};

mod fourcc;
pub mod bytes;
pub mod riff;
mod mmreg;


const AVI_FILE_TYPE: FourCC = FourCC::from_bytes(b"AVI ");
const HDRL_TYPE: FourCC = FourCC::from_bytes(b"hdrl");
const MOVI_TYPE: FourCC = FourCC::from_bytes(b"movi");
const AVIH_TYPE: FourCC = FourCC::from_bytes(b"avih");
const STRL_TYPE: FourCC = FourCC::from_bytes(b"strl");
const STRH_TYPE: FourCC = FourCC::from_bytes(b"strh");
const STRF_TYPE: FourCC = FourCC::from_bytes(b"strf");
const STRD_TYPE: FourCC = FourCC::from_bytes(b"strd");
const STRN_TYPE: FourCC = FourCC::from_bytes(b"strn");
const IDX1_TYPE: FourCC = FourCC::from_bytes(b"idx1");

const AUDIO_STREAM_TYPE: FourCC = FourCC::from_bytes(b"auds");
const MIDI_STREAM_TYPE: FourCC = FourCC::from_bytes(b"mids");
const TXT_STREAM_TYPE: FourCC = FourCC::from_bytes(b"txts");
const VIDEO_STREAM_TYPE: FourCC = FourCC::from_bytes(b"vids");

//Max stream count because of fourcc limits f. e. (00wb - 99wb)
const AVI_MAX_STREAMS: usize = 100;




#[derive(Debug)]
pub enum AviError {
    DuplicateHdrlList,
    DuplicateMoviList,
    DuplicateIdx1Chunk,
    HdrlNotFound,
    MoviNotFound,
    InvalidRiffFileType,
    InvalidHdrlList,
    InvalidMoviList,
    InvalidMainHeader,
    InvalidStreamList,
    InvalidStreamHeader,
    InvalidStreamFormatHeader,
    InvalidStreamAdditionalData,
    InvalidAviMoviHeader,
    InvalidIndexHeader,
    InvalidRecordList,
    InvalidBufferReadSize,
    ChunkInRecordList,
    UnsupportedStreamType,
}

impl Display for AviError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            AviError::InvalidRiffFileType => {
                write!(f, "Riff file type invalid!")
            },
            _ => { unreachable!() }
        }
    }
}

impl Error for AviError {}

pub struct AviFrame {

}

#[derive(Debug, Clone)]
struct RiffChunkList {
    header: RiffListHeader,
    childs: Vec<RiffChunkHeader>
}

#[derive(Debug)]
pub struct AviStreamChunk {
    rec_index: Option<usize>,
    /**
    * chunk_index is the index in a record list or the index in the movi list itself
    */
    chunk_index: usize,
    stream_index: usize,
    chunk: RiffChunkHeader
}

#[derive(Debug)]
pub struct AviStream {
    index: usize,
    format: AviStreamFormat,
    chunks: Vec<AviStreamChunk>,
}

#[derive(Debug)]
pub struct AviAsyncReader<R> where R: AsyncRead + AsyncSeek + Unpin + Send + Sync {
    reader: R,
    header: AviHeader,
    riff_tree: RiffTree,
    movi: Vec<AviStream>,
    recs: Vec<RiffChunkList>
}

#[derive(Debug)]
pub struct Rect {
    left: i16,
    top: i16,
    right: i16,
    bottom: i16
}


#[derive(Debug, Clone)]
pub struct Guid {
    data1: u32,
    data2: u16,
    data3: u16,
    data4: [u8;8]
}

#[derive(Debug)]
pub struct AviHeader {
    avih: AviMainHeader,
    strl: Vec<AviStreamListItem>
}

#[derive(Debug)]
pub struct AviMainHeader {
    dw_micro_sec_per_frame: u32,
    dw_max_bytes_per_sec: u32,
    dw_padding_granularity: u32,
    dw_flags: u32,
    dw_total_frames: u32,
    dw_initial_frames: u32,
    dw_streams: u32,
    dw_suggested_buffer_size: u32,
    dw_width: u32,
    dw_height: u32,
    dw_reserved: [u32;4]
}

#[derive(Debug)]
pub struct AviStreamHeader {
    fcc_type: FourCC,
    fcc_handler: FourCC,
    dw_flags: u32,
    w_priority: u16,
    w_language: u16,
    dw_initial_frames: u32,
    dw_scale: u32,
    dw_rate: u32,
    dw_start: u32,
    dw_length: u32,
    dw_suggested_buffer_size: u32,
    dw_quality: u32,
    dw_sample_size: u32,
    rc_frame: Rect
}

#[derive(Debug, Clone)]
pub struct AviStreamFormat {
    video: Option<AviBitmapInfo>,
    audio: Option<AviWaveInfoExt>
}

#[derive(Debug, Clone)]
pub struct AviBitmapInfo {
    bi_size: u32,
    bi_width: i32,
    bi_height: i32,
    bi_planes: u16,
    bi_bit_count: u16,
    bi_compression: u32,
    bi_size_image: u32,
    bi_x_pels_per_meter: i32,
    bi_y_pels_per_meter: i32,
    bi_clr_used: u32,
    bi_clr_important: u32,
}

#[derive(Clone, Copy)]
pub union AviWaveExtraSampleInfo {
    w_valid_bits_per_sample: u16,
    w_samples_per_block: u16,
    w_reserved: u16
}

impl Debug for AviWaveExtraSampleInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", unsafe { self.w_reserved })
    }
}

#[derive(Debug, Clone)]
pub struct AviWaveInfo {
    w_format_tag: u16,
    n_channels: u16,
    n_samples_per_sec: u32,
    n_avg_bytes_per_sec: u32,
    n_block_align: u16,
    w_bits_per_sample: u16,
    cb_size: Option<u16>
}

#[derive(Debug, Clone)]
pub struct AviWaveExtraInfo {
    samples: AviWaveExtraSampleInfo,
    dw_channel_mask: u32,
    sub_format: Guid
}

#[derive(Debug, Clone)]
pub struct AviWaveInfoExt {
    format: AviWaveInfo,
    extra: Option<AviWaveExtraInfo>
}

#[derive(Debug)]
pub struct AviStreamListItem {
    index: usize,
    strh: AviStreamHeader,
    strf: AviStreamFormat,
    strd: Option<Vec<u8>>,
    strn: Option<Vec<u8>>
}

impl AviMainHeader {

    async fn read_async<R>(reader: &mut R) -> Result<Self, Box<dyn Error>> where R: AsyncRead + Unpin + Send + Sync {
        let mut buf = [0u8;std::mem::size_of::<AviMainHeader>()];
        reader.read_exact(&mut buf).await?;

        //Last four WORDS must be zero
        if LittleEndian::read_u32(&buf, 40) != 0 ||
            LittleEndian::read_u32(&buf, 44) != 0 ||
            LittleEndian::read_u32(&buf, 48) != 0 ||
            LittleEndian::read_u32(&buf, 52) != 0 {
            return Err(AviError::InvalidMainHeader.into());
        }

        Ok(AviMainHeader {
            dw_micro_sec_per_frame: LittleEndian::read_u32(&buf, 0),
            dw_max_bytes_per_sec: LittleEndian::read_u32(&buf, 4),
            dw_padding_granularity: LittleEndian::read_u32(&buf, 8),
            dw_flags: LittleEndian::read_u32(&buf, 12),
            dw_total_frames: LittleEndian::read_u32(&buf, 16),
            dw_initial_frames: LittleEndian::read_u32(&buf, 20),
            dw_streams: LittleEndian::read_u32(&buf, 24),
            dw_suggested_buffer_size: LittleEndian::read_u32(&buf, 28),
            dw_width: LittleEndian::read_u32(&buf, 32),
            dw_height: LittleEndian::read_u32(&buf, 36),
            dw_reserved: [0, 0, 0, 0]
        })
    }
}

impl AviStreamHeader {

    async fn read_async<R>(reader: &mut R) -> Result<Self, Box<dyn Error>> where R: AsyncRead + Unpin + Send + Sync {
        let mut buf = [0u8;std::mem::size_of::<AviStreamHeader>()];
        reader.read_exact(&mut buf).await?;

        Ok(AviStreamHeader {
            fcc_type: FourCC::from(BigEndian::read_u32(&buf, 0)),
            fcc_handler: FourCC::from(BigEndian::read_u32(&buf, 4)),
            dw_flags: LittleEndian::read_u32(&buf, 8),
            w_priority: LittleEndian::read_u16(&buf, 12),
            w_language: LittleEndian::read_u16(&buf, 14),
            dw_initial_frames: LittleEndian::read_u32(&buf, 16),
            dw_scale: LittleEndian::read_u32(&buf, 20),
            dw_rate: LittleEndian::read_u32(&buf, 24),
            dw_start: LittleEndian::read_u32(&buf, 28),
            dw_length: LittleEndian::read_u32(&buf, 32),
            dw_suggested_buffer_size: LittleEndian::read_u32(&buf, 36),
            dw_quality: LittleEndian::read_u32(&buf, 40),
            dw_sample_size: LittleEndian::read_u32(&buf, 44),
            rc_frame: Rect {
                left: LittleEndian::read_i16(&buf, 48),
                top: LittleEndian::read_i16(&buf, 50),
                right: LittleEndian::read_i16(&buf, 52),
                bottom: LittleEndian::read_i16(&buf, 54)
            }
        })
    }
}

impl AviBitmapInfo {
    async fn read_async<R>(reader: &mut R) -> Result<Self, Box<dyn Error>> where R: AsyncRead + Unpin + Send + Sync {
        let mut buf = [0u8;std::mem::size_of::<AviBitmapInfo>()];
        reader.read_exact(&mut buf).await?;

        Ok(AviBitmapInfo {
            bi_size: LittleEndian::read_u32(&buf, 0),
            bi_width: LittleEndian::read_i32(&buf, 4),
            bi_height: LittleEndian::read_i32(&buf, 8),
            bi_planes: LittleEndian::read_u16(&buf, 12),
            bi_bit_count: LittleEndian::read_u16(&buf, 14),
            bi_compression: LittleEndian::read_u32(&buf, 16),
            bi_size_image: LittleEndian::read_u32(&buf, 20),
            bi_x_pels_per_meter: LittleEndian::read_i32(&buf, 24),
            bi_y_pels_per_meter: LittleEndian::read_i32(&buf, 28),
            bi_clr_used: LittleEndian::read_u32(&buf, 32),
            bi_clr_important: LittleEndian::read_u32(&buf, 36)
        })
    }
}

impl AviWaveInfo {

    async fn read_async<R>(reader: &mut R) -> Result<Self, Box<dyn Error>> where R: AsyncRead + Unpin + Send + Sync {
        let mut buf = [0u8;16];
        reader.read_exact(&mut buf).await?;

        let w_format_tag = LittleEndian::read_u16(&buf, 0);
        let mut cb_size = None;
        if w_format_tag != WAVE_FORMAT_PCM {
            let mut cb_size_buf = [0u8;2];
            reader.read_exact(&mut cb_size_buf).await?;
            cb_size = Some(LittleEndian::read_u16(&cb_size_buf, 0))
        }

        Ok(AviWaveInfo {
            w_format_tag,
            n_channels: LittleEndian::read_u16(&buf, 2),
            n_samples_per_sec: LittleEndian::read_u32(&buf, 4),
            n_avg_bytes_per_sec: LittleEndian::read_u32(&buf, 8),
            n_block_align: LittleEndian::read_u16(&buf, 12),
            w_bits_per_sample: LittleEndian::read_u16(&buf, 14),
            cb_size
        })
    }
}

impl AviWaveInfoExt {

    async fn read_async<R>(reader: &mut R) -> Result<Self, Box<dyn Error>> where R: AsyncRead + Unpin + Send + Sync {
        let format = AviWaveInfo::read_async(reader).await?;
        let mut extra = None;
        if format.w_format_tag == WAVE_FORMAT_EXTENSIBLE {
            let mut buf = [0u8;std::mem::size_of::<AviWaveExtraInfo>()];
            let mut data4 = [0u8;8];
            reader.read_exact(&mut buf).await?;

            for i in 14..22 {
                data4[i] = buf[i];
            }

            extra = Some(AviWaveExtraInfo {
                samples: AviWaveExtraSampleInfo {
                    w_valid_bits_per_sample: LittleEndian::read_u16(&buf, 0)
                },
                dw_channel_mask: LittleEndian::read_u32(&buf, 2),
                sub_format: Guid {
                    data1: LittleEndian::read_u32(&buf, 6),
                    data2: LittleEndian::read_u16(&buf, 10),
                    data3: LittleEndian::read_u16(&buf, 12),
                    data4
                }
            });
        }
        Ok(AviWaveInfoExt {
            format,
            extra
        })
    }
}

pub struct AviUtil;
impl AviUtil {
    fn parse_stream_index(fourcc: &FourCC) -> Result<usize, Box<dyn Error>> {
        let buf: [u8;4] = fourcc.into();
        let index_buf = [buf[0], buf[1]];
        let str = std::str::from_utf8(&index_buf)?;
        let index: u8 = str.parse()?;
        Ok(index as usize)
    }
}

impl <R> AviAsyncReader<R> where R: AsyncRead + AsyncSeek + Unpin + Send + Sync {

    /**
    * Reads a "standalone" chunk from the avi reader.
    * Standalone means that the chunk is not part of a record list and can be read independently
    */
    pub async fn read_standalone_chunk(&mut self, chunk: &AviStreamChunk, buf: &mut [u8]) -> Result<(), Box<dyn Error>> {

        if chunk.rec_index.is_some() {
            return Err(AviError::ChunkInRecordList.into());
        }

        let chunk_size = chunk.chunk.data_size() as usize;

        self.reader.seek(SeekFrom::Start(chunk.chunk.data_pos())).await?;
        self.reader.read(&mut buf[0..chunk_size]).await?;

        Ok(())
    }

    /**
    * Reads all chunks of a record list
    * buf needs to be size of record list + header chunks + chunk data
    */
    pub async fn read_record_list<'a>(&mut self, record_list_index: usize, buf: &'a mut [u8]) -> Result<Vec<&'a [u8]>, Box<dyn Error>> {
        let records = match self.recs.get(record_list_index) {
            None => return Err(AviError::InvalidRecordList.into()),
            Some(l) => l
        };

        let records_pos = records.header.data_pos();
        self.reader.seek(SeekFrom::Start(records_pos)).await?;
        self.reader.read_exact(buf).await?;

        let mut slices = Vec::with_capacity(records.childs.len());
        for chunk in &records.childs {
            let relative_pos = (chunk.data_pos() - records_pos) as usize;
            let relative_end = relative_pos + chunk.data_size() as usize;
            slices.push(&buf[relative_pos..relative_end]);
        }
        Ok(slices)
    }

    pub async fn read_header(mut reader: R) -> Result<Self, Box<dyn Error>>  {
        let riff_tree = RiffTree::read_async(&mut reader).await?;
        let riff_header = riff_tree.header();
        let riff_childs = riff_tree.childs();
        if riff_header.file_type() != AVI_FILE_TYPE {
            return Err(AviError::InvalidRiffFileType.into());
        }

        let mut hdrl_node = None;
        let mut movi_node = None;
        let mut idx1_node = None;

        for child in riff_childs {
            let id = child.id();
            if id == HDRL_TYPE {
                if hdrl_node.is_some() {
                    return Err(AviError::DuplicateHdrlList.into());
                }
                hdrl_node = Some(child);
            } else if id == MOVI_TYPE {
                if movi_node.is_some() {
                    return Err(AviError::DuplicateMoviList.into());
                }
                movi_node = Some(child);
            } else if id == IDX1_TYPE {
                if idx1_node.is_some() {
                    return Err(AviError::DuplicateIdx1Chunk.into());
                }
                idx1_node = Some(child);
            }
        }
        let hdrl = match hdrl_node {
            Some(h) => h,
            None => return Err(AviError::HdrlNotFound.into())
        };

        //Parsing hdrl
        let header = {
            let hdrl_childs = hdrl.childs();
            let hdrl_childs_len = hdrl_childs.len();
            if hdrl_childs_len > (AVI_MAX_STREAMS + 1) {
                return Err(AviError::InvalidHdrlList.into());
            }
            let avih = &hdrl_childs[0];
            if avih.data_size() as usize != std::mem::size_of::<AviMainHeader>() {
                return Err(AviError::InvalidMainHeader.into());
            }
            reader.seek(SeekFrom::Start(avih.data_pos())).await?;
            let avih = AviMainHeader::read_async(&mut reader).await?;

            let mut strl = Vec::new();

            //Parse strl
            for i in 1..hdrl_childs_len {

                let child = &hdrl_childs[i];
                let strl_childs = child.childs();
                let strl_childs_len = strl_childs.len();
                if  strl_childs_len < 2 || strl_childs_len > 4{
                    return Err(AviError::InvalidStreamList.into());
                }
                let strh = &strl_childs[0];
                if strh.data_size() as usize != std::mem::size_of::<AviStreamHeader>() {
                    return Err(AviError::InvalidStreamHeader.into());
                }
                reader.seek(SeekFrom::Start(strh.data_pos())).await?;
                let strh = AviStreamHeader::read_async(&mut reader).await?;

                let strf_header = &strl_childs[1];
                let strf;
                if strh.fcc_type == VIDEO_STREAM_TYPE {
                    if strf_header.data_size() as usize != std::mem::size_of::<AviBitmapInfo>() {
                        return Err(AviError::InvalidStreamFormatHeader.into());
                    }
                    //We don't need to seek because already on the correct pos
                    let abih = AviBitmapInfo::read_async(&mut reader).await?;
                    strf = AviStreamFormat {
                        video: Some(abih),
                        audio: None,
                    };
                } else if strh.fcc_type == AUDIO_STREAM_TYPE {
                    //WAVEFORMATEX struct min len is 16 bytes because cb_size possible not present
                    if (strf_header.data_size() as usize) < (std::mem::size_of::<AviWaveInfo>() - std::mem::size_of::<Option<u16>>()) {
                        return Err(AviError::InvalidStreamFormatHeader.into());
                    }
                    //We don't need to seek because already on the correct pos
                    let awie = AviWaveInfoExt::read_async(&mut reader).await?;
                    strf = AviStreamFormat {
                        video: None,
                        audio: Some(awie)
                    };
                } else {
                    return Err(AviError::UnsupportedStreamType.into());
                }

                let mut strl_item = AviStreamListItem {
                    index: i - 1,
                    strh,
                    strf,
                    strd: None,
                    strn: None
                };

                if strl_childs_len == 2 {
                    strl.push(strl_item);
                    continue;
                }

                if strl_childs_len == 3 {
                    let last = &strl_childs[2];
                    let last_id = last.id();
                    if last_id != STRN_TYPE || last_id != STRD_TYPE {
                        return Err(AviError::InvalidStreamAdditionalData.into());
                    }
                    let mut buf = vec![0; (last.data_size() + last.padding()) as usize];
                    reader.read_exact(&mut buf).await?;
                    if last_id == STRN_TYPE {
                        strl_item.strn = Some(buf);
                    } else {
                        strl_item.strd = Some(buf);
                    }
                    strl.push(strl_item);
                    continue;
                }

                let strd = &strl_childs[2];
                let strn = &strl_childs[3];

                if strd.id() != STRD_TYPE || strn.id() != STRN_TYPE {
                    return Err(AviError::InvalidStreamAdditionalData.into());
                }

                let mut strd_buf = vec![0; (strd.data_size() + strd.padding()) as usize];
                reader.read_exact(&mut strd_buf).await?;

                let mut strn_buf = vec![0; (strn.data_size() + strn.padding()) as usize];
                reader.read_exact(&mut strn_buf).await?;

                strl_item.strd = Some(strd_buf);
                strl_item.strn = Some(strn_buf);
                strl.push(strl_item);
            }
            AviHeader {
                avih,
                strl
            }
        };

        let mut movi = Vec::with_capacity(header.strl.len());

        let mut i = 0;
        for item in &header.strl {
            movi.push(AviStream {
                index: i,
                format: item.strf.clone(),
                chunks: vec![]
            });
            i += 1;
        }

        let mut recs = Vec::new();

        //Parse movi list
        match movi_node {
            None => return Err(AviError::MoviNotFound.into()),
            Some(node) => {
                for rec_or_chunk in node.childs() {
                    if rec_or_chunk.childs().is_empty() {
                        let stream_index = AviUtil::parse_stream_index(&rec_or_chunk.id())?;
                        let stream = match movi.get_mut(stream_index) {
                            Some(s) => s,
                            None => return Err(AviError::InvalidMoviList.into())
                        };
                        stream.chunks.push(AviStreamChunk {
                            rec_index: None,
                            chunk_index: stream.chunks.len(),
                            stream_index,
                            chunk: rec_or_chunk.as_chunk()?.header()
                        });
                    } else {
                        let rec_list = rec_or_chunk.as_list()?;
                        recs.push(RiffChunkList {
                            header: rec_list.header(),
                            childs: vec![]
                        });
                        let recs_index = recs.len() - 1;
                        let mut j = 0;
                        for chunk in rec_or_chunk.childs() {
                            if !chunk.childs().is_empty() {
                                return Err(AviError::InvalidMoviList.into());
                            }
                            recs.get_mut(recs_index).unwrap().childs.push(chunk.as_chunk()?.header());
                            let stream_index = AviUtil::parse_stream_index(&chunk.id())?;
                            let stream = match movi.get_mut(stream_index) {
                                Some(s) => s,
                                None => return Err(AviError::InvalidMoviList.into())
                            };
                            stream.chunks.push(AviStreamChunk {
                                rec_index: Some(recs_index),
                                chunk_index: j,
                                stream_index,
                                chunk: chunk.as_chunk()?.header()
                            });
                            j += 1;
                        }
                    }
                }
            }
        };

        Ok(AviAsyncReader {
            reader,
            header,
            riff_tree,
            movi,
            recs
        })
    }

}
