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
        use std::os::windows::process::CommandExt;
        use std::process::Command;

        const CREATE_NO_WINDOW: u32 = 0x08000000;

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
                .creation_flags(CREATE_NO_WINDOW)
                .output()
                .map_err(|e| PlatformError::Api(format!("powercfg: {e}")))?;
            let s = String::from_utf8_lossy(&out.stdout);
            extract_guid(&s).ok_or(PlatformError::Api("GUID do plano ativo não encontrado".into()))
        }

        pub fn set_active_scheme(guid: &str) -> Result<(), PlatformError> {
            let out = Command::new("powercfg")
                .arg("/setactive")
                .arg(guid)
                .creation_flags(CREATE_NO_WINDOW)
                .output()
                .map_err(|e| PlatformError::Api(format!("powercfg: {e}")))?;
            if out.status.success() {
                Ok(())
            } else {
                let detail = {
                    let s = String::from_utf8_lossy(&out.stderr);
                    let o = String::from_utf8_lossy(&out.stdout);
                    let raw = if !s.trim().is_empty() { s } else { o };
                    let t = raw.trim().to_string();
                    if t.is_empty() { format!("código {}", out.status.code().unwrap_or(-1)) } else { t }
                };
                Err(PlatformError::Api(format!("powercfg /setactive falhou: {detail}")))
            }
        }

        pub fn scheme_exists(guid: &str) -> bool {
            Command::new("powercfg")
                .arg("/list")
                .creation_flags(CREATE_NO_WINDOW)
                .output()
                .ok()
                .map(|o| String::from_utf8_lossy(&o.stdout).to_lowercase().contains(&guid.to_lowercase()))
                .unwrap_or(false)
        }

        /// Garante que o plano com `guid` esteja disponível e ativo.
        /// 1. Se já existe → retorna Ok sem alterar nada.
        /// 2. Tenta ativar diretamente (plano oculto em algumas edições).
        /// 3. Tenta `powercfg /duplicatescheme <guid>` para restaurar o plano padrão.
        /// 4. Se tudo falhar → Err com mensagem honesta.
        pub fn ensure_scheme(guid: &str) -> Result<(), PlatformError> {
            if scheme_exists(guid) {
                return Ok(());
            }
            if set_active_scheme(guid).is_ok() {
                return Ok(());
            }
            let st = Command::new("powercfg")
                .args(["/duplicatescheme", guid])
                .creation_flags(CREATE_NO_WINDOW)
                .status()
                .map_err(|e| PlatformError::Api(format!("powercfg: {e}")))?;
            if st.success() {
                Ok(())
            } else {
                Err(PlatformError::Api(format!(
                    "Plano de energia {} não disponível nesta instalação.",
                    guid
                )))
            }
        }

        /// Lista todos os planos de energia disponíveis: Vec<(guid, nome)>.
        /// Parseia a saída de `powercfg /list` — funciona em qualquer idioma ou OEM.
        pub fn list_schemes() -> Vec<(String, String)> {
            let out = match Command::new("powercfg")
                .arg("/list")
                .creation_flags(CREATE_NO_WINDOW)
                .output()
            {
                Ok(o) => o,
                Err(_) => return vec![],
            };
            let text = String::from_utf8_lossy(&out.stdout);
            let mut result = Vec::new();
            for line in text.lines() {
                if let Some(guid) = extract_guid(line) {
                    // Nome entre parênteses: "Power Scheme GUID: xxxx  (Nome do Plano) *"
                    let name = line
                        .find('(')
                        .and_then(|i| {
                            line[i + 1..].find(')').map(|j| line[i + 1..i + 1 + j].trim().to_string())
                        })
                        .unwrap_or_default();
                    result.push((guid, name));
                }
            }
            result
        }

        /// Pontua o quanto um nome de plano corresponde a "alto desempenho".
        /// 3 = Ultimate/Máximo, 2 = High/Alto Desempenho, 1 = Desempenho/Performance genérico.
        fn high_perf_score(name: &str) -> u8 {
            let l = name.to_lowercase();
            if l.contains("ultimate")
                || l.contains("m\u{00e1}ximo")   // máximo
                || l.contains("maximo")
                || l.contains("maximum")
                || (l.contains("desempenho") && (l.contains("m\u{00e1}x") || l.contains("max")))
            {
                return 3;
            }
            if l.contains("high performance")
                || l.contains("alto desempenho")
                || (l.contains("high") && l.contains("perf"))
                || (l.contains("alto") && l.contains("desempenho"))
            {
                return 2;
            }
            if l.contains("desempenho") || l.contains("performance") {
                return 1;
            }
            0
        }

        /// Encontra o plano de alto desempenho existente com maior pontuação.
        /// Retorna `(guid, nome)` ou `None` se nenhum plano de desempenho for encontrado.
        pub fn find_high_perf_scheme() -> Option<(String, String)> {
            list_schemes()
                .into_iter()
                .filter(|(_, name)| high_perf_score(name) > 0)
                .max_by_key(|(_, name)| high_perf_score(name))
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
        pub fn ensure_scheme(_g: &str) -> Result<(), PlatformError> {
            Err(na())
        }
        pub fn list_schemes() -> Vec<(String, String)> {
            vec![]
        }
        pub fn find_high_perf_scheme() -> Option<(String, String)> {
            None
        }
    }

    pub use imp::{ensure_scheme, find_high_perf_scheme, get_active_scheme, list_schemes, scheme_exists, set_active_scheme};
}

