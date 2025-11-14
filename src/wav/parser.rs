use std::fs::File;
use std::io::{Read, Result, Error, ErrorKind};

#[derive(Debug, Clone)]
pub struct WavHeader {
    pub audio_format: u16,      // 1 = PCM
    pub num_channels: u16,      // 1 = Mono, 2 = Stereo
    pub sample_rate: u32,       // 44100, 48000, etc.
    pub byte_rate: u32,         // SampleRate * NumChannels * BitsPerSample/8
    pub block_align: u16,       // NumChannels * BitsPerSample/8
    pub bits_per_sample: u16,   // 8, 16, 24, 32
}

#[derive(Debug)]
pub struct WavFile {
    pub header: WavHeader,
    pub data: Vec<u8>,
    pub data_size: u32,
}

impl WavFile {
    pub fn from_file(path: &str) -> Result<Self> {
        let mut file = File::open(path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        Self::parse(&buffer)
    }

    fn parse(data: &[u8]) -> Result<Self> {
        if data.len() < 44 {
            return Err(Error::new(ErrorKind::InvalidData, "File too small to be a WAV file"));
        }

        // RIFF 헤더 확인
        if &data[0..4] != b"RIFF" {
            return Err(Error::new(ErrorKind::InvalidData, "Not a RIFF file"));
        }

        if &data[8..12] != b"WAVE" {
            return Err(Error::new(ErrorKind::InvalidData, "Not a WAVE file"));
        }

        // fmt 청크 찾기
        let mut pos = 12;
        let (fmt_pos, fmt_size) = Self::find_chunk(data, pos, b"fmt ")?;

        if fmt_size < 16 {
            return Err(Error::new(ErrorKind::InvalidData, "fmt chunk too small"));
        }

        // fmt 청크 파싱
        let fmt_data = &data[fmt_pos..fmt_pos + fmt_size];
        let header = WavHeader {
            audio_format: u16::from_le_bytes([fmt_data[0], fmt_data[1]]),
            num_channels: u16::from_le_bytes([fmt_data[2], fmt_data[3]]),
            sample_rate: u32::from_le_bytes([fmt_data[4], fmt_data[5], fmt_data[6], fmt_data[7]]),
            byte_rate: u32::from_le_bytes([fmt_data[8], fmt_data[9], fmt_data[10], fmt_data[11]]),
            block_align: u16::from_le_bytes([fmt_data[12], fmt_data[13]]),
            bits_per_sample: u16::from_le_bytes([fmt_data[14], fmt_data[15]]),
        };

        // data 청크 찾기
        pos = fmt_pos + fmt_size;
        let (data_pos, data_size) = Self::find_chunk(data, pos, b"data")?;

        let audio_data = data[data_pos..data_pos + data_size].to_vec();

        Ok(WavFile {
            header,
            data: audio_data,
            data_size: data_size as u32,
        })
    }

    fn find_chunk(data: &[u8], start: usize, chunk_id: &[u8]) -> Result<(usize, usize)> {
        let mut pos = start;

        while pos + 8 <= data.len() {
            let id = &data[pos..pos + 4];
            let size = u32::from_le_bytes([
                data[pos + 4],
                data[pos + 5],
                data[pos + 6],
                data[pos + 7],
            ]) as usize;

            if id == chunk_id {
                return Ok((pos + 8, size));
            }

            pos += 8 + size;
            // 청크는 2바이트 정렬
            if size % 2 != 0 {
                pos += 1;
            }
        }

        Err(Error::new(
            ErrorKind::InvalidData,
            format!("Chunk '{}' not found", String::from_utf8_lossy(chunk_id)),
        ))
    }

    pub fn duration_secs(&self) -> f64 {
        let num_samples = self.data_size as f64 / (self.header.bits_per_sample / 8) as f64 / self.header.num_channels as f64;
        num_samples / self.header.sample_rate as f64
    }

    pub fn print_info(&self) {
        println!("WAV File Information:");
        println!("  Format: {}", if self.header.audio_format == 1 { "PCM" } else { "Unknown" });
        println!("  Channels: {}", self.header.num_channels);
        println!("  Sample Rate: {} Hz", self.header.sample_rate);
        println!("  Bit Depth: {} bits", self.header.bits_per_sample);
        println!("  Byte Rate: {} bytes/sec", self.header.byte_rate);
        println!("  Duration: {:.2} seconds", self.duration_secs());
        println!("  Data Size: {} bytes", self.data_size);
    }
}
