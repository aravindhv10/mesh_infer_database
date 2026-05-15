struct metadata_file {
    fd: std::fs::File,
    mmap: memmap2::Mmap,
    size_piece: usize,
}

impl metadata_file {
    pub fn open(path_file_input: &std::path::Path, size_piece: usize) -> anyhow::Result<Self> {
        let fd = std::fs::File::open(path_file_input)?;
        let mmap = unsafe { memmap2::Mmap::map(&fd) }?;
        Ok(Self {
            fd: fd,
            mmap: mmap,
            size_piece: size_piece,
        })
    }

    #[inline(always)]
    pub fn get_len(&self) -> usize {
        self.mmap.len()
    }

    #[inline(always)]
    pub fn get_n_pieces(&self) -> usize {
        let n_pieces = self.mmap.len() / self.size_piece;
        let left_over = self.mmap.len() % self.size_piece;

        if left_over == 0 {
            return n_pieces;
        } else {
            return n_pieces + 1;
        }
    }
}
