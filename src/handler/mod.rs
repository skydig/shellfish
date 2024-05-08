//! # Handler
//!
//! In Shellfish handlers act as a way of interpreting commands. This means
//! they take the command arguments, along with a couple of state parameters
//! and run the command. For most cases you want
//! [`DefaultHandler`](default::DefaultHandler), unless you are doing async
//! in which case [`DefaultAsyncHandler`](asynchronous::DefaultAsyncHandler)
//! is for you.

pub mod default;
pub use default::*;

#[cfg(feature = "app")]
#[cfg_attr(nightly, doc(cfg(feature = "app")))]
pub mod app;
#[cfg_attr(nightly, doc(cfg(feature = "app")))]
#[cfg(feature = "app")]
pub use app::*;

#[cfg(feature = "async")]
#[cfg_attr(nightly, doc(cfg(feature = "async")))]
pub mod asynchronous;
#[cfg(feature = "async")]
#[cfg_attr(nightly, doc(cfg(feature = "async")))]
pub use asynchronous::*;

#[cfg(all(feature = "app", feature = "async"))]
#[cfg_attr(nightly, doc(cfg(all(feature = "async", feature = "app"))))]
pub mod async_app;
#[cfg_attr(nightly, doc(cfg(all(feature = "async", feature = "app"))))]
#[cfg(all(feature = "app", feature = "async"))]
pub use async_app::*;
