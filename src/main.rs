use std::net::IpAddr;
use clap::Parser;
use tokio::{net::{TcpStream}, runtime::Runtime};

#[derive(Debug, Parser)]
struct Args {
    #[arg()]
    addr: IpAddr,

    #[arg(long, default_value_t = 1)]
    port_start: u16,

    #[arg(long, default_value_t = 1024)]
    #[arg(long)]
    port_end: u16
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    assert!(args.port_start > 0);
    assert!(args.port_end >= args.port_start);

    let rt = Runtime::new()?;
    rt.block_on(async {
        for port in args.port_start..=args.port_end {
            tokio::spawn(async move {
                TcpStream::connect((args.addr, port)).await;
                if let Ok(_open) = connection_attempt {
                    println!("= {}:{}", args.addr, port);
                }
            })
        }
    });

    println!("{}", args.addr);

    Ok(())
}
