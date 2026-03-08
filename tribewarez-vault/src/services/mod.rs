/// Vault security and access control service.
pub mod vault_security;

// Re-export main types for convenience
pub use vault_security::{
    SimpleVaultSecurity, TensorVaultSecurity, VaultError, VaultResult, VaultSecurityProvider,
};
