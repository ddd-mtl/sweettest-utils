#![allow(unused_doc_comments)]
#![allow(non_upper_case_globals)]

#[cfg(not(target_arch = "wasm32"))]
mod print;
#[cfg(not(target_arch = "wasm32"))]
mod setup;
#[cfg(not(target_arch = "wasm32"))]
mod try_zome_call;
#[cfg(not(target_arch = "wasm32"))]
mod sweeter_cell;

#[cfg(not(target_arch = "wasm32"))]
pub use print::*;
#[cfg(not(target_arch = "wasm32"))]
pub use setup::*;
#[cfg(not(target_arch = "wasm32"))]
pub use try_zome_call::*;
#[cfg(not(target_arch = "wasm32"))]
pub use sweeter_cell::*;

//----------------------------------------------------------------------------------------

pub const ALEX_NICK: &str = "alex";
pub const BILLY_NICK: &str = "billy";
pub const CAMILLE_NICK: &str = "camille";

