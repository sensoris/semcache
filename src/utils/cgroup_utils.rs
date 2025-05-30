use std::{fs, path::Path};

// Reads the current memory usage (in bytes) from the cgroup v2 file
pub fn read_cgroup_v2_memory_bytes() -> Option<u64> {
    // Cgroup v2 unified path (works inside most containers & modern Linux)
    let path = Path::new("/sys/fs/cgroup/memory.current");

    // Read and parse
    let contents = fs::read_to_string(path).ok()?;
    contents.trim().parse::<u64>().ok()
}

/// Convert to KB
pub fn read_cgroup_v2_memory_kb() -> Option<u64> {
    read_cgroup_v2_memory_bytes().map(|bytes| bytes / 1024)
}
