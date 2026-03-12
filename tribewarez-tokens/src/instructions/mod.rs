//! Instruction handlers for the token program

pub mod burn;
pub mod freeze_account;
pub mod initialize_mint;
pub mod mint;
pub mod set_mint_authority;
pub mod thaw_account;
pub mod transfer;
pub mod update_metadata;

pub use burn::*;
pub use freeze_account::*;
pub use initialize_mint::*;
pub use mint::*;
pub use set_mint_authority::*;
pub use thaw_account::*;
pub use transfer::*;
pub use update_metadata::*;
