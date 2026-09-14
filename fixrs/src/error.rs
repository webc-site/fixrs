use std::{io, result};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
  #[error(transparent)]
  Io(#[from] io::Error),

  #[error(transparent)]
  Syn(#[from] syn::Error),

  #[error(transparent)]
  Ignore(#[from] ignore::Error),
}

pub type Result<T> = result::Result<T, Error>;
