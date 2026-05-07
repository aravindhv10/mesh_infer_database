#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

fn main() {
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
                    eprint!("Operating on send mode");
                }
                "receive" => {
                    eprint!("Operating on receive mode");
                }
                _ => {
                    eprint!("Unknown mode");
                }
            };
        }
    };

    println!("Hello, world!");
}
