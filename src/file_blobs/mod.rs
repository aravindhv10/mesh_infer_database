use std::io::Write;

pub struct metadata_chunk_info {
    pub hash: blake3::Hash,
    pub size: usize,
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
        let res = (self.hash).to_hex().to_string();
        let tmp = path_dir_prefix.as_ref().join(&res[0..2]).join(&res[2..4]);
        std::fs::create_dir_all(&tmp)?;
        let mut fd =
            std::fs::File::create(tmp.join(res + "_" + self.chunk.len().to_string().as_str()))?;

        fd.write_all(self.chunk)?;

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
        size_piece: usize,
    ) -> anyhow::Result<Self> {
        let fd = std::fs::File::open(path_file_input)?;

        let mmap = unsafe { memmap2::Mmap::map(&fd) }?;
        mmap.advise(memmap2::Advice::Sequential)?;

        let n_pieces = {
            let tmp = mmap.len() / size_piece;
            if (mmap.len() % size_piece) == 0 {
                tmp
            } else {
                tmp + 1
            }
        };

        Ok(Self {
            fd: fd,
            mmap: mmap,
            size_piece: size_piece,
            n_pieces: n_pieces,
        })
    }

    #[inline(always)]
    pub fn get_len(&self) -> usize {
        self.mmap.len()
    }

    #[inline(always)]
    pub fn get_n_pieces(&self) -> usize {
        self.n_pieces
    }

    pub fn get_piece(&self, idx: usize) -> anyhow::Result<&[u8]> {
        if idx >= self.n_pieces {
            return Err(anyhow::format_err!("Index out of bounds..."));
        }
        let start = idx * self.size_piece;
        let stop = (start + self.size_piece).min(self.mmap.len());

        self.mmap
            .advise_range(memmap2::Advice::WillNeed, start, stop - start)?;

        let res = &self.mmap[start..stop];
        return Ok(res);
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

    pub fn get_all_chunks(&self) -> Vec<metadata_chunk<'_>> {
        self.mmap
            .chunks(self.size_piece)
            .map(|slice| metadata_chunk::new(slice))
            .collect()
    }
}
