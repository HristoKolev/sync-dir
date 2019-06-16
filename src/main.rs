#![forbid(unsafe_code)]

#[macro_use]
mod global;

use notify::{Watcher, RecursiveMode, watcher};
use std::sync::mpsc::channel;
use std::time::Duration;

use crate::global::prelude::*;
use crate::global::errors::CustomErrorKind;

fn main() {
    global::initialize();
    main_result().crash_on_error();
}

fn main_result() -> Result {

    let (tx, rx) = channel();

    let mut watcher = watcher(tx, Duration::from_secs(1))?;

    watcher.watch("/work/projects/xdxd-backup/target/", RecursiveMode::Recursive)?;

    loop {
        match rx.recv() {
            Ok(event) => {
                println!("{:?}", event)
            },
            Err(e) => {
                println!("watch error: {:?}", e)
            },
        }
    }

    Ok(())
}
