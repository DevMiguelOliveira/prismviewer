//! prysmview-host - Transferência segura de arquivos com verificação SHA-256
//!
//! Requisitos inegociáveis:
//! - Blocos de no máximo 1 MiB (`FILE_CHUNK_MAX = 1024 * 1024`);
//! - Gravação contínua (streaming) em arquivo temporário `.part` durante a recepção (sem carregar arquivo todo na RAM);
//! - Verificação integral do hash SHA-256 com comparação em tempo constante (`subtle::ConstantTimeEq`);
//! - Renomeação atômica para o nome de destino final somente após confirmação do hash;
//! - Remoção imediata do arquivo temporário caso haja divergência no hash ou cancelamento/queda;
//! - Capacidade de desativação local pelo operador do PDV.

use prysmview_core::limits::MAX_FILE_PAYLOAD;

pub const FILE_CHUNK_MAX: usize = MAX_FILE_PAYLOAD as usize;
use prysmview_core::messages::FileBlockMessage;
use sha2::{Digest, Sha256};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use subtle::ConstantTimeEq;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FileTransferError {
    #[error("Erro de I/O em arquivo: {0}")]
    Io(#[from] io::Error),
    #[error("Transferência de arquivos desativada pela política local do PDV")]
    DisabledByPos,
    #[error("Bloco recebido fora de ordem: esperado bloco {expected}, recebido {actual}")]
    OutOfOrderBlock { expected: u32, actual: u32 },
    #[error("ID de transferência divergente: esperado {expected}, recebido {actual}")]
    TransferIdMismatch { expected: u32, actual: u32 },
    #[error("Tamanho do bloco excede limite de 1 MiB: {size} bytes")]
    BlockSizeExceeded { size: usize },
    #[error("Falha na validação de integridade: Hash SHA-256 não confere")]
    HashMismatch,
    #[error("Transferência já finalizada")]
    AlreadyCompleted,
}

/// Receptor de transferência de arquivo que grava em streaming no arquivo temporário `.part` e valida SHA-256.
pub struct FileTransferReceiver {
    pub target_path: PathBuf,
    pub temp_path: PathBuf,
    pub transfer_id: u32,
    pub total_blocks: u32,
    pub blocks_received: u32,
    pub expected_sha256: [u8; 32],
    pub enabled: bool,
    pub is_completed: bool,
    hasher: Sha256,
    file_handle: Option<File>,
}

impl FileTransferReceiver {
    /// Inicia uma nova sessão de recepção de arquivo.
    pub fn new(
        target_path: PathBuf,
        transfer_id: u32,
        total_blocks: u32,
        expected_sha256: [u8; 32],
        enabled: bool,
    ) -> Result<Self, FileTransferError> {
        if !enabled {
            return Err(FileTransferError::DisabledByPos);
        }

        // Gera caminho temporário com extensão .part
        let temp_file_name = format!(
            "{}.{}.part",
            target_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("transfer"),
            transfer_id
        );
        let temp_path = target_path.with_file_name(temp_file_name);

        // Garante que o diretório pai existe
        if let Some(parent) = temp_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&temp_path)?;

        Ok(Self {
            target_path,
            temp_path,
            transfer_id,
            total_blocks,
            blocks_received: 0,
            expected_sha256,
            enabled,
            is_completed: false,
            hasher: Sha256::new(),
            file_handle: Some(file),
        })
    }

    /// Escreve um bloco recebido no arquivo temporário e atualiza o hash SHA-256.
    /// Retorna `Ok(true)` se este foi o último bloco e o arquivo foi verificado com sucesso.
    pub fn write_block(&mut self, block: &FileBlockMessage) -> Result<bool, FileTransferError> {
        if !self.enabled {
            self.cleanup_temp();
            return Err(FileTransferError::DisabledByPos);
        }

        if self.is_completed {
            return Err(FileTransferError::AlreadyCompleted);
        }

        if block.transfer_id != self.transfer_id {
            return Err(FileTransferError::TransferIdMismatch {
                expected: self.transfer_id,
                actual: block.transfer_id,
            });
        }

        if block.block_index != self.blocks_received {
            return Err(FileTransferError::OutOfOrderBlock {
                expected: self.blocks_received,
                actual: block.block_index,
            });
        }

        if block.data.len() > FILE_CHUNK_MAX {
            return Err(FileTransferError::BlockSizeExceeded {
                size: block.data.len(),
            });
        }

        let file = match self.file_handle.as_mut() {
            Some(f) => f,
            None => {
                return Err(FileTransferError::Io(io::Error::other(
                    "Handle do arquivo temporário indisponível",
                )))
            }
        };

        // Grava os dados do bloco
        file.write_all(&block.data)?;
        self.hasher.update(&block.data);
        self.blocks_received += 1;

        // Se for o último bloco, valida o SHA-256 e renomeia
        if self.blocks_received == self.total_blocks {
            file.flush()?;
            // Fecha o arquivo antes de renomear
            self.file_handle = None;

            let computed_hash = self.hasher.clone().finalize();
            let hash_bytes: [u8; 32] = computed_hash.into();

            // Comparação estrita em tempo constante para integridade e segurança
            let is_valid = hash_bytes.ct_eq(&self.expected_sha256);

            if is_valid.into() {
                // Renomeação atômica do arquivo temporário para o destino final
                fs::rename(&self.temp_path, &self.target_path)?;
                self.is_completed = true;
                Ok(true)
            } else {
                // Falha de integridade: descarta arquivo parcial imediatamente
                self.cleanup_temp();
                Err(FileTransferError::HashMismatch)
            }
        } else {
            Ok(false)
        }
    }

    /// Limpa o arquivo temporário caso exista.
    fn cleanup_temp(&mut self) {
        self.file_handle = None;
        if self.temp_path.exists() {
            let _ = fs::remove_file(&self.temp_path);
        }
    }

    /// Cancela a transferência e remove o arquivo temporário.
    pub fn cancel(&mut self) {
        self.cleanup_temp();
    }
}

