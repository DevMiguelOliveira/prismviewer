//! prysmview-host - Atualização automática assinada e mecanismo de rollback.
//!
//! Conforme as Seções 6.4, 9 (Requisito 13) e 12 (Fase 11) do AGENTS.md:
//! - Validação de integridade via hash SHA-256 com comparação em tempo constante.
//! - Validação estrita de autenticidade via assinatura digital Ed25519 contra a chave pública embutida.
//! - Rejeição sumária de pacotes não assinados, com assinatura corrompida ou hash divergente.
//! - Backup atômico prévio do executável anterior e rollback automático se a inicialização falhar.
//! - Suporte a canais 'stable' e 'beta'.

use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use subtle::ConstantTimeEq;
use thiserror::Error;

/// Canal de distribuição da atualização
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum UpdateChannel {
    Stable,
    Beta,
}

/// Manifesto descritivo de uma versão lançada
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct UpdateManifest {
    pub version: String,
    pub channel: UpdateChannel,
    pub package_url: String,
    pub package_sha256: [u8; 32],
    pub signature_ed25519: Vec<u8>,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum UpdateError {
    #[error("Assinatura digital Ed25519 inválida ou forjada")]
    InvalidSignature,

    #[error("Hash SHA-256 do pacote baixado diverge do manifesto")]
    HashMismatch,

    #[error("Chave pública Ed25519 fornecida é inválida")]
    InvalidPublicKey,

    #[error("Erro de E/S no gerenciamento de arquivos: {0}")]
    Io(String),

    #[error("Verificação pós-atualização falhou. Rollback executado")]
    HealthCheckFailed,
}

impl From<std::io::Error> for UpdateError {
    fn from(err: std::io::Error) -> Self {
        UpdateError::Io(err.to_string())
    }
}

/// Valida a integridade (SHA-256) e a autenticidade (Ed25519) de um pacote baixado.
///
/// A assinatura Ed25519 é verificada sobre o digest SHA-256 de 32 bytes do pacote.
pub fn verify_package_integrity(
    package_bytes: &[u8],
    expected_sha256: &[u8; 32],
    public_key_bytes: &[u8; 32],
    signature_bytes: &[u8],
) -> Result<(), UpdateError> {
    if signature_bytes.len() != 64 {
        return Err(UpdateError::InvalidSignature);
    }

    let mut sig_arr = [0u8; 64];
    sig_arr.copy_from_slice(signature_bytes);

    // 1. Calcula o SHA-256 do pacote recebido
    let mut hasher = Sha256::new();
    hasher.update(package_bytes);
    let calculated_hash: [u8; 32] = hasher.finalize().into();

    // 2. Compara em tempo constante o hash calculado contra o esperado
    if calculated_hash.ct_eq(expected_sha256).unwrap_u8() != 1 {
        return Err(UpdateError::HashMismatch);
    }

    // 3. Valida a chave pública Ed25519
    let verifying_key =
        VerifyingKey::from_bytes(public_key_bytes).map_err(|_| UpdateError::InvalidPublicKey)?;

    // 4. Constrói a assinatura Ed25519
    let signature = Signature::from_bytes(&sig_arr);

    // 5. Verifica a assinatura sobre o hash do pacote
    verifying_key
        .verify(expected_sha256, &signature)
        .map_err(|_| UpdateError::InvalidSignature)?;

    Ok(())
}

/// Guardião de backup para aplicação atômica de atualização com rollback em caso de falha.
pub struct UpdateBackupGuard {
    target_path: PathBuf,
    backup_path: PathBuf,
    committed: bool,
}

impl UpdateBackupGuard {
    /// Inicia o processo de substituição:
    /// 1. Cria backup do executável atual para `<alvo>.bak`
    /// 2. Grava o novo executável em `<alvo>`
    pub fn apply<P: AsRef<Path>>(target: P, new_binary_bytes: &[u8]) -> Result<Self, UpdateError> {
        let target_path = target.as_ref().to_path_buf();
        let backup_path = target_path.with_extension("bak");

        // Se o arquivo alvo original já existe, renomeia para backup
        if target_path.exists() {
            if backup_path.exists() {
                let _ = std::fs::remove_file(&backup_path);
            }
            std::fs::rename(&target_path, &backup_path)?;
        }

        // Grava o novo binário
        let mut new_file = File::create(&target_path)?;
        new_file.write_all(new_binary_bytes)?;
        new_file.flush()?;

        Ok(Self {
            target_path,
            backup_path,
            committed: false,
        })
    }

    /// Confirma que a nova versão iniciou com sucesso, descartando o backup.
    pub fn commit(mut self) -> Result<(), UpdateError> {
        self.committed = true;
        if self.backup_path.exists() {
            let _ = std::fs::remove_file(&self.backup_path);
        }
        Ok(())
    }

    /// Executa o rollback imediato restaurando o arquivo de backup.
    pub fn rollback(&mut self) -> Result<(), UpdateError> {
        if !self.committed && self.backup_path.exists() {
            if self.target_path.exists() {
                let _ = std::fs::remove_file(&self.target_path);
            }
            std::fs::rename(&self.backup_path, &self.target_path)?;
        }
        Ok(())
    }
}

impl Drop for UpdateBackupGuard {
    fn drop(&mut self) {
        if !self.committed {
            let _ = self.rollback();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};
    use rand::rngs::OsRng;

    #[test]
    fn test_verify_valid_signature_and_sha256() {
        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        let verifying_key = signing_key.verifying_key();

        let package = b"PrysmView Setup v0.2.0 binary package";
        let mut hasher = Sha256::new();
        hasher.update(package);
        let hash: [u8; 32] = hasher.finalize().into();

        let signature = signing_key.sign(&hash);

        let result = verify_package_integrity(
            package,
            &hash,
            verifying_key.as_bytes(),
            &signature.to_bytes(),
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_tampered_payload_rejected_by_hash() {
        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        let verifying_key = signing_key.verifying_key();

        let package = b"PrysmView Setup v0.2.0 binary package";
        let mut hasher = Sha256::new();
        hasher.update(package);
        let hash: [u8; 32] = hasher.finalize().into();
        let signature = signing_key.sign(&hash);

        // Pacote adulterado (1 byte alterado)
        let tampered_package = b"PrysmView Setup v0.2.0 binary packet!";

        let result = verify_package_integrity(
            tampered_package,
            &hash,
            verifying_key.as_bytes(),
            &signature.to_bytes(),
        );

        assert_eq!(result, Err(UpdateError::HashMismatch));
    }

    #[test]
    fn test_tampered_signature_rejected() {
        let mut csprng = OsRng;
        let signing_key = SigningKey::generate(&mut csprng);
        let verifying_key = signing_key.verifying_key();

        let package = b"PrysmView Setup v0.2.0 binary package";
        let mut hasher = Sha256::new();
        hasher.update(package);
        let hash: [u8; 32] = hasher.finalize().into();

        // Assinatura corrompida
        let mut bad_signature = signing_key.sign(&hash).to_bytes();
        bad_signature[0] ^= 0xFF;

        let result =
            verify_package_integrity(package, &hash, verifying_key.as_bytes(), &bad_signature);

        assert_eq!(result, Err(UpdateError::InvalidSignature));
    }

    #[test]
    fn test_rollback_guard_restores_original_file_on_failure() {
        let temp_dir =
            std::env::temp_dir().join(format!("prysm_update_test_{}", rand::random::<u32>()));
        std::fs::create_dir_all(&temp_dir).unwrap();

        let target_file = temp_dir.join("prysmview-svc.exe");
        std::fs::write(&target_file, b"VERSION 1.0.0").unwrap();

        // Inicia substituição
        {
            let guard =
                UpdateBackupGuard::apply(&target_file, b"VERSION 2.0.0 (CORRUPTED)").unwrap();
            assert_eq!(
                std::fs::read(&target_file).unwrap(),
                b"VERSION 2.0.0 (CORRUPTED)"
            );

            // Simula falha na inicialização da nova versão: o guard é descartado (Drop) sem commit
            drop(guard);
        }

        // Verifica que o rollback automático restaurou a versão original 1.0.0
        assert_eq!(std::fs::read(&target_file).unwrap(), b"VERSION 1.0.0");
        assert!(!target_file.with_extension("bak").exists());

        let _ = std::fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_rollback_guard_removes_backup_on_commit() {
        let temp_dir =
            std::env::temp_dir().join(format!("prysm_update_commit_{}", rand::random::<u32>()));
        std::fs::create_dir_all(&temp_dir).unwrap();

        let target_file = temp_dir.join("prysmview-svc.exe");
        std::fs::write(&target_file, b"VERSION 1.0.0").unwrap();

        // Inicia substituição e efetua commit
        let guard = UpdateBackupGuard::apply(&target_file, b"VERSION 2.0.0 (HEALTHY)").unwrap();
        guard.commit().unwrap();

        // Verifica que a nova versão permanece e o backup foi excluído
        assert_eq!(
            std::fs::read(&target_file).unwrap(),
            b"VERSION 2.0.0 (HEALTHY)"
        );
        assert!(!target_file.with_extension("bak").exists());

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
