mod chunk_worker;

use chunk_worker::*;

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
    ) -> anyhow::Result<Vec<metadata_chunk>> {
        const read_ahead: usize = 1 << 3;
        const read_mask: usize = read_ahead - 1;

        let mut ret: Vec<metadata_chunk> = Vec::with_capacity(self.n_pieces);

        for idx in 0..self.n_pieces {
            let start = idx * self.size_piece;

            if (idx & read_mask) == 0 {
                let stop = (start + ((read_ahead << 1) * self.size_piece)).min(self.mmap.len());

                self.mmap
                    .advise_range(memmap2::Advice::WillNeed, start, stop - start)?;
            }

            let stop = (start + self.size_piece).min(self.mmap.len());

            let res = &self.mmap[start..stop];
            let piece = hashed_chunk::new(res);
            let out_data = piece.write_to_destination(path_dir_prefix.as_ref())?;
            ret.push(out_data);
        }

        Ok(ret)
    }
}
