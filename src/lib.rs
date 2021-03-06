#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate crossbeam_channel;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate log;
#[macro_use]
extern crate observer_attribute;

#[cfg(any(
    all(
        feature = "postgre_default",
        any(feature = "mysql_default", feature = "sqlite_default")
    ),
    all(
        feature = "mysql_default",
        any(feature = "postgre_default", feature = "sqlite_default")
    ),
    all(
        feature = "sqlite_default",
        any(feature = "mysql_default", feature = "postgre_default")
    ),
))]
compile_error!("only one of postgre_default, mysql_default or sqlite_default can be activated");

pub mod base;
mod context;
pub mod iframe;
mod mode;
mod page;
pub mod request_config;
mod response;
pub mod serve;
pub mod serve_static;
pub mod storybook;
pub mod test;
mod urls;
pub mod utils;
pub mod watcher;

pub use crate::context::Context;
pub use crate::mode::Mode;
pub use crate::page::{Page, PageSpec};
pub use crate::request_config::RequestConfig;
pub use crate::response::{json, json_with_context};
pub use crate::serve::{http_to_hyper, THREAD_POOL};
pub use crate::serve_static::serve_static;
pub use crate::urls::{handle, is_realm_url};

pub use crate::response::Response;
pub type Result = std::result::Result<crate::response::Response, failure::Error>;
pub type Request = http::request::Request<Vec<u8>>;

pub trait Subject: askama::Template {}
pub trait Text: askama::Template {}
pub trait HTML: askama::Template {}

pub trait UserData: std::string::ToString + std::str::FromStr {}

#[derive(Fail, Debug)]
pub enum Error {
    #[fail(display = "404 Page Not Found: {}", message)]
    PageNotFound { message: String },

    #[fail(display = "Input Error: {:?}", error)]
    InputError {
        #[cause]
        error: crate::request_config::Error,
    },

    #[fail(display = "Form Error: {:?}", errors)]
    FormError {
        errors: std::collections::HashMap<String, String>,
    },

    #[fail(display = "Internal Server Error: {}", message)]
    CustomError { message: String },

    #[fail(display = "HTTP Error: {}", error)]
    HttpError {
        #[cause]
        error: http::Error,
    },

    #[fail(display = "Env Var Error: {}", error)]
    VarError {
        #[cause]
        error: std::env::VarError,
    },

    #[fail(display = "Diesel Error: {}", error)]
    DieselError {
        #[cause]
        error: diesel::result::Error,
    },
}

pub fn error<T>(key: &str, message: &str) -> std::result::Result<T, failure::Error> {
    let mut e = std::collections::HashMap::new();
    e.insert(key.into(), message.into());

    Err(Error::FormError { errors: e }.into())
}

impl From<diesel::result::Error> for Error {
    fn from(error: diesel::result::Error) -> Error {
        Error::DieselError { error }
    }
}

impl From<std::env::VarError> for Error {
    fn from(error: std::env::VarError) -> Error {
        Error::VarError { error }
    }
}

impl From<http::Error> for Error {
    fn from(error: http::Error) -> Error {
        Error::HttpError { error }
    }
}

impl From<crate::request_config::Error> for Error {
    fn from(error: crate::request_config::Error) -> Error {
        Error::InputError { error }
    }
}

pub trait Or404<T> {
    fn or_404(self) -> std::result::Result<T, failure::Error>;
}

impl<T> Or404<T> for std::result::Result<T, failure::Error> {
    fn or_404(self) -> std::result::Result<T, failure::Error> {
        self.map_err(|e| {
            Error::PageNotFound {
                message: e.to_string(),
            }
            .into()
        })
    }
}
