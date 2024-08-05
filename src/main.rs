use std::net::{IpAddr, Ipv4Addr};
use std::sync::mpsc::{Sender, channel};
use std::io::{self, Write};
use tokio::net::TcpStream;
use tokio::task;
use bpaf::Bpaf;

const MAX_PORT: u16 = 65535;
const IPFALLBACK: IpAddr = IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1));

#[derive(Debug, Clone, Bpaf)]
#[bpaf(options)]
struct Arguments {
    #[bpaf(long, short, fallback(IPFALLBACK))]
    /// IP Address to scan. Falls back to 127.0.0.1
    pub address: IpAddr,
    #[bpaf(long("start"), short('s'), fallback(1u16), guard(start_port_guard, "Must be non-negative"))]
    /// Starting port
    pub start_port: u16,
    #[bpaf(long("end"), short('e'), guard(end_port_guard, "Must be less than or equal to 65535"), fallback(MAX_PORT))]
    /// Ending port
    pub end_port: u16,
}

fn start_port_guard(input: &u16) -> bool {
    *input > 0
}

fn end_port_guard(input: &u16) -> bool {
    *input >= MAX_PORT
}



async fn scan(tx: Sender<u16>, port: u16, addr: IpAddr) {
    match TcpStream::connect(format!("{}:{}", addr, port)).await {
        Ok(_) => {
            print!(".");
            io::stdout().flush().unwrap();
            tx.send(port).unwrap();
        }
        Err(_) => {}
    }
        
}

#[tokio::main]
async fn main() {
    let args = arguments().run();

    let (tx, rx) = channel();

    for port in args.start_port .. args.end_port {
        let tx = tx.clone();

        task::spawn(async move {
            scan(tx, port, args.address).await;
        });
    }

    // put all open ports in a vec
    let mut out = vec![];
    drop(tx);
    for p in rx {
        out.push(p);
    }

    // display all open ports
    println!("");
    out.sort();
    for v in out {
        println!("{} is open", v);
    }

}
