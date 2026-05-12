pub struct dir_watcher {
    notify: inotify::Inotify,
    watchdescriptor: inotify::WatchDescriptor,
    buffer: Vec<u8>,
    path_dir_prefix_input: std::path::PathBuf,
    first_run: bool,
}

impl dir_watcher {
    pub fn new(path_dir_prefix_input: impl AsRef<std::path::Path>) -> anyhow::Result<Self> {
        let notify = inotify::Inotify::init()?;

        let watchdescriptor = notify.watches().add(
            path_dir_prefix_input.as_ref(),
            inotify::WatchMask::ATTRIB
                | inotify::WatchMask::CLOSE_WRITE
                | inotify::WatchMask::CLOSE_NOWRITE
                | inotify::WatchMask::MOVED_TO,
        )?;

        Ok(Self {
            notify: notify,
            watchdescriptor: watchdescriptor,
            buffer: vec![0; 1 << 21],
            path_dir_prefix_input: path_dir_prefix_input.as_ref().to_path_buf(),
            first_run: true,
        })
    }

    pub fn get_all_files(
        &mut self,
        ret: &mut std::collections::HashSet<std::path::PathBuf>,
    ) -> anyhow::Result<()> {
        let dir_read = std::fs::read_dir(&(self.path_dir_prefix_input))?;

        for file in dir_read {
            match file {
                Ok(o) => {
                    ret.insert(o.path());
                }
                Err(e) => {
                    eprintln!("Failed to read the file due to {}", e);
                }
            };
        }

        return Ok(());
    }

    pub fn get_new_files(
        &mut self,
        ret: &mut std::collections::HashSet<std::path::PathBuf>,
    ) -> anyhow::Result<()> {
        self.notify
            .read_events_blocking(self.buffer.as_mut_slice())?
            .into_iter()
            .map(|event| event.name)
            .flatten()
            .for_each(|i| {
                ret.insert(self.path_dir_prefix_input.join(i));
            });

        Ok(())
    }

    pub fn get_new_files_immediate(
        &mut self,
        ret: &mut std::collections::HashSet<std::path::PathBuf>,
    ) -> anyhow::Result<()> {
        self.notify
            .read_events(self.buffer.as_mut_slice())?
            .into_iter()
            .map(|event| event.name)
            .flatten()
            .for_each(|i| {
                ret.insert(self.path_dir_prefix_input.join(i));
            });

        Ok(())
    }

    pub fn get_batch(
        &mut self,
        ret: &mut std::collections::HashSet<std::path::PathBuf>,
        timeout: std::time::Duration,
        batch_size: usize,
        mut num_retries: u8,
    ) -> anyhow::Result<()> {
        self.get_new_files(ret)?;

        let start_time = std::time::Instant::now();

        while (ret.len() < batch_size) && (start_time.elapsed() < timeout) && (num_retries > 0) {
            match self.get_new_files_immediate(ret) {
                Ok(o) => {
                    num_retries -= 1;
                }
                Err(e) => {}
            }
        }

        Ok(())
    }

    pub fn get_files(&mut self) -> anyhow::Result<std::collections::HashSet<std::path::PathBuf>> {
        let mut ret: std::collections::HashSet<std::path::PathBuf> =
            std::collections::HashSet::new();

        if self.first_run {
            self.get_all_files(&mut ret)?;
            self.first_run = false;
        } else {
            self.get_new_files(&mut ret)?;
        }

        return Ok(ret);
    }
}
