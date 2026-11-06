//! Stand-in for the Vision recognizer (`native/ocr.swift`) on hosts
//! without Apple Vision. Reads one image with tesseract and prints the
//! same JSON contract, so the daemon above it cannot tell which host
//! it runs on:
//!
//!     {"text": str, "confidence": float, "x": px, "y": px,
//!      "width": px, "height": px}
//!
//! Coordinates are pixels of the image that was handed in, with the
//! origin at the top left, matching the Swift output.
//!
//! Environment:
//!     POKI_OCR_LANGS   tesseract language list
//!                      (default "chi_sim+eng+jpn")
//!     POKI_OCR_PSM     tesseract page segmentation mode (default
//!                      "11", sparse text: a screen is scattered
//!                      labels, not a page of prose)
//!     POKI_OCR_UPSCALE integer upscale factor applied to narrow
//!                      images before recognition (default "2";
//!                      boxes are divided back afterwards)

use serde::Serialize;
use std::env;
use std::fs;
use std::io::ErrorKind;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

const MIN_WIDTH: u32 = 700;
const TIMEOUT: Duration = Duration::from_secs(40);

#[derive(Serialize)]
struct Line {
    text: String,
    confidence: f64,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

struct Word {
    text: String,
    confidence: f64,
    left: i64,
    top: i64,
    width: i64,
    height: i64,
}

fn env_or(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}

fn main() {
    std::process::exit(run());
}

fn run() -> i32 {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Expected a readable image path");
        return 1;
    }
    match recognize(&args[1]) {
        Ok(lines) => {
            println!("{}", serde_json::to_string(&lines).unwrap());
            0
        }
        Err(message) => {
            eprintln!("OCR failed: {message}");
            1
        }
    }
}

/// Upscale a narrow capture so small glyphs survive recognition.
/// Returns the path to read and the factor its boxes must be divided
/// by; any image problem falls back to the original file.
fn enlarge(path: &str, upscale: u32) -> (String, f64) {
    if upscale <= 1 {
        return (path.to_string(), 1.0);
    }
    let Ok(image) = image::open(path) else {
        return (path.to_string(), 1.0);
    };
    if image.width() >= MIN_WIDTH {
        return (path.to_string(), 1.0);
    }
    let big = image.resize(
        image.width() * upscale,
        image.height() * upscale,
        image::imageops::FilterType::Lanczos3,
    );
    let out = env::temp_dir().join(format!("poki-ocr-{}-big.png", std::process::id()));
    match big.save(&out) {
        Ok(()) => (out.to_string_lossy().into_owned(), upscale as f64),
        Err(_) => (path.to_string(), 1.0),
    }
}

fn tesseract_tsv(path: &str, langs: &str, psm: &str) -> Result<String, String> {
    let stamp = std::process::id();
    let out_file = env::temp_dir().join(format!("poki-ocr-{stamp}.tsv"));
    let err_file = env::temp_dir().join(format!("poki-ocr-{stamp}.err"));
    let stdout = fs::File::create(&out_file).map_err(|e| e.to_string())?;
    let stderr = fs::File::create(&err_file).map_err(|e| e.to_string())?;
    let mut child = Command::new("tesseract")
        .args([path, "stdout", "-l", langs, "--psm", psm, "tsv"])
        .env("OMP_THREAD_LIMIT", "1")
        .stdout(Stdio::from(stdout))
        .stderr(Stdio::from(stderr))
        .spawn()
        .map_err(|e| match e.kind() {
            ErrorKind::NotFound => "tesseract is not installed on this host".to_string(),
            _ => e.to_string(),
        });
    let result = (|| {
        let child = child.as_mut().map_err(|e| e.to_string())?;
        let started = Instant::now();
        loop {
            match child.try_wait().map_err(|e| e.to_string())? {
                Some(status) if status.success() => {
                    return fs::read_to_string(&out_file).map_err(|e| e.to_string());
                }
                Some(_) => {
                    let raw = fs::read_to_string(&err_file).unwrap_or_default();
                    let tail: String = raw
                        .chars()
                        .rev()
                        .take(500)
                        .collect::<String>()
                        .chars()
                        .rev()
                        .collect();
                    let tail = tail.trim();
                    return Err(if tail.is_empty() {
                        "tesseract failed".to_string()
                    } else {
                        tail.to_string()
                    });
                }
                None => {
                    if started.elapsed() > TIMEOUT {
                        let _ = child.kill();
                        return Err("tesseract timed out".to_string());
                    }
                    std::thread::sleep(Duration::from_millis(50));
                }
            }
        }
    })();
    let _ = fs::remove_file(&out_file);
    let _ = fs::remove_file(&err_file);
    result
}

