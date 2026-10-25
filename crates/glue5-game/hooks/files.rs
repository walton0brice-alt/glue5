// Hooks for the runtime's `files.rs` (see ../hooks.rs).
&[
    ("story_save_path", "rage::GetSaveFilePath(unsigned int, char (&) [256], char const*)"),
    // The names of files in memory, which carry their buffer's address.
    ("grow_buffer_file_name", "rage::fiDevice::MakeGrowBufferFileName(char*, int, rage::datGrowBuffer*)"),
    ("memory_file_name", "rage::fiDevice::MakeMemoryFileName(char*, int, void const*, unsigned long, bool, char const*)"),
]
