//! embedded_io trait implementations over the iso buffer.
//! An in-memory implementation

/// Represents an in-memory loaded file to provide I/O trait implementations.
pub struct IsoFile {
    /// In-memory file buffer.
    pub data: alloc::vec::Vec<u8>,
    /// The current seek position.
    pub seek: u64,
}

impl iso9660::io::ErrorType for IsoFile {
    type Error = iso9660::io::ErrorKind;
}

impl iso9660::io::Read for IsoFile {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        let start = self.seek as usize;
        let available = self.data.len().saturating_sub(start);
        let to_read = buf.len().min(available);

        buf[..to_read].copy_from_slice(&self.data[start..start + to_read]);
        self.seek += to_read as u64;

        Ok(to_read)
    }
}

impl iso9660::io::Seek for IsoFile {
    fn seek(&mut self, pos: iso9660::io::SeekFrom) -> Result<u64, Self::Error> {
        let new_pos = match pos {
            iso9660::io::SeekFrom::Start(offset) => offset as i64,
            iso9660::io::SeekFrom::End(offset) => self.data.len() as i64 + offset,
            iso9660::io::SeekFrom::Current(offset) => self.seek as i64 + offset,
        };

        if new_pos < 0 {
            Err(iso9660::io::ErrorKind::InvalidInput)
        } else {
            self.seek = new_pos as u64;
            Ok(self.seek)
        }
    }
}
