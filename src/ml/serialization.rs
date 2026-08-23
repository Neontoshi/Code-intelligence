// src/ml/serialization.rs

use crate::error::{CodeIntelError, Result};
use serde::{de::DeserializeOwned, Serialize};
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;

const BINARY_MAGIC: &[u8; 4] = b"CIMD";
const BINARY_VERSION: u8 = 1;

pub struct ModelSerializer;

impl ModelSerializer {
    /// Save model to disk using compact Bincode serialization with header magic
    pub fn save_binary<T: Serialize, P: AsRef<Path>>(data: &T, path: P) -> Result<()> {
        let file = File::create(path.as_ref())?;
        let mut writer = BufWriter::new(file);

        // Write 4-byte magic + 1-byte version
        writer.write_all(BINARY_MAGIC)?;
        writer.write_all(&[BINARY_VERSION])?;

        bincode::serialize_into(&mut writer, data).map_err(|e| CodeIntelError::ModelError {
            message: format!("Failed to serialize binary model: {}", e),
        })?;

        writer.flush()?;
        Ok(())
    }

    /// Load model from disk, automatically detecting Bincode vs. legacy JSON
    pub fn load_auto<T: DeserializeOwned, P: AsRef<Path>>(path: P) -> Result<T> {
        let path_ref = path.as_ref();
        let file = File::open(path_ref).map_err(|_e| CodeIntelError::ModelNotFound {
            path: path_ref.to_path_buf(),
        })?;

        let mut reader = BufReader::new(file);

        // Peek first 4 bytes to check for binary magic
        let mut header = [0u8; 4];
        if reader.read_exact(&mut header).is_ok() && &header == BINARY_MAGIC {
            let mut version = [0u8; 1];
            reader.read_exact(&mut version)?;

            if version[0] != BINARY_VERSION {
                return Err(CodeIntelError::ModelVersionMismatch {
                    message: format!(
                        "Unsupported binary model version: got {}, expected {}",
                        version[0], BINARY_VERSION
                    ),
                });
            }

            bincode::deserialize_from(reader).map_err(|e| CodeIntelError::ModelError {
                message: format!("Failed to deserialize Bincode model: {}", e),
            })
        } else {
            // Fallback: Read full file as legacy JSON format
            let raw_file = File::open(path_ref)?;
            serde_json::from_reader(BufReader::new(raw_file)).map_err(|e| {
                CodeIntelError::DeserializationError {
                    message: format!(
                        "Model at {:?} is neither valid Bincode nor valid JSON: {}",
                        path_ref, e
                    ),
                }
            })
        }
    }
}
