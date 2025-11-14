// 테스트용 WAV와 BMP 파일 생성기
use std::fs::File;
use std::io::{Write, Result};

fn main() -> Result<()> {
    generate_test_wav("test.wav")?;
    generate_test_bmp("test.bmp")?;
    println!("Test files generated:");
    println!("  - test.wav (440Hz sine wave, 1 second, stereo)");
    println!("  - test.bmp (256x256 gradient image)");
    Ok(())
}

// 1초짜리 440Hz 사인파 생성 (스테레오, 16-bit, 44100Hz)
fn generate_test_wav(path: &str) -> Result<()> {
    let sample_rate = 44100u32;
    let duration = 1.0; // seconds
    let frequency = 440.0; // A4 note
    let channels = 2u16;
    let bits_per_sample = 16u16;

    let num_samples = (sample_rate as f32 * duration) as usize;
    let mut samples = Vec::new();

    // 사인파 생성
    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;
        let value = (2.0 * std::f32::consts::PI * frequency * t).sin();
        let sample = (value * 32767.0 * 0.5) as i16; // 50% 볼륨

        // 스테레오 (양쪽 채널에 같은 값)
        samples.extend_from_slice(&sample.to_le_bytes());
        samples.extend_from_slice(&sample.to_le_bytes());
    }

    let byte_rate = sample_rate * channels as u32 * (bits_per_sample / 8) as u32;
    let block_align = channels * (bits_per_sample / 8);
    let data_size = samples.len() as u32;
    let file_size = 36 + data_size;

    let mut file = File::create(path)?;

    // RIFF 헤더
    file.write_all(b"RIFF")?;
    file.write_all(&file_size.to_le_bytes())?;
    file.write_all(b"WAVE")?;

    // fmt 청크
    file.write_all(b"fmt ")?;
    file.write_all(&16u32.to_le_bytes())?;
    file.write_all(&1u16.to_le_bytes())?; // PCM
    file.write_all(&channels.to_le_bytes())?;
    file.write_all(&sample_rate.to_le_bytes())?;
    file.write_all(&byte_rate.to_le_bytes())?;
    file.write_all(&block_align.to_le_bytes())?;
    file.write_all(&bits_per_sample.to_le_bytes())?;

    // data 청크
    file.write_all(b"data")?;
    file.write_all(&data_size.to_le_bytes())?;
    file.write_all(&samples)?;

    Ok(())
}

// 256x256 그라디언트 BMP 이미지 생성
fn generate_test_bmp(path: &str) -> Result<()> {
    let width = 256i32;
    let height = 256i32;
    let bits_per_pixel = 24u16;

    let row_size = ((width as usize * 3 + 3) / 4) * 4;
    let image_size = (row_size * height as usize) as u32;
    let file_size = 54 + image_size;

    let mut file = File::create(path)?;

    // BMP 파일 헤더
    file.write_all(b"BM")?;
    file.write_all(&file_size.to_le_bytes())?;
    file.write_all(&0u32.to_le_bytes())?;
    file.write_all(&54u32.to_le_bytes())?;

    // DIB 헤더
    file.write_all(&40u32.to_le_bytes())?;
    file.write_all(&width.to_le_bytes())?;
    file.write_all(&height.to_le_bytes())?;
    file.write_all(&1u16.to_le_bytes())?;
    file.write_all(&bits_per_pixel.to_le_bytes())?;
    file.write_all(&0u32.to_le_bytes())?;
    file.write_all(&image_size.to_le_bytes())?;
    file.write_all(&2835u32.to_le_bytes())?;
    file.write_all(&2835u32.to_le_bytes())?;
    file.write_all(&0u32.to_le_bytes())?;
    file.write_all(&0u32.to_le_bytes())?;

    // 픽셀 데이터 (그라디언트)
    let mut pixel_data = vec![0u8; image_size as usize];
    for y in 0..height as usize {
        for x in 0..width as usize {
            let idx = y * row_size + x * 3;
            // BGR 순서
            pixel_data[idx] = x as u8;         // Blue
            pixel_data[idx + 1] = y as u8;     // Green
            pixel_data[idx + 2] = ((x + y) / 2) as u8; // Red
        }
    }

    file.write_all(&pixel_data)?;

    Ok(())
}
