use hmac::{Hmac, Mac};
use rand::rngs::OsRng;
use rand::Rng;
use sha2::Sha256;
use std::time::{Duration, Instant};
use subtle::ConstantTimeEq;
use thiserror::Error;
use zeroize::{Zeroize, ZeroizeOnDrop};

pub const SESSION_CODE_TTL: Duration = Duration::from_secs(120);
pub const MAX_SESSION_CODE_ATTEMPTS: u32 = 3;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SessionCodeError {
    #[error("Codigo de sessao expirado (validade de 120 segundos)")]
    Expired,

    #[error("Limite de tentativas excedido (maximo 3 tentativas)")]
    MaxAttemptsExceeded,

    #[error("Codigo de sessao invalido ({attempts_remaining} tentativas restantes)")]
    InvalidCode { attempts_remaining: u32 },

    #[error("Codigo de sessao ja foi verificado")]
    AlreadyVerified,

    #[error("Nenhum codigo de sessao foi gerado ainda (aguardando pedido valido)")]
    CodeNotGenerated,
}

/// Gerenciador de codigo de sessao de 8 digitos com verificacao HMAC em tempo constante.
/// O codigo so e gerado apos um pedido valido (S-04).
/// Limpa dados sensiveis da memoria ao ser descartado (ZeroizeOnDrop).
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct SessionCodeManager {
    code: String,
    hmac_key: [u8; 32],
    expected_hmac: [u8; 32],
    #[zeroize(skip)]
    created_at: Instant,
    #[zeroize(skip)]
    attempts: u32,
    #[zeroize(skip)]
    verified: bool,
    #[zeroize(skip)]
    generated: bool,
}

impl SessionCodeManager {
    /// Cria um gerenciador sem gerar o codigo.
    /// O codigo so e gerado via `generate_code()` apos um pedido valido (S-04).
    pub fn new() -> Self {
        let mut rng = OsRng;
        let mut hmac_key = [0u8; 32];
        rng.fill(&mut hmac_key);

        let mut mac =
            HmacSha256::new_from_slice(&hmac_key).expect("HMAC-SHA256 aceita chave de 32 bytes");
        mac.update(b"");
        let expected_hmac: [u8; 32] = mac.finalize().into_bytes().into();

        Self {
            code: String::new(),
            hmac_key,
            expected_hmac,
            created_at: Instant::now(),
            attempts: 0,
            verified: false,
            generated: false,
        }
    }

    /// Gera um novo codigo de 8 digitos criptograficamente seguro.
    /// Deve ser chamado apenas apos um pedido valido de sessao (S-04).
    pub fn generate_code(&mut self) {
        if self.generated {
            return; // ja gerado
        }
        let mut rng = OsRng;
        let num: u32 = rng.gen_range(10_000_000..=99_999_999);
        let code = num.to_string();

        let mut mac = HmacSha256::new_from_slice(&self.hmac_key)
            .expect("HMAC-SHA256 aceita chave de 32 bytes");
        mac.update(code.as_bytes());
        let expected_hmac: [u8; 32] = mac.finalize().into_bytes().into();

        self.code = code;
        self.expected_hmac = expected_hmac;
        self.created_at = Instant::now();
        self.attempts = 0;
        self.verified = false;
        self.generated = true;
    }

    /// Retorna o codigo de 8 digitos para ser exibido na janela do PDV.
    /// Erra se nenhum codigo foi gerado ainda.
    pub fn display_code(&self) -> Result<&str, SessionCodeError> {
        if !self.generated {
            return Err(SessionCodeError::CodeNotGenerated);
        }
        Ok(&self.code)
    }

    /// Retorna o tempo restante de validade em segundos.
    pub fn time_remaining(&self) -> Duration {
        SESSION_CODE_TTL.saturating_sub(self.created_at.elapsed())
    }

