use eframe::egui;
use ffmpeg_rs::{WavFile, WavProcessor, BmpFile, BmpProcessor};
use std::path::PathBuf;

fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([900.0, 600.0])
            .with_min_inner_size([600.0, 400.0]),
        ..Default::default()
    };

    eframe::run_native(
        "FFmpeg-RS - Media Processor",
        options,
        Box::new(|_cc| Ok(Box::new(MediaProcessorApp::default()))),
    )
}

#[derive(Default)]
struct MediaProcessorApp {
    active_tab: Tab,
    audio_state: AudioState,
    image_state: ImageState,
}

#[derive(Default, PartialEq)]
enum Tab {
    #[default]
    Audio,
    Image,
}

struct AudioState {
    input_path: Option<PathBuf>,
    output_path: String,
    wav_info: Option<WavInfo>,
    volume: f32,
    convert_to_mono: bool,
    processing: bool,
    status_message: String,
}

#[derive(Clone)]
struct WavInfo {
    format: String,
    channels: u16,
    sample_rate: u32,
    bits_per_sample: u16,
    duration: f64,
    data_size: u32,
}

struct ImageState {
    input_path: Option<PathBuf>,
    output_path: String,
    bmp_info: Option<BmpInfo>,
    resize_enabled: bool,
    new_width: u32,
    new_height: u32,
    grayscale_enabled: bool,
    processing: bool,
    status_message: String,
}

#[derive(Clone)]
struct BmpInfo {
    width: i32,
    height: i32,
    bits_per_pixel: u16,
    image_size: usize,
}

impl MediaProcessorApp {
    fn render_audio_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("Audio Processing (WAV)");
        ui.separator();

