use anyhow::{Context, Result};
use chrono::Utc;
use parking_lot::Mutex;
use rusqlite::{params, Connection, OptionalExtension};
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use std::sync::Arc;

use crate::crypto;
use crate::models::{ClipEntry, ContentType, NewClip, MAX_ENTRIES};

pub type DbHandle = Arc<Mutex<Connection>>;

/// Resolve `%APPDATA%\ClipSnap\history.db` on Windows, or the platform
/// equivalent on other OSes (useful for `cargo run` on macOS/Linux).
pub fn default_db_path() -> Result<PathBuf> {
    let mut dir = dirs::data_dir().context("no platform data dir available")?;
    dir.push("ClipSnap");
    std::fs::create_dir_all(&dir)
        .with_context(|| format!("failed to create data dir {}", dir.display()))?;
    dir.push("history.db");
    Ok(dir)
}

pub fn open(path: &PathBuf) -> Result<DbHandle> {
    let conn = Connection::open(path)
        .with_context(|| format!("failed to open sqlite at {}", path.display()))?;
    conn.pragma_update(None, "journal_mode", "WAL")?;
    conn.pragma_update(None, "synchronous", "NORMAL")?;
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS entries (
            id            INTEGER PRIMARY KEY AUTOINCREMENT,
            content_type  TEXT    NOT NULL,
            content_text  TEXT,
            content_data  BLOB,
            hash          TEXT    NOT NULL UNIQUE,
            byte_size     INTEGER NOT NULL,
            created_at    INTEGER NOT NULL,
            last_used_at  INTEGER NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_last_used ON entries(last_used_at DESC);
        CREATE INDEX IF NOT EXISTS idx_hash ON entries(hash);
        "#,
    )?;
    Ok(Arc::new(Mutex::new(conn)))
}

pub fn hash_payload(content_type: ContentType, data: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content_type.as_str().as_bytes());
    hasher.update(b"\x00");
    hasher.update(data.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Insert a new clip, or bump `last_used_at` if its hash already exists.
/// Returns the row id of the affected entry.
pub fn upsert_clip(db: &DbHandle, clip: &NewClip) -> Result<i64> {
    let now = Utc::now().timestamp_millis();
    let hash = hash_payload(clip.content_type, &clip.content_data);
    let conn = db.lock();

    let existing: Option<i64> = conn
        .query_row(
            "SELECT id FROM entries WHERE hash = ?1",
            params![&hash],
            |row| row.get(0),
        )
        .optional()?;

    let id = if let Some(id) = existing {
        conn.execute(
            "UPDATE entries SET last_used_at = ?1 WHERE id = ?2",
            params![now, id],
        )?;
        id
    } else {
        // `content_text` and `content_data` may contain passwords,
        // tokens, file paths, image bytes — encrypt at rest. `hash` is
        // computed over plaintext (kept plaintext for dedup) and
        // doesn't reveal content beyond duplicate-presence.
        let enc_text = crypto::encrypt(&clip.content_text);
        let enc_data = crypto::encrypt(&clip.content_data);
        conn.execute(
            r#"
            INSERT INTO entries (
                content_type, content_text, content_data, hash,
                byte_size, created_at, last_used_at
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            "#,
            params![
                clip.content_type.as_str(),
                enc_text,
                enc_data,
                hash,
                clip.byte_size,
                now,
                now,
            ],
        )?;
        conn.last_insert_rowid()
    };

    prune_locked(&conn, MAX_ENTRIES)?;
    Ok(id)
}

fn prune_locked(conn: &Connection, keep: i64) -> Result<()> {
    conn.execute(
        r#"
        DELETE FROM entries
        WHERE id IN (
            SELECT id FROM entries
            ORDER BY last_used_at DESC
            LIMIT -1 OFFSET ?1
        )
        "#,
        params![keep],
    )?;
    Ok(())
}

pub fn list(db: &DbHandle, limit: usize, offset: usize) -> Result<Vec<ClipEntry>> {
    let conn = db.lock();
    let mut stmt = conn.prepare(
        r#"
        SELECT id, content_type, content_text, content_data, hash,
               byte_size, created_at, last_used_at
        FROM entries
        ORDER BY last_used_at DESC
        LIMIT ?1 OFFSET ?2
        "#,
    )?;
    let rows = stmt.query_map(params![limit as i64, offset as i64], row_to_entry)?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r?);
    }
    Ok(out)
}

