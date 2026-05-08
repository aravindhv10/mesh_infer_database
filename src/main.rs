#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn run_sender() -> anyhow::Result<()> {
    eprintln!("run_sender");
    Ok(())
}

fn run_receiver() -> anyhow::Result<()> {
    eprintln!("run_receiver");
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let args: Vec<String> = std::env::args().collect();

    match args.len() {
        0 => {
            eprintln!("Bad command line array");
        }
        1 => {
            eprintln!("Need atleast 1 commandline argument {}", args[0]);
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
                    let mut child_send = std::process::Command::new(args[0].as_str())
                        .arg("send")
                        .spawn()?;

                    let mut child_receive = std::process::Command::new(args[0].as_str())
                        .arg("receive")
                        .spawn()?;

                    let status_send = child_send.wait()?;
                    let status_receive = child_receive.wait()?;

                    if !status_send.success() {
                        eprintln!("Sender failed");
                    }

                    if !status_receive.success() {
                        eprintln!("Receiver failed");
                    }
                }
            };
        }
    };

    println!("Hello, world!");

    Ok(())
}
