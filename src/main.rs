#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use std::any::Any;

mod dir_watcher;
mod file_blobs;

fn dir_watcher_main() -> anyhow::Result<()> {
    let mut watched = dir_watcher::dir_watcher::new("/dev/shm".to_string())?;
    loop {
        for i in watched.get_files(std::time::Duration::from_millis(100), 16, 1)? {
            eprintln!("Found file {}", i.to_str().unwrap_or("NO PATH FOUND"));
        }
    }
    return Ok(());
}

fn main() -> anyhow::Result<()> {
    let res = file_blobs::metadata_file::open("./video.mp4", 16)?;
    let tmp = res.write_file_to_prefix(
        /*path_dir_prefix: &std::path::Path =*/ "/home/asd/dest",
    )?;
    tmp.iter().for_each(|i| {
        println!(
            "{}: {}",
            i.hash.to_hex().as_str(),
            i.size.to_string().as_str()
        );
    });

    file_blobs::write_table_2_file(
        /*tables_input: Vec<metadata_chunk> =*/ &tmp,
        /*path_dir_prefix_input: impl AsRef<std::path::Path> =*/ "/home/asd/dest",
        /*path_file_output: impl AsRef<std::path::Path> =*/ "./new.mp4",
    )?;

    Ok(())
}
