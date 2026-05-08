#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn get_node() -> anyhow::Result<iceoryx2::node::Node<iceoryx2::service::ipc::Service>> {
    let node = iceoryx2::node::NodeBuilder::new().create::<iceoryx2::service::ipc::Service>()?;
    return Ok(node);
}

fn run_sender() -> anyhow::Result<()> {
    eprintln!("run_sender");
    // let node = iceoryx2::node::NodeBuilder::new().create::<iceoryx2::service::ipc::Service>()?;
    let node = get_node()?;

    Ok(())
}

fn run_receiver() -> anyhow::Result<()> {
    eprintln!("run_receiver");
    let node = get_node()?;
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

fn main() -> anyhow::Result<()> {
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