pub fn touch(db: &DbHandle, id: i64) -> Result<()> {
    let now = Utc::now().timestamp_millis();
    let conn = db.lock();
    conn.execute(
        "UPDATE entries SET last_used_at = ?1 WHERE id = ?2",
        params![now, id],
    )?;
    Ok(())
}

pub fn get(db: &DbHandle, id: i64) -> Result<Option<ClipEntry>> {
    let conn = db.lock();
    let entry = conn
        .query_row(
            r#"
            SELECT id, content_type, content_text, content_data, hash,
                   byte_size, created_at, last_used_at
            FROM entries
            WHERE id = ?1
            "#,
            params![id],
            row_to_entry,
        )
        .optional()?;
    Ok(entry)
}

pub fn delete(db: &DbHandle, id: i64) -> Result<()> {
    let conn = db.lock();
    conn.execute("DELETE FROM entries WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn clear(db: &DbHandle) -> Result<()> {
    let conn = db.lock();
    conn.execute("DELETE FROM entries", [])?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::prune_locked;
    use crate::models::{ContentType, NewClip};
    use parking_lot::Mutex;
    use rusqlite::Connection;
    use std::sync::Arc;

    fn test_db() -> DbHandle {
        let conn = Connection::open_in_memory().expect("in-memory db");
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS entries (
                id            INTEGER PRIMARY KEY AUTOINCREMENT,
                content_type  TEXT    NOT NULL,
                content_text  TEXT,
                content_data  BLOB,
                hash          TEXT    NOT NULL UNIQUE,
                byte_size     INTEGER NOT NULL,
                created_at    INTEGER NOT NULL,
                last_used_at  INTEGER NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_last_used ON entries(last_used_at DESC);
            CREATE INDEX IF NOT EXISTS idx_hash ON entries(hash);
            "#,
        )
        .unwrap();
        Arc::new(Mutex::new(conn))
    }

    fn text_clip(s: &str) -> NewClip {
        NewClip {
            content_type: ContentType::Text,
            content_text: s.to_string(),
            content_data: s.to_string(),
            byte_size: s.len() as i64,
        }
    }

    #[test]
    fn hash_payload_is_deterministic() {
        let h1 = hash_payload(ContentType::Text, "hello");
        let h2 = hash_payload(ContentType::Text, "hello");
        assert_eq!(h1, h2);
    }

    #[test]
    fn hash_payload_differs_by_content_type() {
        let h1 = hash_payload(ContentType::Text, "hello");
        let h2 = hash_payload(ContentType::Html, "hello");
        assert_ne!(h1, h2);
    }

    #[test]
    fn hash_payload_differs_by_data() {
        let h1 = hash_payload(ContentType::Text, "hello");
        let h2 = hash_payload(ContentType::Text, "world");
        assert_ne!(h1, h2);
    }

    #[test]
    fn hash_payload_is_hex_string() {
        let h = hash_payload(ContentType::Text, "test");
        assert!(h.chars().all(|c| c.is_ascii_hexdigit()), "not hex: {h}");
        assert_eq!(h.len(), 64, "SHA-256 should be 64 hex chars");
    }

    #[test]
    fn upsert_and_list_round_trip() {
        let db = test_db();
        upsert_clip(&db, &text_clip("hello")).unwrap();
        let entries = list(&db, 10, 0).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].content_text, "hello");
        assert_eq!(entries[0].content_type, ContentType::Text);
    }

    #[test]
    fn upsert_deduplicates_identical_content() {
        let db = test_db();
        let id1 = upsert_clip(&db, &text_clip("dup")).unwrap();
        let id2 = upsert_clip(&db, &text_clip("dup")).unwrap();
        assert_eq!(id1, id2);
        assert_eq!(list(&db, 10, 0).unwrap().len(), 1);
    }

    #[test]
    fn upsert_stores_distinct_entries_separately() {
        let db = test_db();
        upsert_clip(&db, &text_clip("a")).unwrap();
        upsert_clip(&db, &text_clip("b")).unwrap();
        assert_eq!(list(&db, 10, 0).unwrap().len(), 2);
    }

    #[test]
    fn get_returns_correct_entry() {
        let db = test_db();
        let id = upsert_clip(&db, &text_clip("find me")).unwrap();
        let entry = get(&db, id).unwrap().unwrap();
        assert_eq!(entry.id, id);
        assert_eq!(entry.content_text, "find me");
    }

    #[test]
    fn get_returns_none_for_missing_id() {
        let db = test_db();
        assert!(get(&db, 9999).unwrap().is_none());
    }

    #[test]
    fn delete_removes_entry() {
        let db = test_db();
        let id = upsert_clip(&db, &text_clip("to delete")).unwrap();
        delete(&db, id).unwrap();
        assert!(get(&db, id).unwrap().is_none());
        assert_eq!(list(&db, 10, 0).unwrap().len(), 0);
    }

    #[test]
    fn clear_removes_all_entries() {
        let db = test_db();
        upsert_clip(&db, &text_clip("a")).unwrap();
        upsert_clip(&db, &text_clip("b")).unwrap();
        upsert_clip(&db, &text_clip("c")).unwrap();
        clear(&db).unwrap();
        assert_eq!(list(&db, 10, 0).unwrap().len(), 0);
    }

    #[test]
    fn list_respects_limit() {
        let db = test_db();
        for i in 0..5 {
            upsert_clip(&db, &text_clip(&format!("item {i}"))).unwrap();
        }
        assert_eq!(list(&db, 2, 0).unwrap().len(), 2);
    }

    #[test]
    fn list_respects_offset() {
        let db = test_db();
        for i in 0..5 {
            upsert_clip(&db, &text_clip(&format!("item {i}"))).unwrap();
        }
        assert_eq!(list(&db, 10, 4).unwrap().len(), 1);
        assert_eq!(list(&db, 10, 5).unwrap().len(), 0);
    }

    #[test]
    fn touch_does_not_fail_on_valid_id() {
        let db = test_db();
        let id = upsert_clip(&db, &text_clip("touchable")).unwrap();
        touch(&db, id).unwrap();
    }

    #[test]
    fn prune_removes_oldest_entries_over_cap() {
        let db = test_db();
        // Insert 5 entries, then prune to 3
        for i in 0..5 {
            upsert_clip(&db, &text_clip(&format!("item {i}"))).unwrap();
        }
        {
            let conn = db.lock();
            prune_locked(&conn, 3).unwrap();
        }
        assert_eq!(list(&db, 10, 0).unwrap().len(), 3);
    }
}

fn row_to_entry(row: &rusqlite::Row<'_>) -> rusqlite::Result<ClipEntry> {
    let ct_str: String = row.get(1)?;
    let content_type = ContentType::from_str(&ct_str).unwrap_or(ContentType::Text);
    let raw_text = row.get::<_, Option<String>>(2)?.unwrap_or_default();
    let raw_data = row.get::<_, Option<String>>(3)?.unwrap_or_default();
    Ok(ClipEntry {
        id: row.get(0)?,
        content_type,
        content_text: crypto::decrypt(&raw_text),
        content_data: crypto::decrypt(&raw_data),
        hash: row.get(4)?,
        byte_size: row.get(5)?,
        created_at: row.get(6)?,
        last_used_at: row.get(7)?,
    })
}
