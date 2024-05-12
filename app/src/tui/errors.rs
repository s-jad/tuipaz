use color_eyre::{
    config::HookBuilder,
    eyre::{self, eyre},
};
use std::error::Error;
use std::fmt;
use std::panic;

use crate::tui::utils;

/// restore the terminal before printing the panic or error.
pub fn install_hooks() -> color_eyre::Result<()> {
    let (panic_hook, eyre_hook) = HookBuilder::default().into_hooks();

    // convert from a color_eyre PanicHook to a standard panic hook
    let panic_hook = panic_hook.into_panic_hook();
    panic::set_hook(Box::new(move |panic_info| {
        utils::restore().unwrap();
        panic_hook(panic_info);
    }));

    // convert from a color_eyre EyreHook to a eyre ErrorHook
    let eyre_hook = eyre_hook.into_eyre_hook();
    eyre::set_hook(Box::new(
        move |error: &(dyn std::error::Error + 'static)| {
            utils::restore().unwrap();
            eyre_hook(error)
        },
    ))?;

    Ok(())
}

#[derive(Debug)]
pub(crate) struct DbError(pub Box<dyn Error + Send + Sync>);
impl fmt::Display for DbError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for DbError {}

impl From<eyre::Report> for DbError {
    fn from(report: eyre::Report) -> Self {
        DbError(report.into())
    }
}

impl From<sqlx::Error> for DbError {
    fn from(err: sqlx::Error) -> Self {
        DbError(err.into())
    }
}

pub(crate) fn create_db_error(msg: String) -> DbError {
    let err = eyre!(msg);
    DbError::from(err)
}
