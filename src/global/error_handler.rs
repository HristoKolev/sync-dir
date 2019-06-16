use super::prelude::*;
use super::sentry_client;
use super::logger;

/// The default error handler.
pub fn handle_error(error: &CustomError) -> Result {

    let log_result = logger().log(&format!("An error occurred: {:#?}", error));
    let sentry_result = sentry_client().send_error(error);

    log_result?;
    sentry_result?;

    Ok(())
}

pub fn handle_fatal_error(error: &CustomError) -> Result {

    let standard_error_handler_result = handle_error(error);

    standard_error_handler_result?;

    Ok(())
}

pub trait ResultExtensionsCrashOnError<R> {

    fn crash_on_error(self) -> R;
}

impl<R> ResultExtensionsCrashOnError<R> for Result<R> {

    fn crash_on_error(self) -> R {

        match self {
            Ok(x) => x,
            Err(err) => {
                handle_fatal_error(&err)
                    .expect(&format!("An error occurred while handling an error. {:#?}", &err));

                ::std::process::exit(1)
            }
        }
    }
}
