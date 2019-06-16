#![forbid(unsafe_code)]

#[macro_use]
mod global;

use crate::global::prelude::*;
use crate::global::errors::CustomErrorKind;

fn main() {
    global::initialize();
    main_result().crash_on_error();
}

fn main_result() -> Result {

    log!("cats");

    Ok(())
}
