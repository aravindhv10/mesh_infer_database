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

pub struct metadata_chunk_info {
    pub hash: blake3::Hash,
    pub size: usize,
}

impl metadata_chunk_info {
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

pub struct metadata_chunk<'a> {
    pub chunk: &'a [u8],
    pub hash: blake3::Hash,
}

impl<'a> metadata_chunk<'a> {
    pub fn new(chunk: &'a [u8]) -> Self {
        let hash = blake3::hash(chunk);
        Self { chunk, hash }
    }

    pub fn write_to_destination(
        &self,
        path_dir_prefix: impl AsRef<std::path::Path>,
    ) -> anyhow::Result<metadata_chunk_info> {
        std::fs::File::create(construct_name(
            /*path_dir_prefix: impl AsRef<std::path::Path> =*/ &path_dir_prefix,
            /*hash: &blake3::Hash =*/ &self.hash,
            /*size: usize =*/ self.chunk.len(),
            /*create_dir: bool =*/ true,
        )?)?
        .write_all(self.chunk)?;

        Ok(metadata_chunk_info {
            hash: self.hash.clone(),
            size: self.chunk.len(),
        })
    }
}

pub struct metadata_file {
    fd: std::fs::File,
    mmap: memmap2::Mmap,
    size_piece: usize,
    n_pieces: usize,
}

impl metadata_file {
    pub fn open(
        path_file_input: impl AsRef<std::path::Path>,
        size_piece_pw: usize,
    ) -> anyhow::Result<Self> {
        let fd = std::fs::File::open(path_file_input)?;

        let size_piece = 1 << size_piece_pw;
        let size_piece_mask = size_piece - 1;

        let mmap = unsafe { memmap2::Mmap::map(&fd) }?;
        mmap.advise(memmap2::Advice::Sequential)?;

        let n_pieces = {
            let q = mmap.len() >> size_piece_pw;
            let r = mmap.len() & size_piece_mask;
            if r == 0 { q } else { q + 1 }
        };

        Ok(Self {
            fd: fd,
            mmap: mmap,
            size_piece: size_piece,
            n_pieces: n_pieces,
        })
    }

    pub fn write_file_to_prefix(
        &self,
        path_dir_prefix: impl AsRef<std::path::Path>,
    ) -> anyhow::Result<Vec<metadata_chunk_info>> {
        const read_ahead: usize = 1 << 3;
        const read_mask: usize = read_ahead - 1;

        let mut ret: Vec<metadata_chunk_info> = Vec::with_capacity(self.n_pieces);

        for idx in 0..self.n_pieces {
            let start = idx * self.size_piece;

            if (idx & read_mask) == 0 {
                let stop = (start + ((read_ahead << 1) * self.size_piece)).min(self.mmap.len());

                self.mmap
                    .advise_range(memmap2::Advice::WillNeed, start, stop - start)?;
            }

            let stop = (start + self.size_piece).min(self.mmap.len());

            let res = &self.mmap[start..stop];
            let piece = metadata_chunk::new(res);
            let out_data = piece.write_to_destination(path_dir_prefix.as_ref())?;
            ret.push(out_data);
        }

        Ok(ret)
    }
}
