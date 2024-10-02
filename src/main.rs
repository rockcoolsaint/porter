use std::net::IpAddr;
use clap::Parser;
use tokio::{net::TcpStream, runtime::Runtime, sync::mpsc};

#[derive(Debug, Parser)]
struct Args {
    #[arg()]
    addr: Option<IpAddr>,

    #[arg(long)]
    cidr: Option<cidr::IpCidr>,

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
    let (tx, mut rx) = mpsc::channel(10);
    rt.block_on(async {

        // This is commented out for a better equivalent
        // let mut tasks = vec![];

        // for port in args.port_start..=args.port_end {
        //     let tx = tx.clone();
        //     let task = tokio::spawn(async
        //         move {
        //         let scan_attempt
        //         = scan(args.addr, port, tx).await;
        //         if let Err(err) =
        //         scan_attempt {
        //             eprintln!("error: {err}");
        //         }
        //     });

        //     tasks.push(task);
        // }

        let n_tasks_per_network = (args.port_end - args.port_start) as usize;
        let mut tasks: Vec<_> = Vec::with_capacity(n_tasks_per_network);

        let (mut from_single, mut from_cidr);

        let addrs: &mut dyn Iterator<Item = IpAddr> = if let Some(addr) = args.addr {
            from_single = vec![addr].into_iter();
            &mut from_single
        } else if let Some(network) = args.cidr {
            from_cidr = network.iter().map(|net| net.address());
            &mut from_cidr
        } else {
            unreachable!()
        };

        for addr in addrs {
            println!("? {addr}:{}-{}", args.port_start, args.port_end);
            for port in args.port_start..=args.port_end {
                let tx = tx.clone();
                let task = tokio::spawn(async move {
                    if let Err(err) = scan(addr, port, tx).await {
                        eprintln!("error: {err}")
                    };
                });

                tasks.push(task);
            }
        }

        for task in tasks {
            task.await.unwrap();
        }
    });

    drop(tx);

    // println!("{}", args.addr);
    while let Ok((addr, port)) = rx.try_recv() {
        println!("= {addr}:{port}")
    }
    Ok(())
}

async fn scan(addr: IpAddr, port: u16, results_tx: mpsc::Sender<(IpAddr, u16)>) -> Result<(), mpsc::error::SendError<(IpAddr, u16)>> {
    let connection_attempt =
    TcpStream::connect((addr, port)).await;
    if let Ok(_open) = connection_attempt {
        results_tx.send((addr, port)).await.unwrap();
    };

    Ok(())
}
