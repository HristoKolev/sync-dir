use std::time::{SystemTime, UNIX_EPOCH};
use std::borrow::Cow;

use url::{Url};
use serde::{Serialize, Deserialize};
use sentry::protocol::{Event, Exception, Stacktrace};
use sentry::utils;
use regex::{Regex};

use super::prelude::*;
use super::sentry_backtrace::*;

#[derive(Serialize, Deserialize, Debug)]
pub struct CustomDsn {
    pub scheme: String,
    pub domain: String,
    pub port: u16,
    pub path: String,
    pub project_id: String,
    pub public_key: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CustomSentryClient {
    dsn: CustomDsn,
}

impl CustomSentryClient {

    pub fn new(dsn_string: &str) -> Result<CustomSentryClient> {

        let dsn = parse_dsn(dsn_string)?;

        Ok(CustomSentryClient {
            dsn
        })
    }

    #[allow(unused)]
    pub fn send_message(&self, message: &str) -> Result {

        let mut event = self.create_event();
        event.message = Some(message.to_string());

        self.send_event(&event)?;

        Ok(())
    }

    pub fn send_error(&self, error: &CustomError) -> Result {

        let stacktrace = self.get_stacktrace(error)?;

        let mut event = self.create_event();

        let exception = Exception {
            value: Some(error.kind.to_string()),
            ty: error_typename(&error.kind),
            stacktrace,
            ..Default::default()
        };

        event.exception = vec![exception].into();

        self.send_event(&event)?;

        Ok(())
    }

    fn get_stacktrace(&self, error: &CustomError) -> Result<Option<Stacktrace>> {

        let mut stacktrace = backtrace_to_stacktrace(&error.backtrace);

        if let Some(ref mut stacktrace) = stacktrace {

            // automatically prime in_app and set package
            let mut any_in_app = false;

            for frame in &mut stacktrace.frames {

                let func_name = match frame.function {
                    Some(ref func) => func,
                    None => continue,
                };

                // set package if missing to crate prefix
                if frame.package.is_none() {
                    frame.package = parse_crate_name(func_name);
                }

                match frame.in_app {
                    Some(true) => {
                        any_in_app = true;
                        continue;
                    }
                    Some(false) => {
                        continue;
                    }
                    None => {}
                }

                if frame.in_app.is_some() {
                    continue;
                }

                if is_sys_function(func_name) {
                    frame.in_app = Some(false);
                }
            }

            stacktrace.frames.retain(|x | {
                if let Some(func_name) = &x.function {
                    if func_name.starts_with(&format!("<{}::global::errors::CustomError as core::convert::From", env!("CARGO_PKG_NAME"))) {
                        return false;
                    }
                    return true;
                }
                return true;
            });

            if !any_in_app {
                for frame in &mut stacktrace.frames {
                    if frame.in_app.is_none() {
                        frame.in_app = Some(true);
                    }
                }
            }
        }

        Ok(stacktrace)
    }

    fn send_event(&self, event: &Event) -> Result {

        let client = reqwest::Client::new();

        let request_url = format!(
            "{}://{}:{}{}/api/{}/store/",
            self.dsn.scheme,
            self.dsn.domain,
            self.dsn.port,
            self.dsn.path,
            self.dsn.project_id,
        );

        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();

        let auth_header = format!(
            "Sentry sentry_version={}, sentry_client={}, sentry_timestamp={}, sentry_key={}",
            7,
            "Custom Sentry Client/1.0.0",
            timestamp,
            self.dsn.public_key
        );

        let request_body = serde_json::to_string(event)?;

        let request = client
            .post(&request_url)
            .body(request_body)
            .header("X-Sentry-Auth",&*auth_header);

        let _response = request.send()?;

        Ok(())
    }

    fn create_event(&self) -> Event {

        let mut event = Event::new();

        event.server_name = utils::server_name().map(Cow::Owned);
        event.platform = "native".into();
        event.release = ::std::env::var("SENTRY_RELEASE").ok().map(Cow::Owned);
        event.environment = ::std::env::var("SENTRY_ENVIRONMENT")
            .ok().map(Cow::Owned).or_else(|| {
                Some(Cow::Borrowed(if cfg!(debug_assertions) {
                    "debug"
                } else {
                    "release"
                }))
            });

        if let Some (os) = utils::os_context(){
            event.contexts.insert("os".to_string(), os);
        }

        if let Some (rust) = utils::rust_context(){
            event.contexts.insert("rust".to_string(), rust);
        }

        if let Some (device) = utils::device_context(){
            event.contexts.insert("device".to_string(), device);
        }

        event
    }
}

fn parse_dsn(dsn: &str) -> Result<CustomDsn> {

    let url = Url::parse(dsn)?;

    let domain = url.domain().ok_or_else(|| CustomError::from_message("Invalid dsn domain."))?;
    let port = url.port().unwrap_or(80);
    let scheme = url.scheme();
    let last_index = url.path().last_index_of('/').unwrap_or(0);
    let path = &url.path()[0..last_index];
    let project_id = &url.path()[(last_index + 1)..];
    let public_key = url.username();

    Ok(CustomDsn {
        scheme: scheme.to_string(),
        domain: domain.to_string(),
        port,
        path: path.to_string(),
        project_id: project_id.to_string(),
        public_key: public_key.to_string(),
    })
}

lazy_static::lazy_static! {
    static ref CRATE_RE: Regex = Regex::new(r"^(?:_<)?([a-zA-Z0-9_]+?)(?:\.\.|::)").unwrap();
}

/// Tries to parse the rust crate from a function name.
fn parse_crate_name(func_name: &str) -> Option<String> {
    CRATE_RE
        .captures(func_name)
        .and_then(|caps| caps.get(1))
        .map(|cr| cr.as_str().into())
}
