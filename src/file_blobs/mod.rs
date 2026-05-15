pub struct metadata_chunk<'a> {
    chunk: &'a [u8],
    hash: blake3::Hash,
}

impl<'a> metadata_chunk<'a> {
    pub fn new(chunk: &'a [u8]) -> Self {
        let hash = blake3::hash(chunk);
        Self { chunk, hash }
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

    #[inline(always)]
    pub fn get_piece(&self, idx: usize) -> anyhow::Result<&[u8]> {
        if idx >= self.n_pieces {
            return Err(anyhow::format_err!("Index out of bounds..."));
        }
        let start = idx * self.size_piece;
        let stop = (start + self.size_piece).min(self.mmap.len());
        let res = &self.mmap[start..stop];
        return Ok(res);
    }

    pub fn get_all_chunks(&'a self) -> Vec<metadata_chunk<'a>> {
        self.mmap
            .chunks(self.size_piece)
            .map(|slice| metadata_chunk::new<'a>(slice))
            .collect()
    }
}
