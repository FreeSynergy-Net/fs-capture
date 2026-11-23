//! ZIP bundling: only the files named in the list, relative names, fixed timestamps.

use std::fs;
use std::io::{Cursor, Write};
use std::path::Path;

use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, DateTime, ZipWriter};

use crate::error::{CaptureError, ErrorCode, Result};
use crate::index::{format_index, Entry, INDEX_FILE};
use crate::naming::is_session_file_name;

fn zip_err(e: zip::result::ZipError) -> CaptureError {
    CaptureError::new(ErrorCode::Zip, e.to_string())
}

/// Builds the ZIP in memory. Same session content gives identical bytes:
/// the list comes first, then the PNGs in order of their numbers; every entry
/// has the timestamp 1980-01-01 and mode 0644. Symlinks are refused.
pub fn bundle_zip(dir: &Path, entries: &[Entry]) -> Result<Vec<u8>> {
    let options = SimpleFileOptions::default()
        .compression_method(CompressionMethod::Deflated)
        .last_modified_time(DateTime::default())
        .unix_permissions(0o644);
    let mut zip = ZipWriter::new(Cursor::new(Vec::new()));

    zip.start_file(INDEX_FILE, options).map_err(zip_err)?;
    zip.write_all(format_index(entries).as_bytes())?;

    let mut sorted: Vec<&Entry> = entries.iter().collect();
    sorted.sort_by_key(|e| e.nr);
    for e in sorted {
        if !is_session_file_name(&e.file) {
            return Err(CaptureError::new(
                ErrorCode::BadIndex,
                format!("unsafe file name in list: {:?}", e.file),
            ));
        }
        let path = dir.join(&e.file);
        let meta = fs::symlink_metadata(&path)?;
        if !meta.file_type().is_file() {
            return Err(CaptureError::new(
                ErrorCode::Io,
                format!("{} is not a regular file", e.file),
            ));
        }
        zip.start_file(e.file.as_str(), options).map_err(zip_err)?;
        zip.write_all(&fs::read(&path)?)?;
    }
    Ok(zip.finish().map_err(zip_err)?.into_inner())
}
