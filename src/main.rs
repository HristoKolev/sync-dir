#![forbid(unsafe_code)]

#[macro_use]
mod global;

use std::sync::mpsc::channel;
use std::time::Duration;
use std::sync::{Mutex, Arc};
use std::thread::JoinHandle;

use crate::global::prelude::*;

use notify::{Watcher, RecursiveMode, watcher, DebouncedEvent};
use std::path::{PathBuf, Path};

fn main() {
    global::initialize();
    main_result().crash_on_error();
}

fn main_result() -> Result {

    let args = ::std::env::args_os()
        .skip(1)
        .map_result(|x| x.get_as_string())?
        .collect_vec();

    if args.len() < 2 {
        log!("Error: not enough parameters.");
        ::std::process::exit(1);
    }

    let source_path = args[0].clone();
    let destination_path = args[1].clone();

    let (sender, receiver) = channel();

    let mut watcher = watcher(sender, Duration::from_millis(100))?;
    watcher.watch(&source_path, RecursiveMode::Recursive)?;

    let flag = Arc::new(Mutex::new(false));

    let syncignore_path = Path::new(&source_path).join(".syncignore");

    let syncignore = if syncignore_path.exists() {

        let (ignore, opt_err) = ::ignore::gitignore::Gitignore::new(syncignore_path);

        Some(match opt_err {
            Some(x) => Err(x),
            None => Ok(ignore)
        }?)
    } else {
        None
    };

    let watch_flag = flag.clone();

    let watch_thread: JoinHandle<Result> = ::std::thread::spawn(move || {

        loop {
            match receiver.recv() {
                Ok(event) => {

                    if let Some(path) = event.get_path() {
                        if let Some(syncignore) = &syncignore {
                            if syncignore.matched(&path, path.is_dir()).is_ignore() {
                                continue;
                            }
                        }
                    }

                    let mut value = watch_flag.lock()?;
                    *value = true;
                },
                Err(error) => elog!("{:#?}", error),
            }
        }
    });

    let sync_flag = flag.clone();

    let sync_thread: JoinHandle<Result> = ::std::thread::spawn(move || {

        loop {
            ::std::thread::sleep(Duration::from_millis(100));

            let mut value = sync_flag.lock()?;

            if *value {

                bash_exec!(
                    r##"rsync -aP --exclude='/.git' --filter="dir-merge,- .syncignore" {} {}"##,
                    source_path,
                    destination_path
                );

                *value = false;
            }
        }
    });

    watch_thread.join().replace_error(||
        CustomError::from_message("The receiver thread failed for some reason."))??;

    sync_thread.join().replace_error(||
        CustomError::from_message("The sync thread failed for some reason."))??;

    Ok(())
}

trait DebounceEventExtensions {
    fn get_path(&self) -> Option<PathBuf>;
}

impl DebounceEventExtensions for DebouncedEvent {

    fn get_path(&self) -> Option<PathBuf> {
        match self {
            DebouncedEvent::NoticeWrite(x) => Some(x.clone()),
            DebouncedEvent::NoticeRemove(x) => Some(x.clone()),
            DebouncedEvent::Create(x) => Some(x.clone()),
            DebouncedEvent::Write(x) => Some(x.clone()),
            DebouncedEvent::Chmod(x) => Some(x.clone()),
            DebouncedEvent::Remove(x) => Some(x.clone()),
            DebouncedEvent::Rename(_, x) => Some(x.clone()),
            DebouncedEvent::Rescan => None,
            DebouncedEvent::Error(_, x) => x.clone(),
        }
    }
}