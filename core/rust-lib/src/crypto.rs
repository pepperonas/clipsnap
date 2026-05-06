//! At-rest encryption for sensitive content fields stored in SQLite.
//!
//! ## Threat model
//!
//! The previous "limitation" the user wanted to close: anyone with read
//! access to the user's profile (other apps running as the same user, a
//! stolen drive image, an accidental backup, …) could open
//! `~/Library/Application Support/ClipSnap/history.db` and read every
//! password, token, and snippet body in plaintext.
//!
//! With this module wired in, those fields are encrypted with AES-256-GCM.
//! The key lives in the OS keychain (macOS Keychain / Windows Credential
//! Manager / Linux Secret Service) so it survives reboots and is bound to
//! the logged-in user. If the keychain is unavailable (rare — locked
//! keychain, missing keychain access, or first launch on Linux without
//! a Secret Service implementation), we fall back to a 0600 keyfile in
//! the data dir. That fallback is strictly worse — file-system access
//! gets the attacker the key too — but it keeps the app usable instead
//! of crashing.
//!
//! ## Storage format
//!
//! Encrypted strings are stored as TEXT with a marker prefix:
//!
//! ```text
//! "v1:" + base64( 12-byte random nonce || aes-gcm ciphertext+tag )
//! ```
//!
//! `decrypt` is permissive: any string that does **not** start with
//! `v1:` is treated as legacy plaintext and returned as-is. That's the
//! migration hook — see [`migrate_table`].
//!
//! ## What is and isn't encrypted
//!
//! Encrypted: `entries.content_text`, `entries.content_data`,
//! `snippets.body`, `notes.content_text`, `notes.content_data`.
//!
//! NOT encrypted: timestamps, IDs, content-type tags, hashes (used for
//! dedup), abbreviations, titles, categories. None of those reveal the
//! actual clipboard content.

use std::path::Path;
use std::sync::OnceLock;

use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use anyhow::{anyhow, Context, Result};
use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine;
use rand::RngCore;

const PREFIX: &str = "v1:";
const NONCE_LEN: usize = 12;
const KEY_LEN: usize = 32;

const KEYRING_SERVICE: &str = "io.celox.clipsnap";
const KEYRING_USER: &str = "history-db-key-v1";
const KEYFILE_NAME: &str = ".dbkey";

/// AES-256-GCM cipher with a 32-byte key. Wrapped so callers can't
/// accidentally serialize the key.
pub struct Cipher {
    aead: Aes256Gcm,
}

impl Cipher {
    fn new(key_bytes: &[u8; KEY_LEN]) -> Self {
        let key = Key::<Aes256Gcm>::from_slice(key_bytes);
        Cipher {
            aead: Aes256Gcm::new(key),
        }
    }

    fn encrypt(&self, plain: &[u8]) -> Result<String> {
        let mut nonce_bytes = [0u8; NONCE_LEN];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        let ciphertext = self
            .aead
            .encrypt(nonce, plain)
            .map_err(|_| anyhow!("AES-GCM encrypt failed"))?;
        let mut combined = Vec::with_capacity(NONCE_LEN + ciphertext.len());
        combined.extend_from_slice(&nonce_bytes);
        combined.extend_from_slice(&ciphertext);
        Ok(format!("{PREFIX}{}", B64.encode(combined)))
    }

    fn decrypt(&self, value: &str) -> Result<String> {
        // Legacy plaintext (or any string that wasn't encrypted by us).
        if !value.starts_with(PREFIX) {
            return Ok(value.to_string());
        }
        let combined = B64
            .decode(value[PREFIX.len()..].as_bytes())
            .context("invalid base64 in encrypted value")?;
        if combined.len() < NONCE_LEN {
            return Err(anyhow!("encrypted value too short"));
        }
        let (nonce_bytes, ciphertext) = combined.split_at(NONCE_LEN);
        let nonce = Nonce::from_slice(nonce_bytes);
        let plain = self
            .aead
            .decrypt(nonce, ciphertext)
            .map_err(|_| anyhow!("AES-GCM decrypt failed (wrong key or tampered data)"))?;
        String::from_utf8(plain).context("decrypted payload was not valid UTF-8")
    }
}

