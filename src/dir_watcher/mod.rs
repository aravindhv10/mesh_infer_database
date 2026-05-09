pub struct dir_watcher {
    notify: inotify::Inotify,
    watchdescriptor: inotify::WatchDescriptor,
    buffer: Vec<u8>,
    path_dir_prefix_input: std::string::String,
    first_run: bool,
}

impl dir_watcher {
    pub fn new(path_dir_prefix_input: std::string::String) -> anyhow::Result<Self> {
        let mut notify = inotify::Inotify::init()?;

        let watchdescriptor = notify.watches().add(
            path_dir_prefix_input.as_str(),
            inotify::WatchMask::ATTRIB
                | inotify::WatchMask::CLOSE_WRITE
                | inotify::WatchMask::CLOSE_NOWRITE
                | inotify::WatchMask::MOVED_TO,
        )?;

        let buffer: Vec<u8> = vec![0; 1 << 21];

        Ok(Self {
            notify: notify,
            watchdescriptor: watchdescriptor,
            buffer: buffer,
            path_dir_prefix_input: path_dir_prefix_input,
            first_run: true,
        })
    }

    pub fn get_files(&mut self) -> anyhow::Result<std::collections::HashSet<std::string::String>> {
        let mut ret: std::collections::HashSet<std::string::String> =
            std::collections::HashSet::new();

        if self.first_run {
            let dir_read = std::fs::read_dir(self.path_dir_prefix_input.as_str())?;
            for file in dir_read {
                match file {
                    Ok(o) => {
                        ret.insert(o.path().to_string_lossy().to_string());
                    }
                    Err(e) => {
                        eprintln!("Failed to read the file due to {}", e);
                    }
                };
            }
            self.first_run = false;
        } else {
            // eprintln!("Starting the notify loop");
            let events = self
                .notify
                .read_events_blocking(self.buffer.as_mut_slice())?;
            // eprintln!("Got events...");

            for event in events {
                let name = event.name;
                match name {
                    Some(o) => {
                        ret.insert(
                            self.path_dir_prefix_input.clone()
                                + "/"
                                + o.to_string_lossy().to_string().as_str(),
                        );
                    }
                    None => {
                        // eprint!("Got unnamed notification...");
                    }
                };
            }
        }

        return Ok(ret);
    }
}
