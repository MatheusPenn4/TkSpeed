//! tk-platform-win — wrappers seguros sobre a Windows API.
//! Centraliza TODO acesso ao sistema (Registry, etc.) num único lugar auditável,
//! evitando shell-out e injeção de comando.
//!
//! MVP: apenas o módulo `registry` (HKCU, sem elevação) está implementado —
//! suficiente para a operação-piloto reversível do TkRollback.

#[derive(Debug, thiserror::Error)]
pub enum PlatformError {
    #[error("operação requer elevação")]
    NeedsElevation,
    #[error("valor não encontrado")]
    NotFound,
    #[error("erro de API: {0}")]
    Api(String),
}

/// Acesso ao Registro do Windows (escopo HKEY_CURRENT_USER — sem UAC).
/// Sempre captura o valor anterior antes de qualquer escrita (base do rollback).
pub mod registry {
    #[cfg(windows)]
    mod imp {
        use super::super::PlatformError;
        use winreg::enums::*;
        use winreg::RegKey;

        fn is_not_found(e: &std::io::Error) -> bool {
            e.kind() == std::io::ErrorKind::NotFound
        }

        /// Lê um valor de string em HKCU\<subkey>\<name>. `None` se ausente.
        pub fn read_string(subkey: &str, name: &str) -> Result<Option<String>, PlatformError> {
            let hkcu = RegKey::predef(HKEY_CURRENT_USER);
            let key = match hkcu.open_subkey(subkey) {
                Ok(k) => k,
                Err(ref e) if is_not_found(e) => return Ok(None),
                Err(e) => return Err(PlatformError::Api(e.to_string())),
            };
            match key.get_value::<String, _>(name) {
                Ok(v) => Ok(Some(v)),
                Err(ref e) if is_not_found(e) => Ok(None),
                Err(e) => Err(PlatformError::Api(e.to_string())),
            }
        }

        /// Escreve (cria/atualiza) um valor de string em HKCU\<subkey>\<name>.
        pub fn write_string(subkey: &str, name: &str, value: &str) -> Result<(), PlatformError> {
            let hkcu = RegKey::predef(HKEY_CURRENT_USER);
            let (key, _) = hkcu
                .create_subkey(subkey)
                .map_err(|e| PlatformError::Api(e.to_string()))?;
            key.set_value(name, &value.to_string())
                .map_err(|e| PlatformError::Api(e.to_string()))
        }

        /// Remove um valor. Ausência é tratada como sucesso (idempotente).
        pub fn delete_value(subkey: &str, name: &str) -> Result<(), PlatformError> {
            let hkcu = RegKey::predef(HKEY_CURRENT_USER);
            let key = match hkcu.open_subkey_with_flags(subkey, KEY_SET_VALUE) {
                Ok(k) => k,
                Err(ref e) if is_not_found(e) => return Ok(()),
                Err(e) => return Err(PlatformError::Api(e.to_string())),
            };
            match key.delete_value(name) {
                Ok(()) => Ok(()),
                Err(ref e) if is_not_found(e) => Ok(()),
                Err(e) => Err(PlatformError::Api(e.to_string())),
            }
        }

        /// Lê um valor DWORD (REG_DWORD). `None` se ausente.
        pub fn read_u32(subkey: &str, name: &str) -> Result<Option<u32>, PlatformError> {
            let hkcu = RegKey::predef(HKEY_CURRENT_USER);
            let key = match hkcu.open_subkey(subkey) {
                Ok(k) => k,
                Err(ref e) if is_not_found(e) => return Ok(None),
                Err(e) => return Err(PlatformError::Api(e.to_string())),
            };
            match key.get_value::<u32, _>(name) {
                Ok(v) => Ok(Some(v)),
                Err(ref e) if is_not_found(e) => Ok(None),
                Err(e) => Err(PlatformError::Api(e.to_string())),
            }
        }

        /// Escreve um valor DWORD (REG_DWORD).
        pub fn write_u32(subkey: &str, name: &str, value: u32) -> Result<(), PlatformError> {
            let hkcu = RegKey::predef(HKEY_CURRENT_USER);
            let (key, _) = hkcu
                .create_subkey(subkey)
                .map_err(|e| PlatformError::Api(e.to_string()))?;
            key.set_value(name, &value).map_err(|e| PlatformError::Api(e.to_string()))
        }
    }

