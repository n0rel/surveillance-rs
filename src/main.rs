mod cli;
mod config;
mod streams;

use log::debug;
use std::thread;
use tokio::sync::mpsc::{self, Sender};

use cli::Cli;
use config::parse_configuration;
use streams::{RTSPStream, Stream, StreamFrame};

/// Synchronous thread for spawning streams.
/// Each stream is spawned in a new thread and
/// is given a copy of `tx` in order to send
/// frames to the connected receiver.
fn streams_thread(tx: &Sender<StreamFrame>, streams: Vec<impl Stream + Send>) {
    thread::scope(|scope| {
        for stream in streams {
            scope.spawn(move || {
                let tx_clone = tx.clone();
                stream.stream(&tx_clone);
            });
        }
    });
}

/// Notes:
/// 1. Important things to monitor:
///     a.  The channels capacity - as more streams are added, more async consumers need
///         to be added in order to not cap the maximum items in the channel.
///     b.  Stream FPS and stability - I want to know if I missed frames from streams

#[tokio::main]
async fn main() {
    env_logger::init();

    let args = Cli::init();
    let configuration = parse_configuration(&args.config).unwrap();

    let (tx, mut rx) = mpsc::channel::<StreamFrame>(10);

    let mut streams = Vec::new();
    for source_configuration in configuration.sources {
        streams.push(RTSPStream {
            stream_name: source_configuration.name,
            rtsp_uri: source_configuration.source_uri,
        });
    }

    tokio::task::spawn_blocking(move || {
        streams_thread(&tx, streams);
        drop(tx);
    });

    tokio::spawn(async move {
        debug!("Running asynchronous consumer");
        while let Some(frame) = rx.recv().await {
            println!("[{:?}], {}", frame.source, frame.data.len())
        }
    })
    .await
    .unwrap();
}