fn is_cjk(character: char) -> bool {
    ('\u{3000}'..='\u{9fff}').contains(&character)
        || ('\u{ff00}'..='\u{ffef}').contains(&character)
}

/// Group tesseract word rows into the text lines a screen shows.
fn collect(tsv: &str, factor: f64) -> Vec<Line> {
    let mut groups: Vec<((String, String, String), Vec<Word>)> = Vec::new();
    for row in tsv.lines().skip(1) {
        let field: Vec<&str> = row.split('\t').collect();
        if field.len() < 12 || field[0] != "5" {
            continue;
        }
        let text = field[11].trim();
        if text.is_empty() {
            continue;
        }
        let (Ok(confidence), Ok(left), Ok(top), Ok(width), Ok(height)) = (
            field[10].parse::<f64>(),
            field[6].parse::<i64>(),
            field[7].parse::<i64>(),
            field[8].parse::<i64>(),
            field[9].parse::<i64>(),
        ) else {
            continue;
        };
        if confidence < 0.0 || width <= 0 || height <= 0 {
            continue;
        }
        let key = (
            field[2].to_string(),
            field[3].to_string(),
            field[4].to_string(),
        );
        let word = Word {
            text: text.to_string(),
            confidence,
            left,
            top,
            width,
            height,
        };
        match groups.iter_mut().find(|(k, _)| *k == key) {
            Some((_, words)) => words.push(word),
            None => groups.push((key, vec![word])),
        }
    }

    let mut lines: Vec<Line> = groups
        .into_iter()
        .map(|(_, mut words)| {
            words.sort_by_key(|word| word.left);
            let mut text = String::new();
            for word in &words {
                // Chinese and Japanese do not space between words;
                // Latin scripts do.
                let boundary = match (text.chars().last(), word.text.chars().next()) {
                    (Some(last), Some(first)) => !(is_cjk(last) && is_cjk(first)),
                    _ => false,
                };
                if !text.is_empty() && boundary {
                    text.push(' ');
                }
                text.push_str(&word.text);
            }
            let left = words.iter().map(|w| w.left).min().unwrap();
            let top = words.iter().map(|w| w.top).min().unwrap();
            let right = words.iter().map(|w| w.left + w.width).max().unwrap();
            let bottom = words.iter().map(|w| w.top + w.height).max().unwrap();
            let confidence =
                words.iter().map(|w| w.confidence).sum::<f64>() / words.len() as f64 / 100.0;
            Line {
                text,
                confidence: (confidence * 10_000.0).round() / 10_000.0,
                x: left as f64 / factor,
                y: top as f64 / factor,
                width: (right - left) as f64 / factor,
                height: (bottom - top) as f64 / factor,
            }
        })
        .collect();
    lines.sort_by(|a, b| {
        a.y.partial_cmp(&b.y)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(a.x.partial_cmp(&b.x).unwrap_or(std::cmp::Ordering::Equal))
    });
    lines
}

fn recognize(path: &str) -> Result<Vec<Line>, String> {
    let langs = env_or("POKI_OCR_LANGS", "chi_sim+eng+jpn");
    let psm = env_or("POKI_OCR_PSM", "11");
    let upscale: u32 = env_or("POKI_OCR_UPSCALE", "2").parse().unwrap_or(2);
    let (read_path, factor) = enlarge(path, upscale);
    let result = tesseract_tsv(&read_path, &langs, &psm).map(|tsv| collect(&tsv, factor));
    if read_path != path {
        let _ = fs::remove_file(&read_path);
    }
    result
}
