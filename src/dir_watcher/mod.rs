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
        let events = self
            .notify
            .read_events_blocking(self.buffer.as_mut_slice())?;

        for event in events {
            let name = event.name;
            match name {
                Some(o) => {
                    ret.insert(self.path_dir_prefix_input.join(o));
                }
                None => {}
            };
        }

        return Ok(());
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