    /// Verifica se o codigo fornecido pelo operador eh valido usando comparacao em tempo constante.
    pub fn verify(&mut self, candidate: &str) -> Result<(), SessionCodeError> {
        if !self.generated {
            return Err(SessionCodeError::CodeNotGenerated);
        }
        if self.verified {
            return Err(SessionCodeError::AlreadyVerified);
        }

        if self.created_at.elapsed() > SESSION_CODE_TTL {
            return Err(SessionCodeError::Expired);
        }

        if self.attempts >= MAX_SESSION_CODE_ATTEMPTS {
            return Err(SessionCodeError::MaxAttemptsExceeded);
        }

        self.attempts += 1;

        // Calcula HMAC do candidato usando a chave protegida
        let mut mac = HmacSha256::new_from_slice(&self.hmac_key)
            .expect("HMAC-SHA256 aceita chave de 32 bytes");
        mac.update(candidate.as_bytes());
        let candidate_hmac: [u8; 32] = mac.finalize().into_bytes().into();

        // Comparacao em tempo constante para mitigar ataques de temporizacao
        let is_valid = candidate_hmac.ct_eq(&self.expected_hmac).into();

        if is_valid {
            self.verified = true;
            Ok(())
        } else {
            let attempts_remaining = MAX_SESSION_CODE_ATTEMPTS.saturating_sub(self.attempts);
            if attempts_remaining == 0 {
                Err(SessionCodeError::MaxAttemptsExceeded)
            } else {
                Err(SessionCodeError::InvalidCode { attempts_remaining })
            }
        }
    }

    /// Retorna true se o codigo foi verificado com sucesso.
    pub fn is_verified(&self) -> bool {
        self.verified
    }
}

impl Default for SessionCodeManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code_not_generated_before_request() {
        let mut mgr = SessionCodeManager::new();
        // Nenhum codigo deve estar disponivel antes de gerar
        assert_eq!(mgr.display_code(), Err(SessionCodeError::CodeNotGenerated));
        // Verificacao sem codigo gerado tambem falha
        assert_eq!(
            mgr.verify("12345678"),
            Err(SessionCodeError::CodeNotGenerated)
        );
    }

    #[test]
    fn test_generate_8_digit_code() {
        let mut mgr = SessionCodeManager::new();
        mgr.generate_code();
        let code = mgr.display_code().unwrap();
        assert_eq!(code.len(), 8);
        assert!(code.chars().all(|c| c.is_ascii_digit()));
        let num: u32 = code.parse().unwrap();
        assert!((10_000_000..=99_999_999).contains(&num));
    }

    #[test]
    fn test_valid_code_verification() {
        let mut mgr = SessionCodeManager::new();
        mgr.generate_code();
        let code = mgr.display_code().unwrap().to_string();

        let result = mgr.verify(&code);
        assert!(result.is_ok());
        assert!(mgr.is_verified());

        // Segunda tentativa deve falhar com AlreadyVerified
        assert_eq!(mgr.verify(&code), Err(SessionCodeError::AlreadyVerified));
    }

    #[test]
    fn test_invalid_code_decrements_attempts() {
        let mut mgr = SessionCodeManager::new();
        mgr.generate_code();

        let res1 = mgr.verify("00000000");
        assert_eq!(
            res1,
            Err(SessionCodeError::InvalidCode {
                attempts_remaining: 2
            })
        );

        let res2 = mgr.verify("11111111");
        assert_eq!(
            res2,
            Err(SessionCodeError::InvalidCode {
                attempts_remaining: 1
            })
        );

        let res3 = mgr.verify("22222222");
        assert_eq!(res3, Err(SessionCodeError::MaxAttemptsExceeded));

        // 4a tentativa bloqueada
        let res4 = mgr.verify("33333333");
        assert_eq!(res4, Err(SessionCodeError::MaxAttemptsExceeded));
    }

    #[test]
    fn test_expired_code() {
        let mut mgr = SessionCodeManager::new();
        mgr.generate_code();
        // Simula passagem de tempo acima de 120s
        mgr.created_at = Instant::now() - Duration::from_secs(121);

        let code = mgr.display_code().unwrap().to_string();
        let res = mgr.verify(&code);
        assert_eq!(res, Err(SessionCodeError::Expired));
    }

    #[test]
    fn test_time_remaining_calculation() {
        let mut mgr = SessionCodeManager::new();
        mgr.generate_code();
        let rem = mgr.time_remaining();
        assert!(rem <= SESSION_CODE_TTL);
        assert!(rem >= Duration::from_secs(115));
    }
}
