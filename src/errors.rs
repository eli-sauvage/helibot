use std::ffi::OsString;

use thiserror::Error;


#[derive(Error, Debug)]
pub enum HelibotError{
    #[error("Sqlx error")]
    Sqlx(#[from] sqlx::error::Error),
    #[error(transparent)]
    EnvVarError(EnvVarError)
}

#[derive(Error, Debug)]
pub enum EnvVarError{
    #[error("could not convert os string to string")]
    OsString(OsString),
    #[error("the variable was not found")]
    VarNotFound
}