impl Drop for FileTransferReceiver {
    fn drop(&mut self) {
        // Se a transferência não foi concluída com sucesso, descarta qualquer arquivo temporário
        if !self.is_completed {
            self.cleanup_temp();
        }
    }
}

/// Emissor que lê um arquivo e gera blocos de até 1 MiB (`FileBlockMessage`) com hash SHA-256.
pub struct FileTransferSender;

impl FileTransferSender {
    /// Calcula o hash SHA-256 completo de um arquivo local.
    pub fn compute_file_sha256<P: AsRef<Path>>(path: P) -> Result<[u8; 32], io::Error> {
        let mut file = File::open(path)?;
        let mut hasher = Sha256::new();
        let mut buffer = [0u8; 64 * 1024];

        loop {
            let bytes_read = file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }

        let hash = hasher.finalize();
        Ok(hash.into())
    }

    /// Lê um bloco específico do arquivo para envio pela rede.
    pub fn read_block<P: AsRef<Path>>(
        path: P,
        transfer_id: u32,
        block_index: u32,
        total_blocks: u32,
        file_sha256: [u8; 32],
    ) -> Result<FileBlockMessage, io::Error> {
        let mut file = File::open(path)?;
        let offset = (block_index as u64) * (FILE_CHUNK_MAX as u64);
        file.seek(SeekFrom::Start(offset))?;

        let mut data = vec![0u8; FILE_CHUNK_MAX];
        let bytes_read = file.read(&mut data)?;
        data.truncate(bytes_read);

        Ok(FileBlockMessage {
            transfer_id,
            block_index,
            total_blocks,
            offset,
            data,
            sha256: file_sha256,
        })
    }

