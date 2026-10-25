//! The engine's files in memory, whose names carry their buffer's address:
//! "growbuffer:<address>" (a `datGrowBuffer`, the cloud's files are read
//! into) and "memory:$<address>,<size>,<flag>:<name>" (the Creator writes a
//! job's JSON into one, other parts of the engine their data). The engine
//! formats the address with `%p`, which in this build's C library begins
//! with "0x": its devices read the address back as 0 (Windows' `%p` has no
//! prefix), and the files in memory had no buffer.

use glue5_game::Env;

use crate::memory::Mem;

/// Import at the path returned by the engine, before it enumerates its slots.
unsafe extern "C" fn story_save_path(env: *mut Env, user: u32, output: u64, filename: u64) {
    unsafe { glue5_game::orig::story_save_path(env, user, output, filename) };
    if filename != 0 {
        let name = unsafe { Mem::of(env) }.str(filename);
        if !name.is_empty() && !name.starts_with("SGTA5") { return; }
    }
    let engine_path = unsafe { Mem::of(env) }.str(output);
    let Some(directory) = crate::rt().vfs.story_save_directory(&engine_path) else { return; };
    #[cfg(target_os = "ios")]
    if let Some(root) = crate::rt().config.user_dir.parent().and_then(std::path::Path::parent).and_then(std::path::Path::parent) {
        let import_dir = root.join("Documents/SaveImport");
        if !import_dir.is_dir() { return; }
        let marker = directory.join(".glue5-story-imported");
        if marker.is_file() { return; }
        let source = [import_dir.join("SGTA50001.imported"), import_dir.join("SGTA50001")]
            .into_iter().find(|p| p.is_file());
        let Some(source) = source else { return; };
        let result = copy_story_save(&source, &directory);
        let report = match result {
            Ok(dest) => {
                let _ = std::fs::write(&marker, dest.to_string_lossy().as_bytes());
                format!("Imported story save successfully.\nEngine path: {engine_path}\nSave: {}\nIn Story Mode, open Game > Load Game and look for Father/Son.\n", dest.display())
            }
            Err(e) => format!("Save import failed: {e}\nEngine path: {engine_path}\n"),
        };
        let _ = std::fs::write(import_dir.join("import-status.txt"), report);
    }
}

#[cfg(target_os = "ios")]
fn copy_story_save(source: &std::path::Path, directory: &std::path::Path) -> std::io::Result<std::path::PathBuf> {
    use std::io::Write;
    let bytes = std::fs::read(source)?;
    if bytes.len() != 533192 || !bytes.starts_with(&[0, 0, 0, 1]) {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "Unexpected save file"));
    }
    std::fs::create_dir_all(directory)?;
    for slot in [1, 0, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14] {
        let dest = directory.join(format!("SGTA5{slot:04}"));
        if dest.is_file() && std::fs::read(&dest)? == bytes { return Ok(dest); }
        let mut file = match std::fs::OpenOptions::new().write(true).create_new(true).open(&dest) {
            Ok(file) => file,
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(e),
        };
        file.write_all(&bytes)?;
        file.sync_all()?;
        if std::fs::read(&dest)? != bytes { return Err(std::io::Error::other("Copy verification failed")); }
        return Ok(dest);
    }
    Err(std::io::Error::other("All manual save slots are occupied"))
}

/// Hook for `rage::fiDevice::MakeGrowBufferFileName(char*, int,
/// datGrowBuffer*)`.
pub(crate) unsafe extern "C" fn grow_buffer_file_name(env: *mut Env, name: u64, size: u32, buffer: u64) {
    let m = unsafe { Mem::of(env) };
    m.write_str(name, &format!("growbuffer:{buffer:x}"), size as usize);
}

/// Hook for `rage::fiDevice::MakeMemoryFileName(char*, int, void const*,
/// unsigned long, bool, char const*)`.
pub(crate) unsafe extern "C" fn memory_file_name(
    env: *mut Env,
    name: u64,
    size: u32,
    buffer: u64,
    length: u64,
    flag: u32,
    file: u64,
) {
    let m = unsafe { Mem::of(env) };
    let file = m.str(file);
    let length = length as u32 as i32;
    m.write_str(name, &format!("memory:${buffer:x},{length},{}:{file}", flag as u8), size as usize);
}

pub(crate) fn hooks(hooks: &mut glue5_game::Hooks) {
    hooks.story_save_path = Some(story_save_path);
    hooks.grow_buffer_file_name = Some(grow_buffer_file_name);
    hooks.memory_file_name = Some(memory_file_name);
}