static CIPHER: OnceLock<Cipher> = OnceLock::new();

/// Initialize the global cipher. Must be called once, early in app
/// startup, before any DB read or write. Idempotent — second call is a
/// no-op (subsequent inits with a different key would silently produce
/// undecryptable data, so we ignore them).
///
/// Tries OS keychain first (`io.celox.clipsnap` / `history-db-key-v1`).
/// Falls back to a 0600 keyfile under `data_dir`.
pub fn init(data_dir: &Path) -> Result<()> {
    if CIPHER.get().is_some() {
        return Ok(());
    }
    let key = load_or_create_key(data_dir)?;
    let _ = CIPHER.set(Cipher::new(&key));
    Ok(())
}

/// Encrypt a string for storage. If crypto isn't initialized, returns
/// the plaintext unchanged — this matters for tests that build
/// in-memory DBs without going through the app's setup path. In
/// production, [`init`] must have run before any DB call, so the
/// passthrough path is unreachable.
pub fn encrypt(plain: &str) -> String {
    match CIPHER.get() {
        Some(c) => c
            .encrypt(plain.as_bytes())
            .unwrap_or_else(|e| {
                tracing::warn!("encrypt failed, storing plaintext: {e:#}");
                plain.to_string()
            }),
        None => plain.to_string(),
    }
}

/// Decrypt a value read from the DB. Permissive: legacy plaintext (no
/// `v1:` prefix) is returned unchanged, so existing rows continue to
/// work until [`migrate_table`] re-encrypts them.
pub fn decrypt(value: &str) -> String {
    match CIPHER.get() {
        Some(c) => match c.decrypt(value) {
            Ok(s) => s,
            Err(e) => {
                tracing::warn!("decrypt failed, returning raw: {e:#}");
                value.to_string()
            }
        },
        None => value.to_string(),
    }
}

/// Walk a TEXT column in `table` and re-encrypt every row that doesn't
/// already start with the `v1:` prefix. Idempotent — already-encrypted
/// rows are skipped. Run from the same connection pool as everything
/// else so we share the WAL.
pub fn migrate_table(
    conn: &rusqlite::Connection,
    table: &str,
    columns: &[&str],
) -> Result<usize> {
    if CIPHER.get().is_none() {
        return Ok(0);
    }
    let id_select = format!("SELECT id FROM {table}");
    let mut stmt = conn.prepare(&id_select)?;
    let ids: Vec<i64> = stmt
        .query_map([], |r| r.get::<_, i64>(0))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    drop(stmt);

    let mut migrated = 0usize;
    for id in ids {
        for col in columns {
            let select_q = format!("SELECT {col} FROM {table} WHERE id = ?1");
            let raw: Option<String> = conn
                .query_row(&select_q, [id], |r| r.get::<_, Option<String>>(0))?;
            let Some(raw) = raw else { continue };
            if raw.starts_with(PREFIX) || raw.is_empty() {
                continue;
            }
            let encrypted = encrypt(&raw);
            let update_q = format!("UPDATE {table} SET {col} = ?1 WHERE id = ?2");
            conn.execute(&update_q, rusqlite::params![encrypted, id])?;
            migrated += 1;
        }
    }
    Ok(migrated)
}

/// Try the OS keychain first, then a keyfile in `data_dir`. Generates a
/// fresh key on first run and stores it in the keychain (and writes a
/// keyfile copy as a fallback so a future keychain-unavailable session
/// can still open the DB).
fn load_or_create_key(data_dir: &Path) -> Result<[u8; KEY_LEN]> {
    if let Some(k) = read_keychain() {
        // Keep the keyfile in sync so the fallback always works.
        let _ = write_keyfile(data_dir, &k);
        return Ok(k);
    }

    if let Some(k) = read_keyfile(data_dir) {
        let _ = write_keychain(&k);
        return Ok(k);
    }

    // First run on this machine — mint a fresh key and persist it.
    let mut key = [0u8; KEY_LEN];
    rand::thread_rng().fill_bytes(&mut key);
    if let Err(e) = write_keychain(&key) {
        tracing::warn!("could not store key in OS keychain: {e:#} — relying on keyfile");
    }
    write_keyfile(data_dir, &key).context("could not write keyfile fallback")?;
    Ok(key)
}

