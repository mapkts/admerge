//! Utility functions.
use crate::error::Result;
use std::io::{Read, Seek, SeekFrom};

/// Move the internal cursor of the given stream to the start position.
pub fn seek_to_start<S: Seek>(stream: &mut S) -> Result<()> {
    stream.seek(SeekFrom::Start(0))?;
    Ok(())
}

/// Move the internal cursor of the given stream to the end position.
pub fn seek_to_end<S: Seek>(stream: &mut S) -> Result<u64> {
    stream.seek(SeekFrom::End(0)).map_err(|e| e.into())
}

/// Offset the internal cursor of the given stream relativing to the current position.
// pub fn seek_relative<S: Seek>(offset: i64, stream: &mut S) -> Result<u64> {
//     stream.seek(SeekFrom::Current(offset)).map_err(|e| e.into())
// }

/// Offset the internal cursor of the given stream relativing to the start of the stream.
pub fn seek_start<S: Seek>(offset: u64, stream: &mut S) -> Result<u64> {
    stream.seek(SeekFrom::Start(offset)).map_err(|e| e.into())
}

/// Offset the internal cursor of the given stream relativing to the end of the stream.
pub fn seek_end<S: Seek>(offset: i64, stream: &mut S) -> Result<u64> {
    stream.seek(SeekFrom::End(offset)).map_err(|e| e.into())
}

/// Returns `true` if the given stream ends with a newline.
///
/// If this function succeed, this cursor position of the given stream will restore to its original
/// position (the cursor position before calling this function).
pub fn endswith_newline<RS: Seek + Read>(stream: &mut RS) -> Result<bool> {
    let pos = stream.seek(SeekFrom::Current(0))?;
    let len = stream.seek(SeekFrom::End(0))?;
    match len {
        0 => {
            stream.seek(SeekFrom::Start(pos))?;
            Ok(false)
        }
        _ => {
            stream.seek(SeekFrom::End(-1)).unwrap();
            let mut buf = [0; 1];
            stream.read_exact(&mut buf)?;
            stream.seek(SeekFrom::Start(pos))?;
            if &buf == b"\n" {
                return Ok(true);
            }
            Ok(false)
        }
    }
}