    /// Retorna o número total de blocos necessários para transferir um arquivo.
    pub fn calculate_total_blocks(file_size: u64) -> u32 {
        if file_size == 0 {
            1
        } else {
            let chunk = FILE_CHUNK_MAX as u64;
            file_size.div_ceil(chunk) as u32
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_transfer_successful_multi_block_roundtrip() {
        let temp_dir = std::env::temp_dir().join(format!("prysm_test_{}", rand::random::<u32>()));
        fs::create_dir_all(&temp_dir).expect("cria temp dir");

        let target_file = temp_dir.join("test_output.bin");
        let transfer_id = 42;

        // Cria dados de teste com 2 blocos (ex: 2.5 KB)
        let block1_data = vec![0xAA; 1024];
        let block2_data = vec![0xBB; 1500];

        let mut hasher = Sha256::new();
        hasher.update(&block1_data);
        hasher.update(&block2_data);
        let expected_sha256: [u8; 32] = hasher.finalize().into();

        let mut receiver =
            FileTransferReceiver::new(target_file.clone(), transfer_id, 2, expected_sha256, true)
                .expect("cria receiver");

        let block0 = FileBlockMessage {
            transfer_id,
            block_index: 0,
            total_blocks: 2,
            offset: 0,
            data: block1_data.clone(),
            sha256: expected_sha256,
        };

        let done0 = receiver.write_block(&block0).expect("bloco 0");
        assert!(!done0);
        assert!(receiver.temp_path.exists());
        assert!(!target_file.exists());

        let block1 = FileBlockMessage {
            transfer_id,
            block_index: 1,
            total_blocks: 2,
            offset: 1024,
            data: block2_data.clone(),
            sha256: expected_sha256,
        };

        let done1 = receiver.write_block(&block1).expect("bloco 1");
        assert!(done1);
        assert!(target_file.exists());
        assert!(!receiver.temp_path.exists());

        // Confere conteúdo final do arquivo
        let mut final_content = Vec::new();
        File::open(&target_file)
            .expect("abre final")
            .read_to_end(&mut final_content)
            .expect("le final");

        let mut expected_content = block1_data;
        expected_content.extend_from_slice(&block2_data);
        assert_eq!(final_content, expected_content);

        // Limpeza
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_file_transfer_hash_mismatch_removes_temp_file() {
        let temp_dir = std::env::temp_dir().join(format!("prysm_test_{}", rand::random::<u32>()));
        fs::create_dir_all(&temp_dir).expect("cria temp dir");
        let target_file = temp_dir.join("tampered_output.bin");

        let fake_sha256 = [0xFF; 32];
        let mut receiver =
            FileTransferReceiver::new(target_file.clone(), 100, 1, fake_sha256, true)
                .expect("cria receiver");

        let block = FileBlockMessage {
            transfer_id: 100,
            block_index: 0,
            total_blocks: 1,
            offset: 0,
            data: vec![1, 2, 3, 4],
            sha256: fake_sha256,
        };

        let res = receiver.write_block(&block);
        assert!(matches!(res, Err(FileTransferError::HashMismatch)));
        assert!(!receiver.temp_path.exists());
        assert!(!target_file.exists());

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_file_transfer_drop_cleans_up_partial_file() {
        let temp_dir = std::env::temp_dir().join(format!("prysm_test_{}", rand::random::<u32>()));
        fs::create_dir_all(&temp_dir).expect("cria temp dir");
        let target_file = temp_dir.join("dropped_output.bin");

        let temp_path_copy;
        {
            let mut receiver =
                FileTransferReceiver::new(target_file.clone(), 200, 2, [0; 32], true)
                    .expect("cria receiver");
            temp_path_copy = receiver.temp_path.clone();

            let block = FileBlockMessage {
                transfer_id: 200,
                block_index: 0,
                total_blocks: 2,
                offset: 0,
                data: vec![42; 500],
                sha256: [0; 32],
            };
            receiver.write_block(&block).expect("escreve bloco 0");
            assert!(temp_path_copy.exists());
            // receiver sai de escopo e é destruído aqui (Drop)
        }

        assert!(
            !temp_path_copy.exists(),
            "Arquivo parcial .part deve ser deletado no Drop"
        );
        assert!(!target_file.exists());

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_file_transfer_disabled_by_pos_policy() {
        let target = PathBuf::from("test.bin");
        let res = FileTransferReceiver::new(target, 1, 1, [0; 32], false);
        assert!(matches!(res, Err(FileTransferError::DisabledByPos)));
    }

    #[test]
    fn test_out_of_order_block_rejected() {
        let temp_dir = std::env::temp_dir().join(format!("prysm_test_{}", rand::random::<u32>()));
        fs::create_dir_all(&temp_dir).expect("cria temp dir");
        let target_file = temp_dir.join("order_test.bin");

        let mut receiver =
            FileTransferReceiver::new(target_file, 300, 3, [0; 32], true).expect("cria receiver");

        let wrong_block = FileBlockMessage {
            transfer_id: 300,
            block_index: 1, // Deveria ser 0
            total_blocks: 3,
            offset: 1024,
            data: vec![0; 100],
            sha256: [0; 32],
        };

        let res = receiver.write_block(&wrong_block);
        assert!(matches!(
            res,
            Err(FileTransferError::OutOfOrderBlock {
                expected: 0,
                actual: 1
            })
        ));

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_calculate_total_blocks() {
        assert_eq!(FileTransferSender::calculate_total_blocks(0), 1);
        assert_eq!(FileTransferSender::calculate_total_blocks(500), 1);
        assert_eq!(
            FileTransferSender::calculate_total_blocks(FILE_CHUNK_MAX as u64),
            1
        );
        assert_eq!(
            FileTransferSender::calculate_total_blocks((FILE_CHUNK_MAX as u64) + 1),
            2
        );
    }
}