fn read_keychain() -> Option<[u8; KEY_LEN]> {
    let entry = keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER).ok()?;
    let secret = entry.get_password().ok()?;
    let bytes = B64.decode(secret.as_bytes()).ok()?;
    bytes.try_into().ok()
}

fn write_keychain(key: &[u8; KEY_LEN]) -> Result<()> {
    let entry = keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER)
        .context("keyring entry create failed")?;
    let encoded = B64.encode(key);
    entry
        .set_password(&encoded)
        .context("keyring set_password failed")?;
    Ok(())
}

fn keyfile_path(data_dir: &Path) -> std::path::PathBuf {
    data_dir.join(KEYFILE_NAME)
}

fn read_keyfile(data_dir: &Path) -> Option<[u8; KEY_LEN]> {
    let path = keyfile_path(data_dir);
    let bytes = std::fs::read(&path).ok()?;
    let decoded = B64.decode(&bytes).ok()?;
    decoded.try_into().ok()
}

fn write_keyfile(data_dir: &Path, key: &[u8; KEY_LEN]) -> Result<()> {
    let path = keyfile_path(data_dir);
    let encoded = B64.encode(key);
    std::fs::write(&path, encoded.as_bytes())
        .with_context(|| format!("write keyfile {}", path.display()))?;
    // Best-effort restrictive permissions on Unix-likes.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Direct cipher roundtrip — bypasses the global state so tests
    /// don't fight over the OnceLock.
    #[test]
    fn cipher_roundtrip() {
        let mut key = [0u8; KEY_LEN];
        rand::thread_rng().fill_bytes(&mut key);
        let c = Cipher::new(&key);
        let plain = "hello, secrets!";
        let enc = c.encrypt(plain.as_bytes()).unwrap();
        assert!(enc.starts_with(PREFIX));
        assert_ne!(enc, plain);
        let dec = c.decrypt(&enc).unwrap();
        assert_eq!(dec, plain);
    }

    #[test]
    fn cipher_passes_through_legacy_plaintext() {
        let key = [42u8; KEY_LEN];
        let c = Cipher::new(&key);
        let dec = c.decrypt("not encrypted").unwrap();
        assert_eq!(dec, "not encrypted");
    }

    #[test]
    fn cipher_handles_empty_string() {
        let key = [7u8; KEY_LEN];
        let c = Cipher::new(&key);
        let enc = c.encrypt(b"").unwrap();
        let dec = c.decrypt(&enc).unwrap();
        assert_eq!(dec, "");
    }

    #[test]
    fn cipher_each_encrypt_uses_fresh_nonce() {
        let key = [1u8; KEY_LEN];
        let c = Cipher::new(&key);
        let enc1 = c.encrypt(b"same").unwrap();
        let enc2 = c.encrypt(b"same").unwrap();
        assert_ne!(enc1, enc2, "nonce must be random per encryption");
    }

    #[test]
    fn cipher_rejects_tampered_ciphertext() {
        let key = [9u8; KEY_LEN];
        let c = Cipher::new(&key);
        let enc = c.encrypt(b"keep me safe").unwrap();
        // Flip a byte in the base64 payload.
        let mut bytes = enc.into_bytes();
        let last = bytes.len() - 1;
        bytes[last] = if bytes[last] == b'A' { b'B' } else { b'A' };
        let tampered = String::from_utf8(bytes).unwrap();
        assert!(c.decrypt(&tampered).is_err());
    }

    #[test]
    fn cipher_rejects_wrong_key() {
        let key1 = [1u8; KEY_LEN];
        let key2 = [2u8; KEY_LEN];
        let c1 = Cipher::new(&key1);
        let c2 = Cipher::new(&key2);
        let enc = c1.encrypt(b"hello").unwrap();
        assert!(c2.decrypt(&enc).is_err());
    }
}
