# ffmpeg-rs

간단한 미디어 처리 도구입니다. 코덱 구현은 순수 Rust 표준 라이브러리만 사용합니다.

## 특징

- **순수 Rust 코덱 구현**: 코덱은 외부 의존성 없이 std만 사용
- **오디오 처리**: WAV 파일 정보 읽기, 볼륨 조절, 스테레오→모노 변환
- **이미지 처리**: BMP 파일 정보 읽기, 리사이징, 그레이스케일 변환
- **2가지 인터페이스**:
  - CLI - 터미널에서 직접 사용
  - GUI - egui 기반 그래픽 인터페이스

## 지원 포맷

### 오디오
- **WAV** (PCM, 8/16-bit)
  - 압축되지 않은 오디오만 지원
  - 44.1kHz, 48kHz 등 다양한 샘플레이트 지원

### 이미지
- **BMP** (24-bit, 32-bit)
  - 압축되지 않은 비트맵만 지원
  - RGB 컬러 이미지

## 설치

```bash
# CLI 도구 빌드
cargo build --release --bin ffmpeg-rs

# GUI 도구 빌드
cargo build --release --bin ffmpeg-rs-gui

# 또는 둘 다 빌드
cargo build --release
```

바이너리 위치:
- CLI: `target/release/ffmpeg-rs` (~530 KB)
- GUI: `target/release/ffmpeg-rs-gui` (~18 MB)

## 사용법

### GUI 사용법

GUI 애플리케이션 실행:

```bash
./target/release/ffmpeg-rs-gui
```

GUI 기능:
- **오디오 탭**: WAV 파일 처리
  - 파일 선택 대화상자
  - 파일 정보 표시
  - 볼륨 조절 슬라이더 (0.0 ~ 2.0)
  - 스테레오 → 모노 변환 체크박스
  - 출력 파일 설정

- **이미지 탭**: BMP 파일 처리
  - 파일 선택 대화상자
  - 파일 정보 표시
  - 리사이징 (폭/높이 설정)
  - 그레이스케일 변환 체크박스
  - 출력 파일 설정

### CLI 사용법

#### 기본 명령어

```bash
# 도움말 보기
./target/release/ffmpeg-rs --help

# 미디어 파일 정보 보기
./target/release/ffmpeg-rs info <file>
```

### 오디오 처리

```bash
# 볼륨 조절 (0.0 ~ 2.0, 1.0 = 원본)
./target/release/ffmpeg-rs audio-volume input.wav output.wav 1.5

# 스테레오를 모노로 변환
./target/release/ffmpeg-rs audio-mono input.wav output.wav
```

### 이미지 처리

```bash
# 이미지 리사이징
./target/release/ffmpeg-rs image-resize input.bmp output.bmp 800 600

# 그레이스케일 변환
./target/release/ffmpeg-rs image-gray input.bmp output.bmp
```

## 예제

```bash
# WAV 파일 정보 확인
$ ./target/release/ffmpeg-rs info test.wav
WAV File Information:
  Format: PCM
  Channels: 2
  Sample Rate: 44100 Hz
  Bit Depth: 16 bits
  Byte Rate: 176400 bytes/sec
  Duration: 3.45 seconds
  Data Size: 608400 bytes

# 볼륨 50% 줄이기
$ ./target/release/ffmpeg-rs audio-volume music.wav quiet.wav 0.5

# 이미지를 640x480으로 리사이징
$ ./target/release/ffmpeg-rs image-resize photo.bmp resized.bmp 640 480
```

## 프로젝트 구조

```
ffmpeg-implement/
├── src/
│   ├── lib.rs            # 라이브러리 루트 (wav, bmp 모듈 export)
│   ├── main.rs           # CLI 엔트리포인트
│   ├── gui_main.rs       # GUI 엔트리포인트 (egui)
│   ├── wav/              # WAV 오디오 모듈
│   │   ├── mod.rs
│   │   ├── parser.rs     # WAV 파일 파싱 (순수 Rust)
│   │   └── processor.rs  # 오디오 처리 (볼륨, 모노 변환)
│   └── bmp/              # BMP 이미지 모듈
│       ├── mod.rs
│       ├── parser.rs     # BMP 파일 파싱 (순수 Rust)
│       └── processor.rs  # 이미지 처리 (리사이징, 그레이스케일)
├── examples/
│   └── generate_test_files.rs  # 테스트 파일 생성기
└── Cargo.toml
```

### 의존성

**코덱 구현** (wav, bmp 모듈):
- 외부 의존성 없음, Rust 표준 라이브러리만 사용

**GUI**:
- `eframe` 0.28 - egui 프레임워크
- `egui` 0.28 - 즉시 모드 GUI 라이브러리
- `rfd` 0.14 - 파일 선택 대화상자

## 기술 상세

### 코덱의 난이도

이 프로젝트는 **간단한 코덱**부터 시작했습니다:

**쉬운 코덱 (구현됨):**
- **PCM (WAV)**: 압축 없음, 바이트를 직접 읽고 처리
- **BMP**: 압축 없는 비트맵, 픽셀 데이터를 직접 조작

**어려운 코덱 (미구현):**
- **H.264/AVC**: DCT 변환, 움직임 예측, 엔트로피 코딩 필요
- **AAC/MP3**: 심리음향 모델, 복잡한 수학 변환 필요
- **JPEG**: DCT, 허프만 코딩, 양자화 필요

### WAV 파일 구조

```
RIFF Header (12 bytes)
  "RIFF" + 파일크기 + "WAVE"

fmt Chunk (24 bytes for PCM)
  포맷정보 (채널, 샘플레이트, 비트뎁스 등)

data Chunk
  실제 오디오 데이터 (PCM 샘플)
```

### BMP 파일 구조

```
File Header (14 bytes)
  "BM" + 파일크기 + 데이터오프셋

DIB Header (40 bytes for BITMAPINFOHEADER)
  이미지 정보 (너비, 높이, 비트뎁스 등)

Pixel Data
  BGR 순서의 픽셀 데이터 (4바이트 정렬)
```

## 제한사항

- 압축된 포맷은 지원하지 않음 (MP3, JPEG, H.264 등)
- WAV는 PCM 8/16-bit만 지원
- BMP는 24-bit, 32-bit만 지원
- 이미지 리사이징은 최근접 이웃 알고리즘 사용 (간단하지만 품질이 낮음)

## 향후 개선 가능 항목

1. **더 나은 리샘플링 알고리즘**
   - Bilinear, Bicubic interpolation

2. **더 많은 오디오 처리**
   - 페이드 인/아웃
   - 샘플레이트 변환
   - 오디오 믹싱

3. **더 많은 이미지 처리**
   - 회전, 뒤집기
   - 밝기/대비 조절
   - 필터 적용

4. **간단한 압축 코덱**
   - RLE (Run-Length Encoding)
   - ADPCM (오디오)

5. **GUI 개선**
   - 실시간 오디오 파형 미리보기
   - 이미지 미리보기
   - 배치 처리 기능
   - 진행률 표시

## 라이선스

MIT

## 기여

이 프로젝트는 교육 목적으로 만들어졌습니다.
실제 프로덕션에서는 ffmpeg나 다른 검증된 라이브러리를 사용하세요.