/// Detecção de elevação (privilégios de administrador).
/// Usado pelo CapabilityMatrix para expor `admin_privileges`.
pub mod elevation {
    /// Retorna true se o processo atual está rodando com privilégios de administrador.
    ///
    /// Implementação: usa GetTokenInformation(TokenElevation) via WinAPI — método
    /// recomendado pela Microsoft, sem dependência de ACLs de chaves específicas.
    #[cfg(windows)]
    pub fn is_elevated() -> bool {
        use std::mem;

        #[repr(C)]
        struct TokenElevation {
            token_is_elevated: u32,
        }

        #[link(name = "kernel32")]
        extern "system" {
            fn GetCurrentProcess() -> isize;
            fn CloseHandle(h: isize) -> i32;
        }

        #[link(name = "advapi32")]
        extern "system" {
            fn OpenProcessToken(process: isize, desired_access: u32, token: *mut isize) -> i32;
            fn GetTokenInformation(
                token: isize,
                class: u32,
                info: *mut core::ffi::c_void,
                length: u32,
                ret_len: *mut u32,
            ) -> i32;
        }

        const TOKEN_QUERY: u32 = 0x0008;
        const TOKEN_ELEVATION_CLASS: u32 = 20;

        unsafe {
            let mut token: isize = 0;
            if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) == 0 {
                return false;
            }
            let mut elev = TokenElevation { token_is_elevated: 0 };
            let mut ret_len: u32 = 0;
            let ok = GetTokenInformation(
                token,
                TOKEN_ELEVATION_CLASS,
                &mut elev as *mut _ as *mut core::ffi::c_void,
                mem::size_of::<TokenElevation>() as u32,
                &mut ret_len,
            );
            CloseHandle(token);
            ok != 0 && elev.token_is_elevated != 0
        }
    }

    #[cfg(not(windows))]
    pub fn is_elevated() -> bool {
        false
    }
}

/// Registro no HKLM (HKEY_LOCAL_MACHINE). Escrita requer privilégios de administrador.
pub mod registry_hklm {
    #[cfg(windows)]
    mod imp {
        use super::super::PlatformError;
        use winreg::enums::*;
        use winreg::RegKey;

        fn is_not_found(e: &std::io::Error) -> bool {
            e.kind() == std::io::ErrorKind::NotFound
        }

