use std::fs::File;
use std::io::{Write, Result, Error, ErrorKind};
use super::BmpFile;

pub struct BmpProcessor;

impl BmpProcessor {
    /// 최근접 이웃 알고리즘을 사용한 이미지 리사이징
    pub fn resize(bmp: &BmpFile, new_width: u32, new_height: u32) -> Result<BmpFile> {
        if new_width == 0 || new_height == 0 {
            return Err(Error::new(ErrorKind::InvalidInput, "Width and height must be greater than 0"));
        }

        let old_width = bmp.header.width as u32;
        let old_height = bmp.header.height.abs() as u32;

        let rgb_data = bmp.get_rgb_data();

        let mut new_data = Vec::with_capacity((new_width * new_height * 3) as usize);

        // 최근접 이웃 리샘플링
        for y in 0..new_height {
            for x in 0..new_width {
                let src_x = (x * old_width / new_width).min(old_width - 1);
                let src_y = (y * old_height / new_height).min(old_height - 1);

                let src_idx = ((src_y * old_width + src_x) * 3) as usize;

                if src_idx + 2 < rgb_data.len() {
                    new_data.push(rgb_data[src_idx]);
                    new_data.push(rgb_data[src_idx + 1]);
                    new_data.push(rgb_data[src_idx + 2]);
                }
            }
        }

        // 새 BMP 파일 생성
        Self::create_bmp_from_rgb(&new_data, new_width as i32, new_height as i32, bmp.header.height < 0)
    }

    /// RGB 데이터로부터 BMP 파일 생성
    fn create_bmp_from_rgb(rgb_data: &[u8], width: i32, height: i32, top_down: bool) -> Result<BmpFile> {
        let height_value = if top_down { -height } else { height };
        let row_size = ((width as usize * 3 + 3) / 4) * 4;
        let image_size = (row_size * height.abs() as usize) as u32;

        let mut pixel_data = vec![0u8; image_size as usize];

        for y in 0..height.abs() as usize {
            for x in 0..width as usize {
                let src_idx = (y * width as usize + x) * 3;
                let dst_idx = y * row_size + x * 3;

                if src_idx + 2 < rgb_data.len() && dst_idx + 2 < pixel_data.len() {
                    // RGB를 BGR로 변환
                    pixel_data[dst_idx] = rgb_data[src_idx + 2];     // B
                    pixel_data[dst_idx + 1] = rgb_data[src_idx + 1]; // G
                    pixel_data[dst_idx + 2] = rgb_data[src_idx];     // R
                }
            }
        }

        let header = super::BmpHeader {
            width,
            height: height_value,
            bits_per_pixel: 24,
            compression: 0,
            image_size,
            data_offset: 54,
        };

        Ok(BmpFile {
            header,
            data: pixel_data,
        })
    }

    /// BMP 파일 저장
    pub fn save(bmp: &BmpFile, path: &str) -> Result<()> {
        let mut file = File::create(path)?;

        let row_size = ((bmp.header.width as usize * (bmp.header.bits_per_pixel as usize / 8) + 3) / 4) * 4;
        let image_size = (row_size * bmp.header.height.abs() as usize) as u32;
        let file_size = 54 + image_size;

        // BMP 파일 헤더 (14 bytes)
        file.write_all(b"BM")?;
        file.write_all(&file_size.to_le_bytes())?;
        file.write_all(&0u32.to_le_bytes())?; // Reserved
        file.write_all(&54u32.to_le_bytes())?; // Data offset

        // DIB 헤더 (40 bytes - BITMAPINFOHEADER)
        file.write_all(&40u32.to_le_bytes())?; // Header size
        file.write_all(&bmp.header.width.to_le_bytes())?;
        file.write_all(&bmp.header.height.to_le_bytes())?;
        file.write_all(&1u16.to_le_bytes())?; // Planes
        file.write_all(&bmp.header.bits_per_pixel.to_le_bytes())?;
        file.write_all(&0u32.to_le_bytes())?; // Compression
        file.write_all(&image_size.to_le_bytes())?;
        file.write_all(&2835u32.to_le_bytes())?; // X pixels per meter
        file.write_all(&2835u32.to_le_bytes())?; // Y pixels per meter
        file.write_all(&0u32.to_le_bytes())?; // Colors used
        file.write_all(&0u32.to_le_bytes())?; // Important colors

        // 픽셀 데이터
        file.write_all(&bmp.data)?;

        Ok(())
    }

    /// 그레이스케일 변환
    pub fn to_grayscale(bmp: &BmpFile) -> Result<BmpFile> {
        let width = bmp.header.width;
        let height = bmp.header.height.abs();
        let rgb_data = bmp.get_rgb_data();

        let mut gray_data = Vec::with_capacity(rgb_data.len());

        for i in (0..rgb_data.len()).step_by(3) {
            if i + 2 < rgb_data.len() {
                let r = rgb_data[i] as f32;
                let g = rgb_data[i + 1] as f32;
                let b = rgb_data[i + 2] as f32;

                // 인간의 시각 인지를 고려한 그레이스케일 변환
                let gray = (0.299 * r + 0.587 * g + 0.114 * b) as u8;

                gray_data.push(gray);
                gray_data.push(gray);
                gray_data.push(gray);
            }
        }

        Self::create_bmp_from_rgb(&gray_data, width, height, bmp.header.height < 0)
    }
}
