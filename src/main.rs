#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use std::any::Any;

// use iceoryx2::prelude::ZeroCopySend;
use iceoryx2::prelude::*;

#[derive(ZeroCopySend, Debug)]
#[repr(C)]
struct message {
    data: i128,
}

fn get_node() -> anyhow::Result<(
    iceoryx2::node::Node<iceoryx2::service::ipc::Service>,
    iceoryx2::service::service_name::ServiceName,
)> {
    let node = iceoryx2::node::NodeBuilder::new().create::<iceoryx2::service::ipc::Service>()?;
    let servicename = iceoryx2::service::service_name::ServiceName::new("testing")?;
    return Ok((node, servicename));
}

fn run_sender() -> anyhow::Result<()> {
    eprintln!("run_sender");
    let (node, servicename) = get_node()?;

    let publisher = node
        .service_builder(&servicename)
        .publish_subscribe::<message>()
        .open_or_create()?
        .publisher_builder()
        .create()?;

    std::thread::sleep(std::time::Duration::from_secs(1));

    let sample = publisher.loan_uninit()?;

    let res = sample.write_payload(message { data: 123 as i128 }).send()?;

    eprintln!("Wrote size {}", res);

    Ok(())
}

fn run_receiver() -> anyhow::Result<()> {
    eprintln!("run_receiver");
    let (node, servicename) = get_node()?;

    let subscriber = node
        .service_builder(&servicename)
        .publish_subscribe::<message>()
        .open_or_create()?
        .subscriber_builder()
        .create()?;

    let listener = node
        .service_builder(&servicename)
        .event()
        .open_or_create()?
        .listener_builder()
        .create()?;

    std::thread::sleep(std::time::Duration::from_secs(1));

    let res = listener.timed_wait_one(std::time::Duration::from_secs(5))?;

    let res = subscriber.receive()?;

    match res {
        Some(o) => {
            eprintln!("Received message {}", o.data);
        }
        None => {
            eprint!("Did not receive any message");
        }
    }

    Ok(())
}

fn run_both(self_path: &str) -> anyhow::Result<()> {
    let mut child_send = std::process::Command::new(self_path).arg("send").spawn()?;

    let mut child_receive = std::process::Command::new(self_path)
        .arg("receive")
        .spawn()?;

    let status_send = child_send.wait()?;
    let status_receive = child_receive.wait()?;

    if !status_send.success() {
        eprintln!("Sender failed");
        return Err(anyhow::format_err!("Sender failed"));
    }

    if !status_receive.success() {
        eprintln!("Receiver failed");
        return Err(anyhow::format_err!("Receiver failed"));
    }

    return Ok(());
}

fn iceoryx2_main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    match args.len() {
        0 => {
            eprintln!("Bad command line array");
        }
        1 => {
            return run_both(/*self_path: &str =*/ args[0].as_str());
        }
        _ => {
            match args[1].as_str() {
                "send" => {
                    return run_sender();
                }
                "receive" => {
                    return run_receiver();
                }
                _ => {
                    return run_both(/*self_path: &str =*/ args[0].as_str());
                }
            };
        }
    };

    println!("Hello, world!");

    Ok(())
}

struct dir_watcher {
    notify: inotify::Inotify,
    watchdescriptor: inotify::WatchDescriptor,
    buffer: Vec<u8>,
    path_dir_prefix_input: std::string::String,
    first_run: bool,
}

impl dir_watcher {
    fn new(path_dir_prefix_input: std::string::String) -> anyhow::Result<Self> {
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

    fn get_files(&mut self) -> anyhow::Result<std::collections::HashSet<std::string::String>> {
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
                            path_dir_prefix_input + "/" + o.to_string_lossy().to_string().as_str(),
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

fn main() -> anyhow::Result<()> {
    let mut watched = dir_watcher::new("/dev/shm".to_string())?;
    loop {
        for i in watched.get_files()? {
            eprintln!("Found file {}", i);
        }
    }
    return Ok(());
}
