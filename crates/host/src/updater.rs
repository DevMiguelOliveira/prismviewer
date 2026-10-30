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
use time::OffsetDateTime;

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
    pub package_size: u64,
    pub platform: String,
    pub expires_at: OffsetDateTime,
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

    #[error("Manifesto inválido ou adulterado (assinatura não confere)")]
    ManifestInvalid,

    #[error("Manifesto expirado em {0}")]
    Expired(String),

    #[error("Downgrade rejeitado: versão {version} abaixo da mínima aceita {min_version}")]
    DowngradeRejected {
        version: String,
        min_version: String,
    },

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

/// Campos do manifesto que são assinados (tudo exceto a assinatura própria).
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct ManifestToSign {
    version: String,
    channel: UpdateChannel,
    package_url: String,
    package_sha256: [u8; 32],
    package_size: u64,
    platform: String,
    expires_at: OffsetDateTime,
}

/// Valida a assinatura Ed25519 sobre o manifesto inteiro, a validade e a regra de anti-rollback.
///
/// A assinatura é verificada sobre os bytes serializados de `ManifestToSign` (sem o campo
/// `signature_ed25519`), de modo que qualquer alteração em versão, canal, hash, tamanho,
/// plataforma ou validade invalida a assinatura.
pub fn verify_manifest(
    manifest: &UpdateManifest,
    public_key_bytes: &[u8; 32],
    min_version_store: &dyn MinVersionStore,
) -> Result<(), UpdateError> {
    // 1. Valida a chave pública
    let verifying_key =
        VerifyingKey::from_bytes(public_key_bytes).map_err(|_| UpdateError::InvalidPublicKey)?;

    // 2. Reconstroi os bytes assinados (manifesto sem a assinatura)
    let to_sign = ManifestToSign {
        version: manifest.version.clone(),
        channel: manifest.channel,
        package_url: manifest.package_url.clone(),
        package_sha256: manifest.package_sha256,
        package_size: manifest.package_size,
        platform: manifest.platform.clone(),
        expires_at: manifest.expires_at,
    };
    let signed_bytes = serde_json::to_vec(&to_sign).map_err(|e| UpdateError::Io(e.to_string()))?;

    // 3. Constrói e verifica a assinatura Ed25519 sobre o manifesto inteiro
    if manifest.signature_ed25519.len() != 64 {
        return Err(UpdateError::InvalidSignature);
    }
    let mut sig_arr = [0u8; 64];
    sig_arr.copy_from_slice(&manifest.signature_ed25519);
    let signature = Signature::from_bytes(&sig_arr);
    verifying_key
        .verify(&signed_bytes, &signature)
        .map_err(|_| UpdateError::ManifestInvalid)?;

    // 4. Valida a validade (não expirou)
    let now = OffsetDateTime::now_utc();
    if now > manifest.expires_at {
        return Err(UpdateError::Expired(manifest.version.clone()));
    }

    // 5. Regra de anti-rollback: não aceita versão abaixo da mínima aceita localmente
    let min_version = min_version_store.read_min_version();
    if let Some(ref min) = min_version {
        if semver_lt(&manifest.version, min) {
            return Err(UpdateError::DowngradeRejected {
                version: manifest.version.clone(),
                min_version: min.clone(),
            });
        }
    }

    Ok(())
}

/// Comparação simples de versões semver (maior.menor.patch).
fn semver_lt(a: &str, b: &str) -> bool {
    let mut pa = a.split('.');
    let mut pb = b.split('.');
    loop {
        let va = pa.next().and_then(|s| s.parse::<u64>().ok()).unwrap_or(0);
        let vb = pb.next().and_then(|s| s.parse::<u64>().ok()).unwrap_or(0);
        if va != vb {
            return va < vb;
        }
        if pa.next().is_none() && pb.next().is_none() {
            return false;
        }
    }
}

