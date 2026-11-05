use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use std::{fs, path::PathBuf, sync::Mutex, time::{Duration, Instant, SystemTime, UNIX_EPOCH}};
use zeroize::Zeroizing;
mod wallet;

const SESSION_IDLE_TIMEOUT: Duration = Duration::from_secs(15 * 60);
use tauri::{Manager, State};
use uuid::Uuid;

struct AppState {
    db: Mutex<Connection>,
    authenticated: Mutex<bool>,
    last_activity: Mutex<Instant>,
}

#[derive(Serialize)]
struct AuthState {
    has_password: bool,
    authenticated: bool,
}

#[derive(Serialize)]
struct WalletState {
    connected: bool,
    address: Option<String>,
}

#[derive(Serialize)]
struct StrategyRecord {
    id: String,
    chain_id: u64,
    base_token: String,
    quote_token: String,
    venues: Vec<String>,
    created_at: i64,
}

fn database_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir.join("arbx.sqlite3"))
}

fn initialize_database(path: PathBuf) -> Result<Connection, String> {
    let conn = Connection::open(path).map_err(|e| e.to_string())?;
    conn.execute_batch(
        "
        PRAGMA foreign_keys = ON;
        PRAGMA journal_mode = WAL;
        CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY,
            applied_at INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS app_settings (
            setting_key TEXT PRIMARY KEY,
            setting_value TEXT NOT NULL
        );
        CREATE TABLE IF NOT EXISTS strategies (
            id TEXT PRIMARY KEY,
            chain_id INTEGER NOT NULL,
            base_token TEXT NOT NULL,
            quote_token TEXT NOT NULL,
            venues_json TEXT NOT NULL,
            created_at INTEGER NOT NULL
        );
        INSERT OR IGNORE INTO schema_migrations(version, applied_at)
        VALUES (1, unixepoch());
        "
    ).map_err(|e| e.to_string())?;
    Ok(conn)
}

fn current_time() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs() as i64
}

fn password_hash(password: &str) -> Result<String, String> {
    let salt = SaltString::generate(&mut argon2::password_hash::rand_core::OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|e| e.to_string())
}

fn verify_password(password: &str, encoded_hash: &str) -> bool {
    let Ok(parsed) = PasswordHash::new(encoded_hash) else { return false; };
    Argon2::default().verify_password(password.as_bytes(), &parsed).is_ok()
}

fn session_is_authenticated(state: &AppState, refresh_activity: bool) -> Result<bool, String> {
    let mut authenticated = state.authenticated.lock().map_err(|_| "Application session is unavailable".to_string())?;
    if !*authenticated { return Ok(false); }
    let mut last_activity = state.last_activity.lock().map_err(|_| "Application session is unavailable".to_string())?;
    if last_activity.elapsed() >= SESSION_IDLE_TIMEOUT {
        *authenticated = false;
        return Ok(false);
    }
    if refresh_activity { *last_activity = Instant::now(); }
    Ok(true)
}

fn require_authenticated(state: &AppState) -> Result<(), String> {
    if session_is_authenticated(state, true)? { Ok(()) } else { Err("Application is locked. Unlock ArbX to continue.".to_string()) }
}

#[tauri::command]
fn auth_state(state: State<'_, AppState>) -> Result<AuthState, String> {
    let db = state.db.lock().map_err(|_| "Database is unavailable".to_string())?;
    let hash: Option<String> = db.query_row(
        "SELECT setting_value FROM app_settings WHERE setting_key = 'password_hash'",
        [], |row| row.get(0)
    ).optional().map_err(|e| e.to_string())?;
    let authenticated = session_is_authenticated(&state, false)?;
    Ok(AuthState { has_password: hash.is_some(), authenticated })
}

