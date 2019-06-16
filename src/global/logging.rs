use std::sync::Mutex;
use std::path::{PathBuf, Path};
use std::fs::{File, OpenOptions};
use std::io::{SeekFrom, Write, Seek};

use chrono::Utc;

use super::prelude::*;

pub struct LoggingConfiguration {
    pub max_length: u64,
    pub file_path: PathBuf,
}

pub struct FileAppenderState {
    file_handle: File,
    file_length: u64,
}

pub struct FileAppender {
    state: Mutex<FileAppenderState>,
    config: LoggingConfiguration,
}

impl FileAppender {

    pub fn new(config: LoggingConfiguration) -> Result<FileAppender> {

        ::std::fs::create_dir_all(config.file_path.get_directory())?;

        let mut file_handle = FileAppender::create_file_handle(&config.file_path)?;
        let file_length = file_handle.seek(SeekFrom::End(0))?;

        Ok(FileAppender {
            state: Mutex::new(FileAppenderState {
                file_handle,
                file_length,
            }),
            config,
        })
    }

    fn create_file_handle(file_path: &Path) -> Result<File> {

        let file_handle = OpenOptions::new()
            .write(true)
            .create(true)
            .open(file_path)?;

        Ok(file_handle)
    }

    fn roll_file(&self, state: &mut FileAppenderState) -> Result {

        let file_stem = self.config.file_path.file_stem_as_string()?;
        let file_extension = self.config.file_path.extension_as_string()?;
        let directory = self.config.file_path.get_directory_as_string()?;

        let now = Utc::now();
        let formatted_date = now.format("%Y_%m_%d__").to_string();
        let nanos = now.timestamp_nanos();

        let new_file_path= format!("{}/{}__{}{}.{}", directory, file_stem, formatted_date, nanos, file_extension);
        let new_path = Path::new(&new_file_path);

        state.file_handle.sync_all()?;

        std::fs::rename(&self.config.file_path, new_path)?;

        let file_handle = FileAppender::create_file_handle(&self.config.file_path)?;

        state.file_handle = file_handle;
        state.file_length = 0;

        Ok(())
    }

    pub fn writeln(&self, message: &str) -> Result {

        let mut state = self.state.lock()?;

        if state.file_length >= self.config.max_length {
            self.roll_file(&mut state)?;
        }

        let len = state.file_handle.write(format!("{}\n", message).as_bytes())? as u64;

        state.file_length += len;

        Ok(())
    }
}

pub struct ConsoleAppender;

impl ConsoleAppender {

    pub fn new() -> ConsoleAppender {
        ConsoleAppender {}
    }

    pub fn writeln(&self, message: &str) -> Result {

        let stdout = &mut ::std::io::stdout();
        write!(stdout, "{}\n", message)?;

        Ok(())
    }
}

pub struct InMemoryAppender {

    pub entries: Mutex<Vec<String>>,
}

impl InMemoryAppender {

    pub fn new() -> InMemoryAppender {
        InMemoryAppender {
            entries: Mutex::new(Vec::new())
        }
    }

    pub fn add_entry(&self, message: &str) -> Result {

        let mut vec = self.entries.lock()?;
        vec.push(message.to_string());

        Ok(())
    }
}

pub struct Logger {
    file_appender: FileAppender,
    console_appender: ConsoleAppender,
    in_memory_appender: InMemoryAppender,
}

impl Logger {

    pub fn new(config: LoggingConfiguration) -> Result<Logger> {
        Ok(Logger {
            console_appender: ConsoleAppender::new(),
            in_memory_appender: InMemoryAppender::new(),
            file_appender: FileAppender::new(config)?
        })
    }

    fn format_message(&self, message: &str) -> Result<String> {

        let now = Utc::now();
        let formatted_date = now.format("%Y-%m-%d %H:%M:%S").to_string();

        Ok(format!("{} | {}", formatted_date, message))
    }

    pub fn log(&self, message: &str) -> Result {

        let formatted_message = self.format_message(message)?;

        let console_appender_result = self.console_appender.writeln(&formatted_message);
        let in_memory_appender_result = self.in_memory_appender.add_entry(message);
        let file_appender_result = self.file_appender.writeln(&formatted_message);

        console_appender_result?;
        in_memory_appender_result?;
        file_appender_result?;

        Ok(())
    }

    pub fn get_logs(&self) -> Result<Vec<String>> {

        let logs = self.in_memory_appender.entries.lock()?;

        Ok(logs.clone())
    }
}
