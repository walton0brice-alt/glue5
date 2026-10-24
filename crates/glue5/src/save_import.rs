//! Import a story save from Files without replacing any existing save.
use std::{fs, io::{self, Write}, path::{Path, PathBuf}};

pub fn import(source: &Path, profile: &Path) -> io::Result<Option<PathBuf>> {
    if !source.is_file() { return Ok(None); }
    let bytes = fs::read(source)?;
    if !(65536..=2097152).contains(&bytes.len()) || !bytes.starts_with(&[0, 0, 0, 1]) {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "Not a supported story save"));
    }
    fs::create_dir_all(profile)?;
    // Prefer its original slot; otherwise take an empty manual slot, never autosave.
    for slot in [1, 0, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14] {
        let dest = profile.join(format!("SGTA5{slot:04}"));
        if dest.is_file() && fs::read(&dest)? == bytes {
            fs::rename(source, source.with_extension("imported"))?;
            return Ok(Some(dest));
        }
        let mut file = match fs::OpenOptions::new().write(true).create_new(true).open(&dest) {
            Ok(file) => file,
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(e),
        };
        file.write_all(&bytes)?;
        file.sync_all()?;
        if fs::read(&dest)? != bytes {
            return Err(io::Error::new(io::ErrorKind::InvalidData, "Save copy verification failed"));
        }
        // Retain the supplied file as a visible backup and prevent repeated imports.
        fs::rename(source, source.with_extension("imported"))?;
        return Ok(Some(dest));
    }
    Err(io::Error::new(io::ErrorKind::AlreadyExists, "All manual save slots are occupied"))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn preserves_existing_and_imports_once() {
        let root = std::env::temp_dir().join(format!("glue5-save-test-{}", std::process::id()));
        fs::create_dir_all(&root).unwrap();
        let source = root.join("SGTA50001");
        let profile = root.join("profile");
        fs::create_dir_all(&profile).unwrap();
        fs::write(profile.join("SGTA50001"), b"existing progress").unwrap();
        let mut bytes = vec![42; 533192]; bytes[..4].copy_from_slice(&[0,0,0,1]);
        fs::write(&source, &bytes).unwrap();
        let dest = import(&source, &profile).unwrap().unwrap();
        assert_eq!(dest.file_name().unwrap(), "SGTA50000");
        assert_eq!(fs::read(&dest).unwrap(), bytes);
        assert_eq!(fs::read(profile.join("SGTA50001")).unwrap(), b"existing progress");
        assert_eq!(fs::read(source.with_extension("imported")).unwrap(), bytes);
        assert!(import(&source, &profile).unwrap().is_none());
        fs::write(&source, b"invalid").unwrap();
        assert!(import(&source, &profile).is_err());
        fs::remove_dir_all(root).unwrap();
    }
}
