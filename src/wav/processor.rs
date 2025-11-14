use std::fs::File;
use std::io::{Write, Result, Error, ErrorKind};
use super::WavFile;

pub struct WavProcessor;

impl WavProcessor {
    /// 볼륨 조절 (0.0 ~ 2.0, 1.0 = 원본)
    pub fn adjust_volume(wav: &mut WavFile, volume: f32) -> Result<()> {
        if wav.header.audio_format != 1 {
            return Err(Error::new(ErrorKind::InvalidData, "Only PCM format is supported"));
        }

        match wav.header.bits_per_sample {
            16 => Self::adjust_volume_16bit(&mut wav.data, volume),
            8 => Self::adjust_volume_8bit(&mut wav.data, volume),
            _ => Err(Error::new(
                ErrorKind::InvalidData,
                format!("Unsupported bit depth: {}", wav.header.bits_per_sample),
            )),
        }
    }

    fn adjust_volume_16bit(data: &mut [u8], volume: f32) -> Result<()> {
        for i in (0..data.len()).step_by(2) {
            if i + 1 < data.len() {
                let sample = i16::from_le_bytes([data[i], data[i + 1]]);
                let adjusted = (sample as f32 * volume).clamp(i16::MIN as f32, i16::MAX as f32) as i16;
                let bytes = adjusted.to_le_bytes();
                data[i] = bytes[0];
                data[i + 1] = bytes[1];
            }
        }
        Ok(())
    }

    fn adjust_volume_8bit(data: &mut [u8], volume: f32) -> Result<()> {
        for byte in data.iter_mut() {
            // 8-bit audio는 unsigned (0-255, 중심 128)
            let sample = *byte as i16 - 128;
            let adjusted = (sample as f32 * volume).clamp(-128.0, 127.0) as i16 + 128;
            *byte = adjusted as u8;
        }
        Ok(())
    }

    /// WAV 파일로 저장
    pub fn save(wav: &WavFile, path: &str) -> Result<()> {
        let mut file = File::create(path)?;

        // RIFF 헤더
        file.write_all(b"RIFF")?;
        let file_size = 36 + wav.data.len() as u32;
        file.write_all(&file_size.to_le_bytes())?;
        file.write_all(b"WAVE")?;

        // fmt 청크
        file.write_all(b"fmt ")?;
        file.write_all(&16u32.to_le_bytes())?; // fmt 청크 크기
        file.write_all(&wav.header.audio_format.to_le_bytes())?;
        file.write_all(&wav.header.num_channels.to_le_bytes())?;
        file.write_all(&wav.header.sample_rate.to_le_bytes())?;
        file.write_all(&wav.header.byte_rate.to_le_bytes())?;
        file.write_all(&wav.header.block_align.to_le_bytes())?;
        file.write_all(&wav.header.bits_per_sample.to_le_bytes())?;

        // data 청크
        file.write_all(b"data")?;
        file.write_all(&(wav.data.len() as u32).to_le_bytes())?;
        file.write_all(&wav.data)?;

        Ok(())
    }

    /// 스테레오를 모노로 변환
    pub fn stereo_to_mono(wav: &mut WavFile) -> Result<()> {
        if wav.header.num_channels != 2 {
            return Err(Error::new(ErrorKind::InvalidData, "Not a stereo file"));
        }

        if wav.header.bits_per_sample != 16 {
            return Err(Error::new(ErrorKind::InvalidData, "Only 16-bit audio supported"));
        }

        let mut mono_data = Vec::new();

        for i in (0..wav.data.len()).step_by(4) {
            if i + 3 < wav.data.len() {
                let left = i16::from_le_bytes([wav.data[i], wav.data[i + 1]]);
                let right = i16::from_le_bytes([wav.data[i + 2], wav.data[i + 3]]);
                let mono = ((left as i32 + right as i32) / 2) as i16;
                mono_data.extend_from_slice(&mono.to_le_bytes());
            }
        }

        wav.data = mono_data;
        wav.data_size = wav.data.len() as u32;
        wav.header.num_channels = 1;
        wav.header.byte_rate = wav.header.sample_rate * wav.header.num_channels as u32 * (wav.header.bits_per_sample / 8) as u32;
        wav.header.block_align = wav.header.num_channels * (wav.header.bits_per_sample / 8);

        Ok(())
    }
}
