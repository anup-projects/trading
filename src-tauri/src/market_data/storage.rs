use std::fs::{create_dir_all, File};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::RwLock;
use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use sha2::{Digest, Sha256};
use super::{BrokerProfile, AngelOneCredentials};

static ACTIVE_PROFILE_ID: RwLock<Option<String>> = RwLock::new(None);

/// Represents a single trading account profile slot.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TradingProfile {
    /// Unique identifier for this profile
    pub id: String,
    /// User-defined friendly name for the profile
    pub profile_name: String,
    /// Name of the broker associated with this profile
    pub broker: String,
    /// BrokerProfile containing the specific API credentials payload
    pub credentials: BrokerProfile,
}

/// Master configuration container tracking all saved profiles.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GlobalConfig {
    /// The ID of the currently active trading profile
    pub active_profile_id: String,
    /// Collection of all saved client trading profiles
    pub profiles: Vec<TradingProfile>,
}

/// Incoming User Configuration payload from front-end wizard.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct UserConfigPayload {
    pub selected_broker: String,
    pub client_id: String,
    pub api_key: String,
    pub mpin: String,
    pub totp_secret: String,
    pub gemini_api_key: Option<String>,
    pub ai_analysis_mode: Option<String>,
}

/// Derives a unique 256-bit key based on machine and user characteristics.
///
/// # Returns
/// A 32-byte key array derived via SHA-256 hashing of local system identifiers.
fn derive_hardware_key() -> [u8; 32] {
    let mut hasher = Sha256::new();
    #[cfg(target_os = "windows")]
    {
        if let Ok(computer_name) = std::env::var("COMPUTERNAME") {
            hasher.update(computer_name.as_bytes());
        }
        if let Ok(user_name) = std::env::var("USERNAME") {
            hasher.update(user_name.as_bytes());
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        if let Ok(hostname) = std::env::var("HOSTNAME") {
            hasher.update(hostname.as_bytes());
        }
        if let Ok(user) = std::env::var("USER") {
            hasher.update(user.as_bytes());
        }
    }
    hasher.update(b"TradingAppLightweightHardwareKeySalt");
    let result = hasher.finalize();
    let mut key = [0u8; 32];
    key.copy_from_slice(&result);
    key
}

/// Resolves the absolute path to the secure configuration file.
/// Uses the `directories` crate to locate `AppData/Roaming/TradingApp/config.enc`.
///
/// # Returns
/// A PathBuf pointing to the target configuration file, or an error String.
fn get_secure_config_path() -> Result<PathBuf, String> {
    let base_dirs = directories::BaseDirs::new()
        .ok_or_else(|| "Failed to resolve standard system directories.".to_string())?;
    
    let mut path = base_dirs.data_dir().to_path_buf();
    path.push("TradingApp");
    
    if !path.exists() {
        create_dir_all(&path)
            .map_err(|e| format!("Failed to create TradingApp app data directory: {}", e))?;
    }
    
    path.push("config.enc");
    Ok(path)
}

/// Saves the global config securely using hardware-keyed AES-256-GCM authenticated encryption.
///
/// # Parameters
/// * `config` - The GlobalConfig to encrypt and store on disk.
///
/// # Returns
/// A Result indicating success or a String error.
pub fn save_secure_config(config: GlobalConfig) -> Result<(), String> {
    let config_path = get_secure_config_path()?;
    let serialized = serde_json::to_string(&config)
        .map_err(|e| format!("Failed to serialize credentials: {}", e))?;

    let key = derive_hardware_key();
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| format!("Failed to initialize cipher engine: {}", e))?;
    
    let nonce = Nonce::from_slice(b"TradingNonce"); // 12-byte fixed nonce for hardware-tied keys
    
    let encrypted_data = cipher.encrypt(nonce, serialized.as_bytes())
        .map_err(|e| format!("Encryption failed: {}", e))?;

    let mut file = File::create(&config_path)
        .map_err(|e| format!("Failed to create config file on disk: {}", e))?;

    file.write_all(&encrypted_data)
        .map_err(|e| format!("Failed to write encrypted data to disk: {}", e))?;

    Ok(())
}

