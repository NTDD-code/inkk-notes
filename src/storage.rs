use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;

use crate::model::Notebook;

pub struct NotebookStorage;

impl NotebookStorage {
    pub fn save<P: AsRef<Path>>(path: P, notebook: &Notebook) -> Result<(), String> {
        let json_data = serde_json::to_string(notebook)
            .map_err(|e| format!("Failed to serialize notebook: {}", e))?;

        let file = File::create(path).map_err(|e| format!("Failed to create file: {}", e))?;
        let mut encoder = GzEncoder::new(file, Compression::default());
        encoder
            .write_all(json_data.as_bytes())
            .map_err(|e| format!("Failed to compress data: {}", e))?;
        encoder.finish().map_err(|e| format!("Failed to finish compression: {}", e))?;

        Ok(())
    }

    pub fn load<P: AsRef<Path>>(path: P) -> Result<Notebook, String> {
        let file = File::open(&path).map_err(|e| format!("Failed to open file: {}", e))?;
        let mut decoder = GzDecoder::new(file);
        let mut contents = String::new();

        if decoder.read_to_string(&mut contents).is_ok() {
            let notebook: Notebook = serde_json::from_str(&contents)
                .map_err(|e| format!("Invalid notebook JSON: {}", e))?;
            return Ok(notebook);
        }

        // Fallback: try raw JSON
        let raw_file = File::open(path).map_err(|e| format!("Failed to open file: {}", e))?;
        let mut raw_reader = std::io::BufReader::new(raw_file);
        let mut raw_contents = String::new();
        raw_reader.read_to_string(&mut raw_contents).map_err(|e| format!("Read failed: {}", e))?;
        let notebook: Notebook = serde_json::from_str(&raw_contents)
            .map_err(|e| format!("Invalid notebook format: {}", e))?;

        Ok(notebook)
    }
}
