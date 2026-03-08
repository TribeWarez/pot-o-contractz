/// Vault security and access control service.
pub mod vault_security;

// Re-export main types for convenience
pub use vault_security::{
    VaultSecurityProvider, VaultError, VaultResult,
    SimpleVaultSecurity, TensorVaultSecurity,
};