#[tauri::command]
fn setup_password(password: String, state: State<'_, AppState>) -> Result<(), String> {
    let password = Zeroizing::new(password);
    if password.chars().count() < 12 {
        return Err("Use a password with at least 12 characters.".into());
    }
    let mut db = state.db.lock().map_err(|_| "Database is unavailable".to_string())?;
    let existing: Option<String> = db.query_row(
        "SELECT setting_value FROM app_settings WHERE setting_key = 'password_hash'",
        [], |row| row.get(0)
    ).optional().map_err(|e| e.to_string())?;
    if existing.is_some() {
        return Err("A password is already configured. Unlock the app instead.".into());
    }
    let encoded = password_hash(&password)?;
    db.execute(
        "INSERT INTO app_settings(setting_key, setting_value) VALUES ('password_hash', ?1)",
        params![encoded]
    ).map_err(|e| e.to_string())?;
    *state.authenticated.lock().map_err(|_| "Application session is unavailable".to_string())? = true;
    *state.last_activity.lock().map_err(|_| "Application session is unavailable".to_string())? = Instant::now();
    Ok(())
}

#[tauri::command]
fn login(password: String, state: State<'_, AppState>) -> Result<(), String> {
    let password = Zeroizing::new(password);
    let db = state.db.lock().map_err(|_| "Database is unavailable".to_string())?;
    let encoded: Option<String> = db.query_row(
        "SELECT setting_value FROM app_settings WHERE setting_key = 'password_hash'",
        [], |row| row.get(0)
    ).optional().map_err(|e| e.to_string())?;
    let Some(encoded) = encoded else { return Err("No application password has been configured.".into()); };
    if !verify_password(&password, &encoded) {
        return Err("Incorrect password.".into());
    }
    *state.authenticated.lock().map_err(|_| "Application session is unavailable".to_string())? = true;
    *state.last_activity.lock().map_err(|_| "Application session is unavailable".to_string())? = Instant::now();
    Ok(())
}

#[tauri::command]
fn lock_app(state: State<'_, AppState>) -> Result<(), String> {
    *state.authenticated.lock().map_err(|_| "Application session is unavailable".to_string())? = false;
    Ok(())
}

#[tauri::command]
fn wallet_state(state: State<'_, AppState>) -> Result<WalletState, String> {
    require_authenticated(&state)?;
    let db = state.db.lock().map_err(|_| "Database is unavailable".to_string())?;
    let address: Option<String> = db.query_row(
        "SELECT setting_value FROM app_settings WHERE setting_key = 'wallet_address'",
        [], |row| row.get(0)
    ).optional().map_err(|e| e.to_string())?;
    drop(db);
    let stored_address = wallet::stored_address()?;
    match stored_address {
        Some(derived) => {
            if address.as_deref() != Some(derived.as_str()) {
                let db = state.db.lock().map_err(|_| "Database is unavailable".to_string())?;
                db.execute(
                    "INSERT INTO app_settings(setting_key, setting_value) VALUES ('wallet_address', ?1)
                     ON CONFLICT(setting_key) DO UPDATE SET setting_value = excluded.setting_value",
                    params![derived]
                ).map_err(|e| e.to_string())?;
            }
            Ok(WalletState { connected: true, address: Some(derived) })
        }
        None => {
            if address.is_some() {
                let db = state.db.lock().map_err(|_| "Database is unavailable".to_string())?;
                db.execute("DELETE FROM app_settings WHERE setting_key = 'wallet_address'", [])
                    .map_err(|e| e.to_string())?;
            }
            Ok(WalletState { connected: false, address: None })
        }
    }
}

