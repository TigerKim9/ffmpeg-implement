mod wav;
mod bmp;

use std::env;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage();
        process::exit(1);
    }

    let command = &args[1];

    match command.as_str() {
        "info" => handle_info(&args),
        "audio-volume" => handle_audio_volume(&args),
        "audio-mono" => handle_audio_mono(&args),
        "image-resize" => handle_image_resize(&args),
        "image-gray" => handle_image_gray(&args),
        "--help" | "-h" => print_usage(),
        _ => {
            eprintln!("Unknown command: {}", command);
            print_usage();
            process::exit(1);
        }
    }
}

fn print_usage() {
    println!("ffmpeg-rs - Simple media processing tool written in Rust");
    println!();
    println!("USAGE:");
    println!("    ffmpeg-rs <COMMAND> [OPTIONS]");
    println!();
    println!("COMMANDS:");
    println!("    info <file>                           Show media file information");
    println!("    audio-volume <input> <output> <vol>   Adjust audio volume (0.0-2.0)");
    println!("    audio-mono <input> <output>           Convert stereo to mono");
    println!("    image-resize <input> <output> <w> <h> Resize image");
    println!("    image-gray <input> <output>           Convert image to grayscale");
    println!();
    println!("EXAMPLES:");
    println!("    ffmpeg-rs info audio.wav");
    println!("    ffmpeg-rs audio-volume input.wav output.wav 1.5");
    println!("    ffmpeg-rs image-resize input.bmp output.bmp 800 600");
    println!("    ffmpeg-rs image-gray input.bmp output.bmp");
}

fn handle_info(args: &[String]) {
    if args.len() < 3 {
        eprintln!("Usage: ffmpeg-rs info <file>");
        process::exit(1);
    }

    let path = &args[2];

    // 파일 확장자로 타입 판단
    if path.ends_with(".wav") || path.ends_with(".WAV") {
        match wav::WavFile::from_file(path) {
            Ok(wav) => wav.print_info(),
            Err(e) => {
                eprintln!("Error reading WAV file: {}", e);
                process::exit(1);
            }
        }
    } else if path.ends_with(".bmp") || path.ends_with(".BMP") {
        match bmp::BmpFile::from_file(path) {
            Ok(bmp) => bmp.print_info(),
            Err(e) => {
                eprintln!("Error reading BMP file: {}", e);
                process::exit(1);
            }
        }
    } else {
        eprintln!("Unsupported file format. Only WAV and BMP are supported.");
        process::exit(1);
    }
}

fn handle_audio_volume(args: &[String]) {
    if args.len() < 5 {
        eprintln!("Usage: ffmpeg-rs audio-volume <input> <output> <volume>");
        eprintln!("  volume: 0.0-2.0 (1.0 = original)");
        process::exit(1);
    }

    let input = &args[2];
    let output = &args[3];
    let volume: f32 = match args[4].parse() {
        Ok(v) => v,
        Err(_) => {
            eprintln!("Invalid volume value: {}", args[4]);
            process::exit(1);
        }
    };

    if volume < 0.0 || volume > 2.0 {
        eprintln!("Volume must be between 0.0 and 2.0");
        process::exit(1);
    }

    println!("Processing: {} -> {} (volume: {})", input, output, volume);

    let mut wav = match wav::WavFile::from_file(input) {
        Ok(w) => w,
        Err(e) => {
            eprintln!("Error reading input file: {}", e);
            process::exit(1);
        }
    };

    if let Err(e) = wav::WavProcessor::adjust_volume(&mut wav, volume) {
        eprintln!("Error adjusting volume: {}", e);
        process::exit(1);
    }

    if let Err(e) = wav::WavProcessor::save(&wav, output) {
        eprintln!("Error saving output file: {}", e);
        process::exit(1);
    }

    println!("Done!");
}

fn handle_audio_mono(args: &[String]) {
    if args.len() < 4 {
        eprintln!("Usage: ffmpeg-rs audio-mono <input> <output>");
        process::exit(1);
    }

    let input = &args[2];
    let output = &args[3];

    println!("Converting to mono: {} -> {}", input, output);

    let mut wav = match wav::WavFile::from_file(input) {
        Ok(w) => w,
        Err(e) => {
            eprintln!("Error reading input file: {}", e);
            process::exit(1);
        }
    };

    if let Err(e) = wav::WavProcessor::stereo_to_mono(&mut wav) {
        eprintln!("Error converting to mono: {}", e);
        process::exit(1);
    }

    if let Err(e) = wav::WavProcessor::save(&wav, output) {
        eprintln!("Error saving output file: {}", e);
        process::exit(1);
    }

    println!("Done!");
}

fn handle_image_resize(args: &[String]) {
    if args.len() < 6 {
        eprintln!("Usage: ffmpeg-rs image-resize <input> <output> <width> <height>");
        process::exit(1);
    }

    let input = &args[2];
    let output = &args[3];
    let width: u32 = match args[4].parse() {
        Ok(w) => w,
        Err(_) => {
            eprintln!("Invalid width: {}", args[4]);
            process::exit(1);
        }
    };
    let height: u32 = match args[5].parse() {
        Ok(h) => h,
        Err(_) => {
            eprintln!("Invalid height: {}", args[5]);
            process::exit(1);
        }
    };

    println!("Resizing: {} -> {} ({}x{})", input, output, width, height);

    let bmp = match bmp::BmpFile::from_file(input) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Error reading input file: {}", e);
            process::exit(1);
        }
    };

    let resized = match bmp::BmpProcessor::resize(&bmp, width, height) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Error resizing image: {}", e);
            process::exit(1);
        }
    };

    if let Err(e) = bmp::BmpProcessor::save(&resized, output) {
        eprintln!("Error saving output file: {}", e);
        process::exit(1);
    }

    println!("Done!");
}

fn handle_image_gray(args: &[String]) {
    if args.len() < 4 {
        eprintln!("Usage: ffmpeg-rs image-gray <input> <output>");
        process::exit(1);
    }

    let input = &args[2];
    let output = &args[3];

    println!("Converting to grayscale: {} -> {}", input, output);

    let bmp = match bmp::BmpFile::from_file(input) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Error reading input file: {}", e);
            process::exit(1);
        }
    };

    let gray = match bmp::BmpProcessor::to_grayscale(&bmp) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Error converting to grayscale: {}", e);
            process::exit(1);
        }
    };

    if let Err(e) = bmp::BmpProcessor::save(&gray, output) {
        eprintln!("Error saving output file: {}", e);
        process::exit(1);
    }

    println!("Done!");
}