/// Loja local da versão mínima aceita (anti-rollback).
///
/// Em produção no Windows, o arquivo é armazenado em diretório com ACL restrita
/// (somente SYSTEM e o usuário do serviço), impedindo que um atacante reduza
/// artificialmente a versão mínima aceita.
pub trait MinVersionStore {
    fn read_min_version(&self) -> Option<String>;
    fn write_min_version(&self, version: &str) -> std::io::Result<()>;
}

/// Implementação em arquivo de texto (um número de versão por linha).
#[derive(Debug, Clone)]
pub struct FileMinVersionStore {
    path: PathBuf,
}

impl FileMinVersionStore {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
}

impl MinVersionStore for FileMinVersionStore {
    fn read_min_version(&self) -> Option<String> {
        let content = std::fs::read_to_string(&self.path).ok()?;
        let trimmed = content.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    }

    fn write_min_version(&self, version: &str) -> std::io::Result<()> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&self.path, version)
    }
}

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

    fn make_manifest(
        version: &str,
        channel: UpdateChannel,
        expires_at: OffsetDateTime,
        signing_key: &SigningKey,
    ) -> UpdateManifest {
        let to_sign = ManifestToSign {
            version: version.to_string(),
            channel,
            package_url: "https://example.com/pkg.exe".to_string(),
            package_sha256: [0u8; 32],
            package_size: 1234,
            platform: "windows-x64".to_string(),
            expires_at,
        };
        let signed_bytes = serde_json::to_vec(&to_sign).unwrap();
        let signature = signing_key.sign(&signed_bytes);
        UpdateManifest {
            version: version.to_string(),
            channel,
            package_url: "https://example.com/pkg.exe".to_string(),
            package_sha256: [0u8; 32],
            package_size: 1234,
            platform: "windows-x64".to_string(),
            expires_at,
            signature_ed25519: signature.to_bytes().to_vec(),
        }
    }

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

    // ==================== S-01: Manifesto assinado ====================

    #[test]
    fn test_manifest_valid_accepted() {
        let signing_key = SigningKey::generate(&mut OsRng);
        let verifying_key = signing_key.verifying_key();
        let expires_at = OffsetDateTime::now_utc() + time::Duration::days(7);

        let manifest = make_manifest("2.0.0", UpdateChannel::Stable, expires_at, &signing_key);
        let store = FileMinVersionStore::new(std::env::temp_dir().join("nonexistent_min_v"));

        assert!(verify_manifest(&manifest, verifying_key.as_bytes(), &store).is_ok());
    }

    #[test]
    fn test_manifest_version_tampered_rejected() {
        let signing_key = SigningKey::generate(&mut OsRng);
        let verifying_key = signing_key.verifying_key();
        let expires_at = OffsetDateTime::now_utc() + time::Duration::days(7);

        let mut manifest = make_manifest("2.0.0", UpdateChannel::Stable, expires_at, &signing_key);
        // Adulterar a versão sem reassinar
        manifest.version = "9.9.9".to_string();

        let store = FileMinVersionStore::new(std::env::temp_dir().join("nonexistent_min_v"));
        assert_eq!(
            verify_manifest(&manifest, verifying_key.as_bytes(), &store),
            Err(UpdateError::ManifestInvalid)
        );
    }

    #[test]
    fn test_manifest_channel_tampered_rejected() {
        let signing_key = SigningKey::generate(&mut OsRng);
        let verifying_key = signing_key.verifying_key();
        let expires_at = OffsetDateTime::now_utc() + time::Duration::days(7);

        let mut manifest = make_manifest("2.0.0", UpdateChannel::Stable, expires_at, &signing_key);
        manifest.channel = UpdateChannel::Beta;

        let store = FileMinVersionStore::new(std::env::temp_dir().join("nonexistent_min_v"));
        assert_eq!(
            verify_manifest(&manifest, verifying_key.as_bytes(), &store),
            Err(UpdateError::ManifestInvalid)
        );
    }

    #[test]
    fn test_manifest_expired_rejected() {
        let signing_key = SigningKey::generate(&mut OsRng);
        let verifying_key = signing_key.verifying_key();
        let expires_at = OffsetDateTime::now_utc() - time::Duration::days(1);

        let manifest = make_manifest("2.0.0", UpdateChannel::Stable, expires_at, &signing_key);
        let store = FileMinVersionStore::new(std::env::temp_dir().join("nonexistent_min_v"));

        assert_eq!(
            verify_manifest(&manifest, verifying_key.as_bytes(), &store),
            Err(UpdateError::Expired(manifest.version.clone()))
        );
    }

    #[test]
    fn test_manifest_wrong_signature_key_rejected() {
        let signing_key_a = SigningKey::generate(&mut OsRng);
        let signing_key_b = SigningKey::generate(&mut OsRng);
        let verifying_key_b = signing_key_b.verifying_key();
        let expires_at = OffsetDateTime::now_utc() + time::Duration::days(7);

        let manifest = make_manifest("2.0.0", UpdateChannel::Stable, expires_at, &signing_key_a);
        let store = FileMinVersionStore::new(std::env::temp_dir().join("nonexistent_min_v"));

        assert_eq!(
            verify_manifest(&manifest, verifying_key_b.as_bytes(), &store),
            Err(UpdateError::ManifestInvalid)
        );
    }

    // ==================== S-02: Anti-rollback ====================

    #[test]
    fn test_manifest_downgrade_rejected() {
        let signing_key = SigningKey::generate(&mut OsRng);
        let verifying_key = signing_key.verifying_key();
        let expires_at = OffsetDateTime::now_utc() + time::Duration::days(7);

        // Versão mínima aceita localmente é 2.0.0
        let store = FileMinVersionStore::new(std::env::temp_dir().join("min_v_2"));
        store.write_min_version("2.0.0").unwrap();

        // Manifesto oferece 1.5.0 (downgrade)
        let manifest = make_manifest("1.5.0", UpdateChannel::Stable, expires_at, &signing_key);

        assert_eq!(
            verify_manifest(&manifest, verifying_key.as_bytes(), &store),
            Err(UpdateError::DowngradeRejected {
                version: "1.5.0".to_string(),
                min_version: "2.0.0".to_string(),
            })
        );
    }

    #[test]
    fn test_manifest_same_version_accepted() {
        let signing_key = SigningKey::generate(&mut OsRng);
        let verifying_key = signing_key.verifying_key();
        let expires_at = OffsetDateTime::now_utc() + time::Duration::days(7);

        let store = FileMinVersionStore::new(std::env::temp_dir().join("min_v_1"));
        store.write_min_version("1.0.0").unwrap();

        let manifest = make_manifest("1.0.0", UpdateChannel::Stable, expires_at, &signing_key);

        assert!(verify_manifest(&manifest, verifying_key.as_bytes(), &store).is_ok());
    }

    #[test]
    fn test_manifest_upgrade_accepted() {
        let signing_key = SigningKey::generate(&mut OsRng);
        let verifying_key = signing_key.verifying_key();
        let expires_at = OffsetDateTime::now_utc() + time::Duration::days(7);

        let store = FileMinVersionStore::new(std::env::temp_dir().join("min_v_1"));
        store.write_min_version("1.0.0").unwrap();

        let manifest = make_manifest("2.0.0", UpdateChannel::Stable, expires_at, &signing_key);

        assert!(verify_manifest(&manifest, verifying_key.as_bytes(), &store).is_ok());
    }

    #[test]
    fn test_file_min_version_store_roundtrip() {
        let temp_dir = std::env::temp_dir().join(format!("min_v_rt_{}", rand::random::<u32>()));
        let path = temp_dir.join("min_version.txt");
        let store = FileMinVersionStore::new(path.clone());

        assert_eq!(store.read_min_version(), None);

        store.write_min_version("3.0.0").unwrap();
        assert_eq!(store.read_min_version(), Some("3.0.0".to_string()));

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
