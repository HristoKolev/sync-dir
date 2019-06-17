#![forbid(unsafe_code)]

#[macro_use]
mod global;

use notify::{Watcher, RecursiveMode, watcher, raw_watcher, Op};
use std::sync::mpsc::channel;
use std::time::Duration;

use crate::global::prelude::*;
use crate::global::errors::CustomErrorKind;
use std::sync::{Mutex, Arc};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;

fn main() {
    global::initialize();
    main_result().crash_on_error();
}

fn main_result() -> Result {

    let (sender, receiver) = channel();

    let mut watcher = watcher(sender, Duration::from_millis(100))?;
    watcher.watch("/work/projects/", RecursiveMode::Recursive)?;

    let flag = Arc::new(Mutex::new(false));

    let watch_flag = flag.clone();

    let watch_thread: JoinHandle<Result> = ::std::thread::spawn(move || {

        loop {
            match receiver.recv() {
                Ok(event) => {
                    log!("got event...");
                    log!("{:#?}", event);
                    let mut val = watch_flag.lock()?;
                    *val = true;
                },
                Err(e) => {
                    println!("watch error: {:?}", e)
                },
            }
        }

        Ok(())
    });

    let sync_flag = flag.clone();

    let sync_thread: JoinHandle<Result> = ::std::thread::spawn(move || {

        loop {

            ::std::thread::sleep(Duration::from_millis(100));

            let mut val = sync_flag.lock()?;

            if *val {
                bash_exec!("rsync /work/projects/ root@docker-vm1.lan:/work/projects/");
                *val = false;
            }
        }

        Ok(())
    });

    watch_thread.join().replace_error(||
        CustomError::from_message("The receiver thread failed for some reason."))??;

    sync_thread.join().replace_error(||
        CustomError::from_message("The sync thread failed for some reason."))??;

    Ok(())
}
