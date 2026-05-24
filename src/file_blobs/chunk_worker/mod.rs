use std::io::{Read, Write};

#[inline(always)]
fn construct_name(
    path_dir_prefix: impl AsRef<std::path::Path>,
    hash: &blake3::Hash,
    size: usize,
    create_dir: bool,
) -> anyhow::Result<std::path::PathBuf> {
    let res = hash.to_hex().to_string();

    let tmp = path_dir_prefix
        .as_ref()
        .join(&res[0..2])
        .join(&res[2..4])
        .join(&res);

    if create_dir {
        std::fs::create_dir_all(&tmp)?;
    }

    Ok(tmp.join(format!("{:x}", size)))
}

pub struct standalone_chunk<'a> {
    pub chunk: Vec<u8>,
    pub hash: &'a blake3::Hash,
}

pub struct metadata_chunk {
    pub hash: blake3::Hash,
    pub size: usize,
}

pub struct hashed_chunk<'a> {
    pub chunk: &'a [u8],
    pub hash: blake3::Hash,
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

impl<'a> standalone_chunk<'a> {
    fn from(
        element: &'a metadata_chunk,
        path_dir_prefix: impl AsRef<std::path::Path>,
    ) -> anyhow::Result<Self> {
        return Ok(Self {
            chunk: element.read_from_prefix(&path_dir_prefix)?,
            hash: &element.hash,
        });
    }

    fn append_to_file(&self, fd: &mut std::fs::File) -> anyhow::Result<()> {
        fd.write_all(self.chunk.as_ref())?;
        Ok(())
    }
}

fn write_table_2_file(
    tables_input: Vec<metadata_chunk>,
    path_dir_prefix_input: impl AsRef<std::path::Path>,
    path_file_output: impl AsRef<std::path::Path>,
) -> anyhow::Result<()> {
    let mut fd = std::fs::File::create(/*path =*/ path_file_output.as_ref())?;

    for i in tables_input.iter() {
        standalone_chunk::from(&i, path_dir_prefix_input.as_ref())?.append_to_file(&mut fd)?;
    }

    return Ok(());
}