#[tauri::command]
async fn import_wallet(state: State<'_, AppState>) -> Result<WalletState, String> {
    require_authenticated(&state)?;
    let (address, private_key) = tauri::async_runtime::spawn_blocking(wallet::prompt_and_derive)
        .await
        .map_err(|_| "The secure wallet-entry task could not be started.".to_string())??;
    require_authenticated(&state)?;
    let previous = wallet::replace_secret(&private_key)?;
    let update_result = (|| -> Result<(), String> {
        let db = state.db.lock().map_err(|_| "Database is unavailable".to_string())?;
        db.execute(
            "INSERT INTO app_settings(setting_key, setting_value) VALUES ('wallet_address', ?1)
             ON CONFLICT(setting_key) DO UPDATE SET setting_value = excluded.setting_value",
            params![address]
        ).map_err(|e| e.to_string())?;
        Ok(())
    })();
    if let Err(error) = update_result {
        let previous_bytes = previous.as_ref().map(|value| value.as_slice());
        if wallet::restore_secret(previous_bytes).is_err() {
            return Err("Wallet metadata could not be saved and the previous secure credential could not be restored. Stop here and contact support before continuing.".into());
        }
        return Err(error);
    }
    Ok(WalletState { connected: true, address: Some(address) })
}

#[tauri::command]
fn remove_wallet(state: State<'_, AppState>) -> Result<(), String> {
    require_authenticated(&state)?;
    let previous = wallet::delete_secret()?;
    let update_result = (|| -> Result<(), String> {
        let db = state.db.lock().map_err(|_| "Database is unavailable".to_string())?;
        db.execute("DELETE FROM app_settings WHERE setting_key = 'wallet_address'", [])
            .map_err(|e| e.to_string())?;
        Ok(())
    })();
    if let Err(error) = update_result {
        let previous_bytes = previous.as_ref().map(|value| value.as_slice());
        if wallet::restore_secret(previous_bytes).is_err() {
            return Err("Wallet metadata could not be updated and the secure credential could not be restored. Stop here and contact support before continuing.".into());
        }
        return Err(error);
    }
    Ok(())
}