        pub fn read_u32(subkey: &str, name: &str) -> Result<Option<u32>, PlatformError> {
            let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
            let key = match hklm.open_subkey(subkey) {
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

        pub fn write_u32(subkey: &str, name: &str, value: u32) -> Result<(), PlatformError> {
            let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
            let (key, _) = hklm
                .create_subkey(subkey)
                .map_err(|e| PlatformError::Api(e.to_string()))?;
            key.set_value(name, &value).map_err(|e| PlatformError::Api(e.to_string()))
        }

        pub fn delete_value(subkey: &str, name: &str) -> Result<(), PlatformError> {
            let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
            let key = match hklm.open_subkey_with_flags(subkey, KEY_SET_VALUE) {
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

        pub fn read_string(subkey: &str, name: &str) -> Result<Option<String>, PlatformError> {
            let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
            let key = match hklm.open_subkey(subkey) {
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

        pub fn write_string(subkey: &str, name: &str, value: &str) -> Result<(), PlatformError> {
            let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
            let (key, _) = hklm
                .create_subkey(subkey)
                .map_err(|e| PlatformError::Api(e.to_string()))?;
            key.set_value(name, &value.to_string())
                .map_err(|e| PlatformError::Api(e.to_string()))
        }

        /// Lista os nomes dos subkeys diretos de HKLM\<subkey>. Retorna [] se ausente.
        pub fn enumerate_subkeys(subkey: &str) -> Result<Vec<String>, PlatformError> {
            let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
            let key = match hklm.open_subkey(subkey) {
                Ok(k) => k,
                Err(ref e) if is_not_found(e) => return Ok(vec![]),
                Err(e) => return Err(PlatformError::Api(e.to_string())),
            };
            Ok(key.enum_keys().filter_map(|r| r.ok()).collect())
        }
    }

    #[cfg(not(windows))]
    mod imp {
        use super::super::PlatformError;
        fn na() -> PlatformError {
            PlatformError::Api("HKLM indisponível fora do Windows".into())
        }
        pub fn read_u32(_s: &str, _n: &str) -> Result<Option<u32>, PlatformError> { Err(na()) }
        pub fn write_u32(_s: &str, _n: &str, _v: u32) -> Result<(), PlatformError> { Err(na()) }
        pub fn delete_value(_s: &str, _n: &str) -> Result<(), PlatformError> { Err(na()) }
        pub fn read_string(_s: &str, _n: &str) -> Result<Option<String>, PlatformError> { Err(na()) }
        pub fn write_string(_s: &str, _n: &str, _v: &str) -> Result<(), PlatformError> { Err(na()) }
        pub fn enumerate_subkeys(_s: &str) -> Result<Vec<String>, PlatformError> { Ok(vec![]) }
    }

    pub use imp::{delete_value, enumerate_subkeys, read_string, read_u32, write_string, write_u32};
}

/// Medição e liberação de memória em espera (standby list).
pub mod memory {
    /// Bytes de memória física disponível (RAM livre, inclui standby recuperável).
    pub fn available_bytes() -> u64 {
        #[cfg(windows)]
        return imp::available_bytes();
        #[cfg(not(windows))]
        return 0;
    }

    /// Libera a standby list via NtSetSystemInformation.
    /// Requer administrador. Retorna bytes liberados (diferença de available antes/depois).
    pub fn flush_standby() -> Result<u64, String> {
        #[cfg(windows)]
        return imp::flush_standby();
        #[cfg(not(windows))]
        return Err("Indisponível fora do Windows.".into());
    }

    #[cfg(windows)]
    mod imp {
        use std::mem;

        #[repr(C)]
        struct MemoryStatusEx {
            dw_length: u32,
            dw_memory_load: u32,
            ull_total_phys: u64,
            ull_avail_phys: u64,
            ull_total_page_file: u64,
            ull_avail_page_file: u64,
            ull_total_virtual: u64,
            ull_avail_virtual: u64,
            ull_avail_extended_virtual: u64,
        }

        #[link(name = "kernel32")]
        extern "system" {
            fn GlobalMemoryStatusEx(lp: *mut MemoryStatusEx) -> i32;
            fn GetCurrentProcess() -> isize;
            fn CloseHandle(h: isize) -> i32;
        }

        #[link(name = "ntdll")]
        extern "system" {
            fn NtSetSystemInformation(class: u32, info: *mut u32, len: u32) -> i32;
        }

        // Estruturas e APIs de token para habilitar SeProfileSingleProcessPrivilege.
        #[repr(C)]
        struct Luid {
            low_part: u32,
            high_part: i32,
        }

        #[repr(C)]
        struct LuidAndAttributes {
            luid: Luid,
            attributes: u32,
        }

        #[repr(C)]
        struct TokenPrivileges {
            privilege_count: u32,
            privileges: [LuidAndAttributes; 1],
        }

        #[link(name = "advapi32")]
        extern "system" {
            fn OpenProcessToken(process: isize, desired_access: u32, token: *mut isize) -> i32;
            fn LookupPrivilegeValueW(
                system_name: *const u16,
                name: *const u16,
                luid: *mut Luid,
            ) -> i32;
            fn AdjustTokenPrivileges(
                token: isize,
                disable_all: i32,
                new_state: *const TokenPrivileges,
                buffer_length: u32,
                previous_state: *mut TokenPrivileges,
                return_length: *mut u32,
            ) -> i32;
        }

        const TOKEN_ADJUST_PRIVILEGES: u32 = 0x0020;
        const TOKEN_QUERY: u32 = 0x0008;
        const SE_PRIVILEGE_ENABLED: u32 = 0x0002;

        pub fn available_bytes() -> u64 {
            let mut s: MemoryStatusEx = unsafe { mem::zeroed() };
            s.dw_length = mem::size_of::<MemoryStatusEx>() as u32;
            unsafe { GlobalMemoryStatusEx(&mut s) };
            s.ull_avail_phys
        }

        /// Habilita SeProfileSingleProcessPrivilege no token do processo atual.
        /// O privilégio existe num processo elevado, mas costuma vir desabilitado;
        /// NtSetSystemInformation(MemoryPurgeStandbyList) exige-o habilitado.
        /// Best-effort: erros são ignorados aqui — o resultado real é validado pelo
        /// NTSTATUS da chamada subsequente, que diferencia privilégio de acesso negado.
        fn enable_profile_privilege() {
            // "SeProfileSingleProcessPrivilege" como UTF-16 terminado em nul.
            let name: Vec<u16> = "SeProfileSingleProcessPrivilege"
                .encode_utf16()
                .chain(std::iter::once(0))
                .collect();
            unsafe {
                let mut token: isize = 0;
                if OpenProcessToken(
                    GetCurrentProcess(),
                    TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY,
                    &mut token,
                ) == 0
                {
                    return;
                }
                let mut luid = Luid { low_part: 0, high_part: 0 };
                if LookupPrivilegeValueW(std::ptr::null(), name.as_ptr(), &mut luid) != 0 {
                    let tp = TokenPrivileges {
                        privilege_count: 1,
                        privileges: [LuidAndAttributes { luid, attributes: SE_PRIVILEGE_ENABLED }],
                    };
                    AdjustTokenPrivileges(
                        token,
                        0,
                        &tp,
                        0,
                        std::ptr::null_mut(),
                        std::ptr::null_mut(),
                    );
                }
                CloseHandle(token);
            }
        }

        pub fn flush_standby() -> Result<u64, String> {
            const SYSTEM_MEMORY_LIST_INFORMATION: u32 = 0x50;
            const MEMORY_PURGE_STANDBY_LIST: u32 = 4;

            // Habilita o privilégio necessário ANTES da syscall (correção da falha
            // de "admin" mesmo com o processo elevado).
            enable_profile_privilege();

            let before = available_bytes();
            let mut cmd: u32 = MEMORY_PURGE_STANDBY_LIST;
            let status = unsafe {
                NtSetSystemInformation(
                    SYSTEM_MEMORY_LIST_INFORMATION,
                    &mut cmd,
                    mem::size_of::<u32>() as u32,
                )
            };
            if status < 0 {
                // Mensagens honestas por código — não converter tudo em "Requer administrador".
                let msg = match status as u32 {
                    0xC0000061 => "O Windows não permitiu habilitar o privilégio necessário para liberar memória em espera.".to_string(),
                    0xC0000022 => "Acesso negado pelo Windows ao liberar memória em espera.".to_string(),
                    0xC000000D => "Parâmetro inválido ao solicitar limpeza de memória em espera.".to_string(),
                    other => format!("Falha ao liberar memória em espera (código 0x{other:08X})."),
                };
                return Err(msg);
            }
            let after = available_bytes();
            Ok(after.saturating_sub(before))
        }
    }
}

/// Detecção e ajuste de prioridade de processos de jogos.
pub mod process_win {
    /// Processo de jogo detectado.
    #[derive(Debug, Clone)]
    pub struct GameProcess {
        pub pid: u32,
        pub name: String,
    }

    /// Eleva a prioridade de um processo para AboveNormal.
    /// Não usa Realtime (proibido pelo design).
    pub fn set_above_normal_priority(pid: u32) -> Result<(), String> {
        #[cfg(windows)]
        return imp::set_priority(pid, 0x00008000); // ABOVE_NORMAL_PRIORITY_CLASS
        #[cfg(not(windows))]
        { let _ = pid; Err("Indisponível fora do Windows.".into()) }
    }

    /// Restaura prioridade para Normal.
    pub fn set_normal_priority(pid: u32) -> Result<(), String> {
        #[cfg(windows)]
        return imp::set_priority(pid, 0x00000020); // NORMAL_PRIORITY_CLASS
        #[cfg(not(windows))]
        { let _ = pid; Err("Indisponível fora do Windows.".into()) }
    }

    #[cfg(windows)]
    mod imp {
        #[link(name = "kernel32")]
        extern "system" {
            fn OpenProcess(desired_access: u32, inherit_handle: i32, process_id: u32) -> isize;
            fn SetPriorityClass(h_process: isize, dw_priority_class: u32) -> i32;
            fn CloseHandle(h_object: isize) -> i32;
        }

        const PROCESS_SET_INFORMATION: u32 = 0x0200;
        const PROCESS_QUERY_LIMITED_INFORMATION: u32 = 0x1000;

        pub fn set_priority(pid: u32, class: u32) -> Result<(), String> {
            unsafe {
                let handle = OpenProcess(
                    PROCESS_SET_INFORMATION | PROCESS_QUERY_LIMITED_INFORMATION,
                    0,
                    pid,
                );
                if handle == 0 {
                    return Err(
                        "Processo não encontrado ou acesso negado — pode requerir administrador."
                            .into(),
                    );
                }
                let ok = SetPriorityClass(handle, class);
                CloseHandle(handle);
                if ok == 0 {
                    Err("Falha ao definir prioridade do processo.".into())
                } else {
                    Ok(())
                }
            }
        }
    }
}

// Placeholders para fases futuras.
pub mod services {}

/// Detecção de jogos instalados (Steam, Epic, Riot, Battle.net, EA, Ubisoft, GOG, Rockstar).
/// Somente leitura.
pub mod games;

/// Detecção de GPUs instaladas via chaves do registro de classe de dispositivo.
/// Lê sem elevação (HKLM\SYSTEM\...\Class\...) — somente leitura, não modifica nada.
pub mod gpu {
    #[derive(Debug, Clone)]
    pub struct GpuInfo {
        pub name: String,
        pub vendor: String, // "NVIDIA" | "AMD" | "Intel"
    }

    pub fn detect() -> Vec<GpuInfo> {
        #[cfg(windows)]
        return imp::detect();
        #[cfg(not(windows))]
        return vec![];
    }

    #[cfg(windows)]
    mod imp {
        use super::super::registry_hklm;
        use super::GpuInfo;

        const DISPLAY_CLASS: &str =
            "SYSTEM\\CurrentControlSet\\Control\\Class\\{4d36e968-e325-11ce-bfc1-08002be10318}";

        pub fn detect() -> Vec<GpuInfo> {
            let mut gpus: Vec<GpuInfo> = Vec::new();
            for i in 0..16u32 {
                let sub = format!("{DISPLAY_CLASS}\\{i:04}");
                let desc = match registry_hklm::read_string(&sub, "DriverDesc") {
                    Ok(Some(s)) if !s.trim().is_empty() => s,
                    _ => continue,
                };
                let vendor = if desc.contains("NVIDIA") || desc.contains("nVidia") {
                    "NVIDIA"
                } else if desc.contains("AMD") || desc.contains("Radeon") || desc.contains("Advanced Micro") {
                    "AMD"
                } else if desc.contains("Intel") {
                    "Intel"
                } else {
                    continue;
                };
                // Evitar duplicatas (mesmo nome)
                if !gpus.iter().any(|g| g.name == desc) {
                    gpus.push(GpuInfo { name: desc, vendor: vendor.into() });
                }
            }
            gpus
        }
    }
}

/// Diagnóstico (somente leitura) de drivers de dispositivo: GPU, Rede e Áudio.
/// Lê as chaves de classe em HKLM\SYSTEM\...\Control\Class — não modifica nada,
/// não baixa, não instala, não atualiza. Apenas detecta e informa.
pub mod drivers {
    /// Categoria de driver inspecionada.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
    pub enum DriverCategory {
        Gpu,
        Network,
        Audio,
    }

    /// Informação de um driver detectado. Campos opcionais quando o SO não os expõe.
    #[derive(Debug, Clone, serde::Serialize)]
    pub struct DriverInfo {
        pub category: DriverCategory,
        pub name: String,
        pub vendor: Option<String>,
        pub version: Option<String>,
        /// Data do driver, normalizada para "AAAA-MM-DD" quando possível.
        pub date: Option<String>,
    }

    /// Detecta os drivers de GPU, Rede e Áudio instalados (somente leitura).
    pub fn detect() -> Vec<DriverInfo> {
        #[cfg(windows)]
        return imp::detect();
        #[cfg(not(windows))]
        return vec![];
    }

    /// Normaliza a data do driver. O Windows grava "M-D-AAAA" em DriverDate (REG_SZ).
    /// Convertemos para "AAAA-MM-DD"; se o formato for inesperado, retornamos cru.
    /// Exposta para teste fora do Windows (sem dependência de registro).
    pub fn normalize_date(raw: &str) -> String {
        let parts: Vec<&str> = raw.trim().split(['-', '/']).collect();
        if parts.len() == 3 {
            if let (Ok(m), Ok(d), Ok(y)) =
                (parts[0].parse::<u32>(), parts[1].parse::<u32>(), parts[2].parse::<u32>())
            {
                if y > 1900 && (1..=12).contains(&m) && (1..=31).contains(&d) {
                    return format!("{y:04}-{m:02}-{d:02}");
                }
            }
        }
        raw.trim().to_string()
    }

    /// Heurística leve para filtrar adaptadores virtuais/genéricos que só geram ruído.
    /// Exposta para teste fora do Windows.
    pub fn is_virtual_adapter(name: &str) -> bool {
        let l = name.to_lowercase();
        l.contains("virtual")
            || l.contains("loopback")
            || l.contains("miniport")
            || l.contains("microsoft kernel")
            || l.contains("vpn")
    }

    #[cfg(windows)]
    mod imp {
        use super::super::registry_hklm;
        use super::{is_virtual_adapter, normalize_date, DriverCategory, DriverInfo};

        // GUIDs de classe de dispositivo do Windows (estáveis há décadas).
        const CLASS_GPU: &str = "{4d36e968-e325-11ce-bfc1-08002be10318}";
        const CLASS_NET: &str = "{4d36e972-e325-11ce-bfc1-08002be10318}";
        const CLASS_AUDIO: &str = "{4d36e96c-e325-11ce-bfc1-08002be10318}";

        fn class_base(guid: &str) -> String {
            format!("SYSTEM\\CurrentControlSet\\Control\\Class\\{guid}")
        }

        fn read_opt(sub: &str, name: &str) -> Option<String> {
            registry_hklm::read_string(sub, name)
                .ok()
                .flatten()
                .filter(|s| !s.trim().is_empty())
                .map(|s| s.trim().to_string())
        }

        fn read_category(guid: &str, category: DriverCategory) -> Vec<DriverInfo> {
            let base = class_base(guid);
            let mut out: Vec<DriverInfo> = Vec::new();
            // As instâncias são subchaves "0000", "0001", ...
            for i in 0..32u32 {
                let sub = format!("{base}\\{i:04}");
                let name = match read_opt(&sub, "DriverDesc") {
                    Some(s) => s,
                    None => continue,
                };
                if is_virtual_adapter(&name) {
                    continue;
                }
                let vendor = read_opt(&sub, "ProviderName");
                let version = read_opt(&sub, "DriverVersion");
                let date = read_opt(&sub, "DriverDate").map(|s| normalize_date(&s));

                if !out.iter().any(|d| d.name == name) {
                    out.push(DriverInfo { category, name, vendor, version, date });
                }
            }
            out
        }

        pub fn detect() -> Vec<DriverInfo> {
            let mut all = Vec::new();
            all.extend(read_category(CLASS_GPU, DriverCategory::Gpu));
            all.extend(read_category(CLASS_NET, DriverCategory::Network));
            all.extend(read_category(CLASS_AUDIO, DriverCategory::Audio));
            all
        }
    }
}

#[cfg(test)]
mod drivers_tests {
    use crate::drivers::{is_virtual_adapter, normalize_date};

    #[test]
    fn normalize_date_converts_windows_format() {
        assert_eq!(normalize_date("3-15-2024"), "2024-03-15");
        assert_eq!(normalize_date("12-1-2023"), "2023-12-01");
    }

    #[test]
    fn normalize_date_passes_through_unknown_format() {
        assert_eq!(normalize_date("desconhecido"), "desconhecido");
        assert_eq!(normalize_date("2024.03.15"), "2024.03.15");
    }

    #[test]
    fn virtual_adapters_are_filtered() {
        assert!(is_virtual_adapter("WAN Miniport (IP)"));
        assert!(is_virtual_adapter("Microsoft Kernel Debug Network Adapter"));
        assert!(is_virtual_adapter("VirtualBox Host-Only Ethernet Adapter"));
        assert!(!is_virtual_adapter("NVIDIA GeForce RTX 4070"));
        assert!(!is_virtual_adapter("Realtek PCIe GbE Family Controller"));
    }
}

#[cfg(all(test, windows))]
mod tests {
    use crate::{power, registry};

    // Subchave de teste isolada por processo (HKCU, sem elevação). Limpa ao final.
    fn subkey(tag: &str) -> String {
        format!("Software\\TkSpeedTest\\{}_{}", tag, std::process::id())
    }

    #[test]
    fn registry_string_write_read_delete() {
        let sub = subkey("str");
        let name = "Val";
        let _ = registry::delete_value(&sub, name);

        // ausente → None
        assert_eq!(registry::read_string(&sub, name).unwrap(), None);
        // escreve → lê de volta
        registry::write_string(&sub, name, "hello").unwrap();
        assert_eq!(registry::read_string(&sub, name).unwrap(), Some("hello".to_string()));
        // atualiza → novo valor
        registry::write_string(&sub, name, "world").unwrap();
        assert_eq!(registry::read_string(&sub, name).unwrap(), Some("world".to_string()));
        // remove → None; remover de novo é idempotente (sucesso)
        registry::delete_value(&sub, name).unwrap();
        assert_eq!(registry::read_string(&sub, name).unwrap(), None);
        registry::delete_value(&sub, name).unwrap();
    }

    #[test]
    fn registry_dword_write_read_delete() {
        let sub = subkey("dword");
        let name = "Num";
        let _ = registry::delete_value(&sub, name);

        assert_eq!(registry::read_u32(&sub, name).unwrap(), None);
        registry::write_u32(&sub, name, 42).unwrap();
        assert_eq!(registry::read_u32(&sub, name).unwrap(), Some(42));
        registry::write_u32(&sub, name, 7).unwrap();
        assert_eq!(registry::read_u32(&sub, name).unwrap(), Some(7));
        registry::delete_value(&sub, name).unwrap();
        assert_eq!(registry::read_u32(&sub, name).unwrap(), None);
    }

    #[test]
    fn power_active_scheme_is_valid_and_settable_to_self() {
        // Em ambientes sem powercfg disponível, não falha o suite.
        let guid = match power::get_active_scheme() {
            Ok(g) => g,
            Err(_) => return,
        };
        assert_eq!(guid.len(), 36, "GUID do plano ativo deve ter 36 chars");
        assert!(power::scheme_exists(&guid), "o plano ativo deve existir na lista");
        // Trocar para o MESMO plano é seguro (no-op) e não exige elevação.
        power::set_active_scheme(&guid).unwrap();
        assert_eq!(power::get_active_scheme().unwrap(), guid, "plano permanece o mesmo");
    }
}