        // File Selection
        ui.horizontal(|ui| {
            ui.label("Input File:");
            if ui.button("📁 Select WAV File").clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("WAV Audio", &["wav"])
                    .pick_file()
                {
                    self.load_audio_file(path);
                }
            }
            if let Some(path) = &self.audio_state.input_path {
                ui.label(path.display().to_string());
            }
        });

        ui.add_space(10.0);

        // File Info
        if let Some(info) = &self.audio_state.wav_info {
            egui::Grid::new("audio_info_grid")
                .num_columns(2)
                .spacing([40.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    ui.label("Format:");
                    ui.label(&info.format);
                    ui.end_row();

                    ui.label("Channels:");
                    ui.label(format!("{}", info.channels));
                    ui.end_row();

                    ui.label("Sample Rate:");
                    ui.label(format!("{} Hz", info.sample_rate));
                    ui.end_row();

                    ui.label("Bit Depth:");
                    ui.label(format!("{} bits", info.bits_per_sample));
                    ui.end_row();

                    ui.label("Duration:");
                    ui.label(format!("{:.2} seconds", info.duration));
                    ui.end_row();

                    ui.label("Data Size:");
                    ui.label(format!("{} bytes", info.data_size));
                    ui.end_row();
                });
        }

        ui.add_space(20.0);
        ui.separator();

        // Processing Options
        ui.heading("Processing Options");
        ui.add_space(10.0);

        ui.horizontal(|ui| {
            ui.label("Volume Adjustment:");
            ui.add(egui::Slider::new(&mut self.audio_state.volume, 0.0..=2.0)
                .text("Volume")
                .step_by(0.1));
            ui.label(format!("{:.1}x", self.audio_state.volume));
        });

        ui.checkbox(&mut self.audio_state.convert_to_mono, "Convert to Mono (Stereo → Mono)");

        ui.add_space(10.0);

        ui.horizontal(|ui| {
            ui.label("Output File:");
            ui.text_edit_singleline(&mut self.audio_state.output_path);
            if ui.button("📁").clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("WAV Audio", &["wav"])
                    .save_file()
                {
                    self.audio_state.output_path = path.display().to_string();
                }
            }
        });

        ui.add_space(20.0);

        // Process Button
        let can_process = self.audio_state.input_path.is_some()
            && !self.audio_state.output_path.is_empty()
            && !self.audio_state.processing;

        if ui.add_enabled(can_process, egui::Button::new("🎵 Process Audio"))
            .clicked()
        {
            self.process_audio();
        }

        // Status Message
        if !self.audio_state.status_message.is_empty() {
            ui.add_space(10.0);
            ui.colored_label(
                if self.audio_state.status_message.starts_with("Error") {
                    egui::Color32::RED
                } else {
                    egui::Color32::GREEN
                },
                &self.audio_state.status_message,
            );
        }
    }

    fn render_image_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("Image Processing (BMP)");
        ui.separator();

        // File Selection
        ui.horizontal(|ui| {
            ui.label("Input File:");
            if ui.button("📁 Select BMP File").clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("BMP Image", &["bmp"])
                    .pick_file()
                {
                    self.load_image_file(path);
                }
            }
            if let Some(path) = &self.image_state.input_path {
                ui.label(path.display().to_string());
            }
        });

        ui.add_space(10.0);

        // File Info
        if let Some(info) = &self.image_state.bmp_info {
            egui::Grid::new("image_info_grid")
                .num_columns(2)
                .spacing([40.0, 4.0])
                .striped(true)
                .show(ui, |ui| {
                    ui.label("Width:");
                    ui.label(format!("{} pixels", info.width));
                    ui.end_row();

                    ui.label("Height:");
                    ui.label(format!("{} pixels", info.height));
                    ui.end_row();

                    ui.label("Bits per Pixel:");
                    ui.label(format!("{}", info.bits_per_pixel));
                    ui.end_row();

                    ui.label("Image Size:");
                    ui.label(format!("{} bytes", info.image_size));
                    ui.end_row();
                });
        }

        ui.add_space(20.0);
        ui.separator();

        // Processing Options
        ui.heading("Processing Options");
        ui.add_space(10.0);

        ui.checkbox(&mut self.image_state.resize_enabled, "Resize Image");
        if self.image_state.resize_enabled {
            ui.horizontal(|ui| {
                ui.label("Width:");
                ui.add(egui::DragValue::new(&mut self.image_state.new_width)
                    .speed(1)
                    .range(1..=4096));
                ui.label("Height:");
                ui.add(egui::DragValue::new(&mut self.image_state.new_height)
                    .speed(1)
                    .range(1..=4096));
            });
        }

        ui.checkbox(&mut self.image_state.grayscale_enabled, "Convert to Grayscale");

        ui.add_space(10.0);

        ui.horizontal(|ui| {
            ui.label("Output File:");
            ui.text_edit_singleline(&mut self.image_state.output_path);
            if ui.button("📁").clicked() {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("BMP Image", &["bmp"])
                    .save_file()
                {
                    self.image_state.output_path = path.display().to_string();
                }
            }
        });

        ui.add_space(20.0);

        // Process Button
        let can_process = self.image_state.input_path.is_some()
            && !self.image_state.output_path.is_empty()
            && !self.image_state.processing
            && (self.image_state.resize_enabled || self.image_state.grayscale_enabled);

        if ui.add_enabled(can_process, egui::Button::new("🖼️ Process Image"))
            .clicked()
        {
            self.process_image();
        }

        // Status Message
        if !self.image_state.status_message.is_empty() {
            ui.add_space(10.0);
            ui.colored_label(
                if self.image_state.status_message.starts_with("Error") {
                    egui::Color32::RED
                } else {
                    egui::Color32::GREEN
                },
                &self.image_state.status_message,
            );
        }
    }

    fn load_audio_file(&mut self, path: PathBuf) {
        match WavFile::from_file(&path.display().to_string()) {
            Ok(wav) => {
                self.audio_state.wav_info = Some(WavInfo {
                    format: if wav.header.audio_format == 1 {
                        "PCM".to_string()
                    } else {
                        "Unknown".to_string()
                    },
                    channels: wav.header.num_channels,
                    sample_rate: wav.header.sample_rate,
                    bits_per_sample: wav.header.bits_per_sample,
                    duration: wav.duration_secs(),
                    data_size: wav.data_size,
                });
                self.audio_state.input_path = Some(path.clone());

                // Set default output path
                let mut output = path.clone();
                output.set_file_name(format!(
                    "{}_processed.wav",
                    path.file_stem().unwrap().to_string_lossy()
                ));
                self.audio_state.output_path = output.display().to_string();
                self.audio_state.status_message.clear();
            }
            Err(e) => {
                self.audio_state.status_message = format!("Error loading file: {}", e);
                self.audio_state.wav_info = None;
            }
        }
    }

    fn load_image_file(&mut self, path: PathBuf) {
        match BmpFile::from_file(&path.display().to_string()) {
            Ok(bmp) => {
                self.image_state.bmp_info = Some(BmpInfo {
                    width: bmp.header.width,
                    height: bmp.header.height.abs(),
                    bits_per_pixel: bmp.header.bits_per_pixel,
                    image_size: bmp.data.len(),
                });

                // Set default resize dimensions
                self.image_state.new_width = (bmp.header.width / 2).max(1) as u32;
                self.image_state.new_height = (bmp.header.height.abs() / 2).max(1) as u32;

                self.image_state.input_path = Some(path.clone());

                // Set default output path
                let mut output = path.clone();
                output.set_file_name(format!(
                    "{}_processed.bmp",
                    path.file_stem().unwrap().to_string_lossy()
                ));
                self.image_state.output_path = output.display().to_string();
                self.image_state.status_message.clear();
            }
            Err(e) => {
                self.image_state.status_message = format!("Error loading file: {}", e);
                self.image_state.bmp_info = None;
            }
        }
    }

    fn process_audio(&mut self) {
        self.audio_state.processing = true;
        self.audio_state.status_message = "Processing...".to_string();

        let input_path = self.audio_state.input_path.as_ref().unwrap().display().to_string();
        let output_path = self.audio_state.output_path.clone();
        let volume = self.audio_state.volume;
        let convert_to_mono = self.audio_state.convert_to_mono;

        match WavFile::from_file(&input_path) {
            Ok(mut wav) => {
                let mut result = Ok(());

                // Apply volume adjustment
                if (volume - 1.0).abs() > 0.01 {
                    result = WavProcessor::adjust_volume(&mut wav, volume);
                }

                // Apply mono conversion
                if convert_to_mono && result.is_ok() {
                    result = WavProcessor::stereo_to_mono(&mut wav);
                }

                // Save result
                if result.is_ok() {
                    result = WavProcessor::save(&wav, &output_path);
                }

                match result {
                    Ok(_) => {
                        self.audio_state.status_message =
                            format!("✓ Successfully processed to: {}", output_path);
                    }
                    Err(e) => {
                        self.audio_state.status_message = format!("Error processing: {}", e);
                    }
                }
            }
            Err(e) => {
                self.audio_state.status_message = format!("Error loading file: {}", e);
            }
        }

        self.audio_state.processing = false;
    }

    fn process_image(&mut self) {
        self.image_state.processing = true;
        self.image_state.status_message = "Processing...".to_string();

        let input_path = self.image_state.input_path.as_ref().unwrap().display().to_string();
        let output_path = self.image_state.output_path.clone();

        match BmpFile::from_file(&input_path) {
            Ok(bmp) => {
                let mut processed = bmp;
                let mut result = Ok(());

                // Apply resize
                if self.image_state.resize_enabled {
                    match BmpProcessor::resize(
                        &processed,
                        self.image_state.new_width,
                        self.image_state.new_height,
                    ) {
                        Ok(resized) => processed = resized,
                        Err(e) => result = Err(e),
                    }
                }

                // Apply grayscale
                if self.image_state.grayscale_enabled && result.is_ok() {
                    match BmpProcessor::to_grayscale(&processed) {
                        Ok(gray) => processed = gray,
                        Err(e) => result = Err(e),
                    }
                }

                // Save result
                if result.is_ok() {
                    result = BmpProcessor::save(&processed, &output_path);
                }

                match result {
                    Ok(_) => {
                        self.image_state.status_message =
                            format!("✓ Successfully processed to: {}", output_path);
                    }
                    Err(e) => {
                        self.image_state.status_message = format!("Error processing: {}", e);
                    }
                }
            }
            Err(e) => {
                self.image_state.status_message = format!("Error loading file: {}", e);
            }
        }

        self.image_state.processing = false;
    }
}

impl eframe::App for MediaProcessorApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("🎬 FFmpeg-RS Media Processor");
            ui.label("Simple media processing tool built with Rust");
            ui.add_space(10.0);

            // Tab selection
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.active_tab, Tab::Audio, "🎵 Audio");
                ui.selectable_value(&mut self.active_tab, Tab::Image, "🖼️ Image");
            });

            ui.separator();
            ui.add_space(10.0);

            // Render active tab
            egui::ScrollArea::vertical().show(ui, |ui| {
                match self.active_tab {
                    Tab::Audio => self.render_audio_tab(ui),
                    Tab::Image => self.render_image_tab(ui),
                }
            });
        });
    }
}

impl Default for AudioState {
    fn default() -> Self {
        Self {
            input_path: None,
            output_path: String::new(),
            wav_info: None,
            volume: 1.0,
            convert_to_mono: false,
            processing: false,
            status_message: String::new(),
        }
    }
}

impl Default for ImageState {
    fn default() -> Self {
        Self {
            input_path: None,
            output_path: String::new(),
            bmp_info: None,
            resize_enabled: false,
            new_width: 800,
            new_height: 600,
            grayscale_enabled: false,
            processing: false,
            status_message: String::new(),
        }
    }
}