#[tauri::command]
fn list_strategies(state: State<'_, AppState>) -> Result<Vec<StrategyRecord>, String> {
    require_authenticated(&state)?;
    let db = state.db.lock().map_err(|_| "Database is unavailable".to_string())?;
    let mut stmt = db.prepare(
        "SELECT id, chain_id, base_token, quote_token, venues_json, created_at FROM strategies ORDER BY created_at DESC"
    ).map_err(|e| e.to_string())?;
    let rows = stmt.query_map([], |row| {
        let venues_json: String = row.get(4)?;
        let venues = serde_json::from_str(&venues_json).unwrap_or_default();
        Ok(StrategyRecord {
            id: row.get(0)?,
            chain_id: row.get::<_, i64>(1)? as u64,
            base_token: row.get(2)?,
            quote_token: row.get(3)?,
            venues,
            created_at: row.get(5)?,
        })
    }).map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

fn validate_strategy_input(base_token: &str, quote_token: &str, venues: &[String]) -> Result<(), String> {
    let valid_address = |address: &str| {
        address.len() == 42 && address.starts_with("0x") && address[2..].bytes().all(|b| b.is_ascii_hexdigit())
    };
    if !valid_address(base_token) || !valid_address(quote_token) {
        return Err("Both token addresses must be valid 20-byte EVM addresses.".into());
    }
    if base_token.eq_ignore_ascii_case(quote_token) {
        return Err("Base and quote token addresses must be different.".into());
    }
    let unique_venues: std::collections::HashSet<&String> = venues.iter().collect();
    if venues.len() < 2 || unique_venues.len() != venues.len() || venues.iter().any(|v| v.trim().is_empty()) {
        return Err("Select at least two different venues.".into());
    }
    Ok(())
}

#[tauri::command]
fn create_strategy(base_token: String, quote_token: String, venues: Vec<String>, state: State<'_, AppState>) -> Result<StrategyRecord, String> {
    require_authenticated(&state)?;
    validate_strategy_input(&base_token, &quote_token, &venues)?;
    let created_at = current_time();
    let record = StrategyRecord {
        id: Uuid::new_v4().to_string(),
        chain_id: 137,
        base_token: base_token.to_lowercase(),
        quote_token: quote_token.to_lowercase(),
        venues: venues.clone(),
        created_at,
    };
    let db = state.db.lock().map_err(|_| "Database is unavailable".to_string())?;
    db.execute(
        "INSERT INTO strategies(id, chain_id, base_token, quote_token, venues_json, created_at) VALUES (?1, 137, ?2, ?3, ?4, ?5)",
        params![record.id, record.base_token, record.quote_token, serde_json::to_string(&venues).map_err(|e| e.to_string())?, created_at]
    ).map_err(|e| e.to_string())?;
    Ok(record)
}

#[tauri::command]
fn delete_strategy(id: String, state: State<'_, AppState>) -> Result<(), String> {
    require_authenticated(&state)?;
    let db = state.db.lock().map_err(|_| "Database is unavailable".to_string())?;
    db.execute("DELETE FROM strategies WHERE id = ?1", params![id]).map_err(|e| e.to_string())?;
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let path = database_path(app.handle()).map_err(std::io::Error::other)?;
            let db = initialize_database(path).map_err(std::io::Error::other)?;
            app.manage(AppState {
                db: Mutex::new(db),
                authenticated: Mutex::new(false),
                last_activity: Mutex::new(Instant::now()),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            auth_state, setup_password, login, lock_app,
            wallet_state, import_wallet, remove_wallet,
            list_strategies, create_strategy, delete_strategy
        ])
        .run(tauri::generate_context!())
        .expect("error while running ArbX");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn password_hash_is_not_plaintext_and_verifies() {
        let encoded = password_hash("a sufficiently long test password").unwrap();
        assert_ne!(encoded, "a sufficiently long test password");
        assert!(verify_password("a sufficiently long test password", &encoded));
        assert!(!verify_password("wrong password", &encoded));
    }

    #[test]
    fn database_initializes_and_persists_schema() {
        let path = std::env::temp_dir().join(format!("arbx-test-{}.sqlite3", Uuid::new_v4()));
        let conn = initialize_database(path.clone()).unwrap();
        let version: i64 = conn.query_row("SELECT MAX(version) FROM schema_migrations", [], |row| row.get(0)).unwrap();
        assert_eq!(version, 1);
        drop(conn);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn strategy_validation_accepts_two_distinct_evm_tokens_and_venues() {
        let venues = vec!["dex-a".to_string(), "dex-b".to_string()];
        assert!(validate_strategy_input(&format!("0x{}", "1".repeat(40)), &format!("0x{}", "2".repeat(40)), &venues).is_ok());
    }

    #[test]
    fn strategy_validation_rejects_invalid_or_identical_tokens() {
        let venues = vec!["dex-a".to_string(), "dex-b".to_string()];
        assert!(validate_strategy_input("0x1234", &format!("0x{}", "2".repeat(40)), &venues).is_err());
        let same = format!("0x{}", "1".repeat(40));
        assert!(validate_strategy_input(&same, &same, &venues).is_err());
    }

    #[test]
    fn strategy_validation_rejects_duplicate_or_insufficient_venues() {
        let base = format!("0x{}", "1".repeat(40));
        let quote = format!("0x{}", "2".repeat(40));
        assert!(validate_strategy_input(&base, &quote, &["dex-a".into()]).is_err());
        assert!(validate_strategy_input(&base, &quote, &["dex-a".into(), "dex-a".into()]).is_err());
        assert!(validate_strategy_input(&base, &quote, &["dex-a".into(), " ".into()]).is_err());
    }

    #[test]
    fn idle_session_expires_and_revokes_authenticated_state() {
        let path = std::env::temp_dir().join(format!("arbx-session-test-{}.sqlite3", Uuid::new_v4()));
        let db = initialize_database(path.clone()).unwrap();
        let state = AppState {
            db: Mutex::new(db),
            authenticated: Mutex::new(true),
            last_activity: Mutex::new(Instant::now() - SESSION_IDLE_TIMEOUT - Duration::from_secs(1)),
        };
        assert!(!session_is_authenticated(&state, false).unwrap());
        assert!(!*state.authenticated.lock().unwrap());
        drop(state);
        let _ = fs::remove_file(path);
    }
}
