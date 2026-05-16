use std::io::{Read, Write};

#[inline(always)]
fn construct_name(
    path_dir_prefix: impl AsRef<std::path::Path>,
    hash: &blake3::Hash,
    size: usize,
    create_dir: bool,
) -> anyhow::Result<std::path::PathBuf> {
    let res = hash.to_hex().to_string();
    let tmp = path_dir_prefix.as_ref().join(&res[0..2]).join(&res[2..4]);
    if create_dir {
        std::fs::create_dir_all(&tmp)?;
    }

    Ok(tmp.join(format!("{}_{:x}", res, size)))
}

pub struct metadata_chunk {
    pub hash: blake3::Hash,
    pub size: usize,
}

pub struct hashed_chunk<'a> {
    pub chunk: &'a [u8],
    pub hash: blake3::Hash,
}

pub struct standalone_chunk<'a> {
    pub chunk: Vec<u8>,
    pub hash: &'a blake3::Hash,
}

impl metadata_chunk {
    #[inline(always)]
    pub fn read_from_prefix(
        &self,
        path_dir_prefix: impl AsRef<std::path::Path>,
    ) -> anyhow::Result<Vec<u8>> {
        Ok(std::fs::read(construct_name(
            /*path_dir_prefix: impl AsRef<std::path::Path> =*/ path_dir_prefix,
            /*hash: &blake3::Hash =*/ &self.hash,
            /*size: usize =*/ self.size,
            /*create_dir: bool =*/ false,
        )?)?)
    }
}

impl<'a> hashed_chunk<'a> {
    pub fn new(chunk: &'a [u8]) -> Self {
        let hash = blake3::hash(chunk);
        Self { chunk, hash }
    }

    pub fn write_to_destination(
        &self,
        path_dir_prefix: impl AsRef<std::path::Path>,
    ) -> anyhow::Result<metadata_chunk> {
        std::fs::File::create(construct_name(
            /*path_dir_prefix: impl AsRef<std::path::Path> =*/ &path_dir_prefix,
            /*hash: &blake3::Hash =*/ &self.hash,
            /*size: usize =*/ self.chunk.len(),
            /*create_dir: bool =*/ true,
        )?)?
        .write_all(self.chunk)?;

        Ok(metadata_chunk {
            hash: self.hash.clone(),
            size: self.chunk.len(),
        })
    }
}

impl<'a> standalone_chunk<'a> {}