    #[cfg(not(windows))]
    mod imp {
        use super::super::PlatformError;
        fn unavailable() -> PlatformError {
            PlatformError::Api("registry indisponível fora do Windows".into())
        }
        pub fn read_string(_s: &str, _n: &str) -> Result<Option<String>, PlatformError> {
            Err(unavailable())
        }
        pub fn write_string(_s: &str, _n: &str, _v: &str) -> Result<(), PlatformError> {
            Err(unavailable())
        }
        pub fn delete_value(_s: &str, _n: &str) -> Result<(), PlatformError> {
            Err(unavailable())
        }
        pub fn read_u32(_s: &str, _n: &str) -> Result<Option<u32>, PlatformError> {
            Err(unavailable())
        }
        pub fn write_u32(_s: &str, _n: &str, _v: u32) -> Result<(), PlatformError> {
            Err(unavailable())
        }
    }

    pub use imp::{delete_value, read_string, read_u32, write_string, write_u32};
}

/// Leitura (somente) dos apps de inicialização — para a Análise de Startup.
pub mod startup {
    /// (nome, comando, origem) dos itens de Run (HKCU + HKLM).
    #[cfg(windows)]
    pub fn list() -> Vec<(String, String, String)> {
        use winreg::enums::*;
        use winreg::RegKey;
        let mut out = Vec::new();
        let run = "Software\\Microsoft\\Windows\\CurrentVersion\\Run";
        for (root, label) in [(HKEY_CURRENT_USER, "HKCU"), (HKEY_LOCAL_MACHINE, "HKLM")] {
            if let Ok(key) = RegKey::predef(root).open_subkey(run) {
                for item in key.enum_values().flatten() {
                    out.push((item.0, item.1.to_string(), label.to_string()));
                }
            }
        }
        out
    }

    #[cfg(not(windows))]
    pub fn list() -> Vec<(String, String, String)> {
        Vec::new()
    }
}

/// Plano de energia via `powercfg` (ferramenta nativa do Windows). Args fixos
/// (sem injeção). Trocar entre planos existentes não exige elevação. Reversível
/// (captura o GUID ativo anterior).
pub mod power {
    #[cfg(windows)]
    mod imp {
        use super::super::PlatformError;
        use std::process::Command;

        fn is_guid(t: &str) -> bool {
            t.len() == 36
                && t.as_bytes().iter().enumerate().all(|(i, &b)| {
                    if matches!(i, 8 | 13 | 18 | 23) {
                        b == b'-'
                    } else {
                        b.is_ascii_hexdigit()
                    }
                })
        }

        fn extract_guid(s: &str) -> Option<String> {
            s.split(|c: char| !(c.is_ascii_hexdigit() || c == '-'))
                .find(|t| is_guid(t))
                .map(|t| t.to_lowercase())
        }

        pub fn get_active_scheme() -> Result<String, PlatformError> {
            let out = Command::new("powercfg")
                .arg("/getactivescheme")
                .output()
                .map_err(|e| PlatformError::Api(format!("powercfg: {e}")))?;
            let s = String::from_utf8_lossy(&out.stdout);
            extract_guid(&s).ok_or(PlatformError::Api("GUID do plano ativo não encontrado".into()))
        }

        pub fn set_active_scheme(guid: &str) -> Result<(), PlatformError> {
            let st = Command::new("powercfg")
                .arg("/setactive")
                .arg(guid)
                .status()
                .map_err(|e| PlatformError::Api(format!("powercfg: {e}")))?;
            if st.success() {
                Ok(())
            } else {
                Err(PlatformError::Api("powercfg /setactive falhou (esquema inexistente?)".into()))
            }
        }

        pub fn scheme_exists(guid: &str) -> bool {
            Command::new("powercfg")
                .arg("/list")
                .output()
                .ok()
                .map(|o| String::from_utf8_lossy(&o.stdout).to_lowercase().contains(&guid.to_lowercase()))
                .unwrap_or(false)
        }
    }

    #[cfg(not(windows))]
    mod imp {
        use super::super::PlatformError;
        fn na() -> PlatformError {
            PlatformError::Api("power indisponível fora do Windows".into())
        }
        pub fn get_active_scheme() -> Result<String, PlatformError> {
            Err(na())
        }
        pub fn set_active_scheme(_g: &str) -> Result<(), PlatformError> {
            Err(na())
        }
        pub fn scheme_exists(_g: &str) -> bool {
            false
        }
    }

    pub use imp::{get_active_scheme, scheme_exists, set_active_scheme};
}

// Placeholders para fases futuras.
pub mod services {}
pub mod process {}
