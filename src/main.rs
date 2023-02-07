use std::net::SocketAddr;
use std::path::PathBuf;
use std::time::Duration;

use async_compression::tokio::write::ZstdEncoder;
use async_compression::Level;

use tokio::io::AsyncWriteExt;
use tokio_util::codec::{FramedRead, LinesCodec, LinesCodecError};

use futures::stream::StreamExt;

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    let flags = xflags::parse_or_exit! {
        required listenaddr: SocketAddr
        required outputfile: PathBuf
        optional -i,--flush-interval seconds: u64
        optional -l,--max-line-length bytes: usize
        optional -z,--zstd-compression-level level: u32
    };

    let tcp = tokio::net::TcpListener::bind(flags.listenaddr).await?;
    let out = tokio::fs::File::create(flags.outputfile).await?;

    let mut zout = ZstdEncoder::with_quality(
        out,
        Level::Precise(flags.zstd_compression_level.unwrap_or(12)),
    );

    let (tx, mut rx) = tokio::sync::mpsc::channel::<String>(16);

    let maxlinelen = flags.max_line_length.unwrap_or(65536);

    tokio::spawn(async move {
        loop {
            match tcp.accept().await {
                Ok((s, from)) => {
                    println!("Incoming connection from {from}");
                    let l = LinesCodec::new_with_max_length(maxlinelen);
                    let mut f = FramedRead::with_capacity(s, l, 1024);
                    let tx = tx.clone();
                    tokio::spawn(async move {
                        while let Some(x) = f.next().await {
                            match x {
                                Ok(x) => match tx.send(x).await {
                                    Ok(()) => (),
                                    Err(_) => {
                                        println!("Channel send failed");
                                        break;
                                    }
                                },
                                Err(LinesCodecError::MaxLineLengthExceeded) => {
                                    println!("  max line len exceed from {from}");
                                    break;
                                }
                                Err(e) => {
                                    println!("  io error from {from}: {e}");
                                    break;
                                }
                            }
                        }
                        println!("  finished serving {from}");
                    });
                }
                Err(e) => {
                    println!("accept: {e}");
                    continue;
                }
            }
        }
    });

    let mut timer = tokio::time::interval(Duration::from_secs(flags.flush_interval.unwrap_or(60)));
    timer.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

    loop {
        tokio::select! {
            _ = timer.tick() => {
                zout.flush().await?;
            }
            msg = rx.recv() => {
                match msg {
                    None => break,
                    Some(x) => {
                        zout.write_all(x.as_bytes()).await?;
                        zout.write_all(b"\n").await?;
                    }
                }
            }
        }
    }
    zout.flush().await?;

    Ok(())
}
