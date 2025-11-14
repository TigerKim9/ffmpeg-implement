use std::fs::File;
use std::io::{Read, Result, Error, ErrorKind};

#[derive(Debug, Clone)]
pub struct BmpHeader {
    pub width: i32,
    pub height: i32,
    pub bits_per_pixel: u16,
    pub compression: u32,
    #[allow(dead_code)]
    pub image_size: u32,
    #[allow(dead_code)]
    pub data_offset: u32,
}

#[derive(Debug)]
pub struct BmpFile {
    pub header: BmpHeader,
    pub data: Vec<u8>,
}

impl BmpFile {
    pub fn from_file(path: &str) -> Result<Self> {
        let mut file = File::open(path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        Self::parse(&buffer)
    }

    fn parse(data: &[u8]) -> Result<Self> {
        if data.len() < 54 {
            return Err(Error::new(ErrorKind::InvalidData, "File too small to be a BMP file"));
        }

        // BMP 시그니처 확인
        if &data[0..2] != b"BM" {
            return Err(Error::new(ErrorKind::InvalidData, "Not a BMP file"));
        }

        // 파일 헤더 (14 bytes)
        let data_offset = u32::from_le_bytes([data[10], data[11], data[12], data[13]]);

        // DIB 헤더 크기
        let dib_header_size = u32::from_le_bytes([data[14], data[15], data[16], data[17]]);

        if dib_header_size < 40 {
            return Err(Error::new(ErrorKind::InvalidData, "Unsupported BMP header size"));
        }

        // DIB 헤더 파싱 (BITMAPINFOHEADER)
        let width = i32::from_le_bytes([data[18], data[19], data[20], data[21]]);
        let height = i32::from_le_bytes([data[22], data[23], data[24], data[25]]);
        let bits_per_pixel = u16::from_le_bytes([data[28], data[29]]);
        let compression = u32::from_le_bytes([data[30], data[31], data[32], data[33]]);
        let image_size = u32::from_le_bytes([data[34], data[35], data[36], data[37]]);

        if compression != 0 {
            return Err(Error::new(ErrorKind::InvalidData, "Compressed BMP files are not supported"));
        }

        if bits_per_pixel != 24 && bits_per_pixel != 32 {
            return Err(Error::new(
                ErrorKind::InvalidData,
                format!("Only 24-bit and 32-bit BMP files are supported, got {}", bits_per_pixel),
            ));
        }

        let header = BmpHeader {
            width,
            height,
            bits_per_pixel,
            compression,
            image_size,
            data_offset,
        };

        // 픽셀 데이터 추출
        let pixel_data = data[data_offset as usize..].to_vec();

        Ok(BmpFile {
            header,
            data: pixel_data,
        })
    }

    pub fn print_info(&self) {
        println!("BMP File Information:");
        println!("  Width: {} pixels", self.header.width);
        println!("  Height: {} pixels", self.header.height.abs());
        println!("  Bits per Pixel: {}", self.header.bits_per_pixel);
        println!("  Compression: {}", if self.header.compression == 0 { "None" } else { "Compressed" });
        println!("  Image Size: {} bytes", self.data.len());
        println!("  Orientation: {}", if self.header.height < 0 { "Top-down" } else { "Bottom-up" });
    }

    /// 픽셀 데이터를 RGB888 포맷으로 반환 (폭 * 높이 * 3 바이트)
    pub fn get_rgb_data(&self) -> Vec<u8> {
        let width = self.header.width as usize;
        let height = self.header.height.abs() as usize;
        let bytes_per_pixel = (self.header.bits_per_pixel / 8) as usize;

        // BMP의 행은 4바이트로 정렬됨
        let row_size = ((width * bytes_per_pixel + 3) / 4) * 4;

        let mut rgb_data = Vec::with_capacity(width * height * 3);

        for y in 0..height {
            let row_start = y * row_size;

            for x in 0..width {
                let pixel_start = row_start + x * bytes_per_pixel;

                if pixel_start + 2 < self.data.len() {
                    // BMP는 BGR 순서로 저장됨
                    let b = self.data[pixel_start];
                    let g = self.data[pixel_start + 1];
                    let r = self.data[pixel_start + 2];

                    rgb_data.push(r);
                    rgb_data.push(g);
                    rgb_data.push(b);
                }
            }
        }

        rgb_data
    }
}