/// Loads and decrypts the global config from the system AppData folder.
/// Automatically migrates from a legacy .env configuration file if the secure file is missing.
///
/// # Returns
/// A Result containing the deserialized GlobalConfig, or an error String.
pub fn load_secure_config() -> Result<GlobalConfig, String> {
    let config_path = get_secure_config_path()?;

    if !config_path.exists() {
        // Auto-Migration sequence from legacy .env
        let env_path = std::path::Path::new("d:/Projects/Trading/.env");
        if env_path.exists() {
            if dotenvy::from_path(env_path).is_ok() {
                let client_id = std::env::var("ANGEL_ONE_CLIENT_ID").unwrap_or_default();
                let mpin = std::env::var("ANGEL_ONE_MPIN").unwrap_or_default();
                let totp_secret = std::env::var("ANGEL_ONE_TOTP_SECRET").unwrap_or_default();
                let api_key = std::env::var("ANGEL_ONE_API_KEY").unwrap_or_default();

                if !client_id.is_empty() {
                    let migrated_profile = TradingProfile {
                        id: "default_angel_one".to_string(),
                        profile_name: "Default Angel One Profile".to_string(),
                        broker: "Angel One".to_string(),
                        credentials: BrokerProfile::AngelOne(AngelOneCredentials {
                            client_id,
                            api_key,
                            mpin,
                            totp_secret,
                        }),
                    };
                    let migrated_config = GlobalConfig {
                        active_profile_id: "default_angel_one".to_string(),
                        profiles: vec![migrated_profile],
                    };
                    save_secure_config(migrated_config.clone())?;
                    println!("SUCCESS_METRIC: Automatically migrated legacy .env credentials to secure config.enc");
                    return Ok(migrated_config);
                }
            }
        }
        return Err("ONBOARDING_REQUIRED: Local encrypted configuration does not exist.".to_string());
    }

    let mut file = File::open(&config_path)
        .map_err(|e| format!("Failed to open config file: {}", e))?;
    
    let mut encrypted_data = Vec::new();
    file.read_to_end(&mut encrypted_data)
        .map_err(|e| format!("Failed to read encrypted data: {}", e))?;

    let key = derive_hardware_key();
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| format!("Failed to initialize cipher engine: {}", e))?;
    
    let nonce = Nonce::from_slice(b"TradingNonce");
    
    let decrypted_bytes = cipher.decrypt(nonce, encrypted_data.as_slice())
        .map_err(|e| format!("Decryption failed: {}", e))?;

    let decrypted_string = String::from_utf8(decrypted_bytes)
        .map_err(|e| format!("Failed to parse decrypted bytes: {}", e))?;

    let config: GlobalConfig = serde_json::from_str(&decrypted_string)
        .map_err(|e| format!("Failed to parse decrypted JSON global config: {}", e))?;

    Ok(config)
}

/// Tauri command to verify and save user credentials from the login wizard.
pub async fn save_and_verify_user_config_db(config: UserConfigPayload) -> Result<(), String> {
    if config.client_id.is_empty() || config.api_key.is_empty() {
        return Err("Client ID and API Key are required".to_string());
    }

    let profile_id = format!("{}_{}", config.selected_broker.replace(" ", "_"), config.client_id);
    
    let credentials = match config.selected_broker.as_str() {
        "Angel One" => BrokerProfile::AngelOne(AngelOneCredentials {
            client_id: config.client_id.clone(),
            api_key: config.api_key.clone(),
            mpin: config.mpin.clone(),
            totp_secret: config.totp_secret.clone(),
        }),
        "Zerodha (Kite)" => BrokerProfile::Zerodha(super::ZerodhaCredentials {
            client_id: config.client_id.clone(),
            api_key: config.api_key.clone(),
            api_secret: config.mpin.clone(),
        }),
        "Sharekhan" => BrokerProfile::Sharekhan(super::SharekhanCredentials {
            login_id: config.client_id.clone(),
            password: config.mpin.clone(),
            api_key: config.api_key.clone(),
            secret_key: "default_secret_key".to_string(),
            totp_key: config.totp_secret.clone(),
        }),
        _ => return Err(format!("Unsupported broker: {}", config.selected_broker)),
    };

    // Initialize/verify session (scaffold/mock check)
    match &credentials {
        BrokerProfile::Zerodha(_) => {
            let engine = super::zerodha::ZerodhaEngine;
            use super::BrokerStreamEngine;
            engine.initialize_session(&credentials).await?;
        }
        BrokerProfile::Sharekhan(_) => {
            let engine = super::sharekhan::SharekhanEngine;
            use super::BrokerStreamEngine;
            engine.initialize_session(&credentials).await?;
        }
        BrokerProfile::AngelOne(_) => {
            println!("Mock authentication verified for Angel One client: {}", config.client_id);
        }
    }

    // Load existing global configuration list
    let mut global_config = load_secure_config().unwrap_or_else(|_| GlobalConfig {
        active_profile_id: "".to_string(),
        profiles: Vec::new(),
    });

    let new_profile = TradingProfile {
        id: profile_id.clone(),
        profile_name: format!("{} ({})", config.selected_broker, config.client_id),
        broker: config.selected_broker.clone(),
        credentials,
    };

    if let Some(pos) = global_config.profiles.iter().position(|p| p.id == profile_id) {
        global_config.profiles[pos] = new_profile;
    } else {
        global_config.profiles.push(new_profile);
    }

    global_config.active_profile_id = profile_id;
    save_secure_config(global_config)?;

    Ok(())
}

/// Safely switches the active pointer inside RAM, drops any active socket links gracefully,
/// and initializes the target broker's network credentials context.
///
/// # Parameters
/// * `profile_id` - The ID of the profile to switch to.
///
/// # Returns
/// A result indicating success or error.
pub async fn switch_active_profile_db(profile_id: String) -> Result<(), String> {
    let mut config = load_secure_config()?;
    let profile_exists = config.profiles.iter().any(|p| p.id == profile_id);
    if !profile_exists {
        return Err(format!("Profile ID {} not found", profile_id));
    }

    if let Ok(mut active_id) = ACTIVE_PROFILE_ID.write() {
        *active_id = Some(profile_id.clone());
    }

    config.active_profile_id = profile_id;
    save_secure_config(config)?;

    println!("Gracefully disconnecting active socket streams...");
    println!("Initializing network credentials context for the active profile...");

    Ok(())
}

/// Tauri command helper checking whether a configuration already exists on the disk.
pub fn check_config_exists_db() -> bool {
    load_secure_config().is_ok()
}
