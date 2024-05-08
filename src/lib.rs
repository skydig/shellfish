#![cfg_attr(nightly, feature(doc_cfg))]

#[doc=include_str!("../README.md")]
pub mod command;
pub use command::Command;

pub mod handler;
#[cfg(feature = "async")]
pub use handler::AsyncHandler;
pub use handler::Handler;
pub mod input_handler;
pub use input_handler::InputHandler;

#[cfg(feature = "app")]
#[cfg_attr(nightly, doc(cfg(feature = "app")))]
pub mod app;
#[cfg(feature = "app")]
#[cfg_attr(nightly, doc(cfg(feature = "app")))]
pub use app::App;

pub mod shell;
pub use shell::Shell;

#[cfg(feature = "clap")]
#[cfg_attr(nightly, doc(cfg(feature = "clap")))]
mod clap_command;

#[cfg(feature = "rustyline")]
pub use rustyline;
