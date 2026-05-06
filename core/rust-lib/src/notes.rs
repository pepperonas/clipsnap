use anyhow::{anyhow, Result};
use chrono::Utc;
use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};

use crate::db::DbHandle;
use crate::models::ContentType;

/// A note is a user-curated, persistent copy of a clipboard entry (or a
/// from-scratch text note). Notes live in their own table so they are not
/// affected by the 1 000-entry pruning of `entries`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub id: i64,
    pub content_type: ContentType,
    pub content_text: String,
    pub content_data: String,
    pub title: String,
    pub category: String,
    pub byte_size: i64,
    pub created_at: i64,
    pub updated_at: i64,
}

pub fn init_table(db: &DbHandle) -> Result<()> {
    let conn = db.lock();
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS notes (
            id           INTEGER PRIMARY KEY AUTOINCREMENT,
            content_type TEXT    NOT NULL,
            content_text TEXT    NOT NULL DEFAULT '',
            content_data TEXT    NOT NULL DEFAULT '',
            title        TEXT    NOT NULL DEFAULT '',
            category     TEXT    NOT NULL DEFAULT '',
            byte_size    INTEGER NOT NULL DEFAULT 0,
            created_at   INTEGER NOT NULL,
            updated_at   INTEGER NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_notes_category ON notes(category);
        CREATE INDEX IF NOT EXISTS idx_notes_updated  ON notes(updated_at DESC);
        "#,
    )?;
    Ok(())
}

pub fn list_all(db: &DbHandle) -> Result<Vec<Note>> {
    let conn = db.lock();
    let mut stmt = conn.prepare(
        r#"
        SELECT id, content_type, content_text, content_data,
               title, category, byte_size, created_at, updated_at
        FROM notes
        ORDER BY updated_at DESC
        "#,
    )?;
    let rows = stmt.query_map([], row_to_note)?;
    rows.collect::<rusqlite::Result<Vec<_>>>().map_err(Into::into)
}

/// Distinct, non-empty categories sorted alphabetically (case-insensitive).
pub fn list_categories(db: &DbHandle) -> Result<Vec<String>> {
    let conn = db.lock();
    let mut stmt = conn.prepare(
        r#"
        SELECT DISTINCT category FROM notes
        WHERE category != ''
        ORDER BY LOWER(category) ASC
        "#,
    )?;
    let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
    rows.collect::<rusqlite::Result<Vec<_>>>().map_err(Into::into)
}

/// Create a fresh text-only note. For from-scratch notes in the UI.
pub fn create_text(db: &DbHandle, title: &str, body: &str, category: &str) -> Result<i64> {
    let now = Utc::now().timestamp_millis();
    let body_len = body.len() as i64;
    let enc_body = crate::crypto::encrypt(body);
    let conn = db.lock();
    conn.execute(
        r#"
        INSERT INTO notes (
            content_type, content_text, content_data,
            title, category, byte_size, created_at, updated_at
        ) VALUES ('text', ?1, ?1, ?2, ?3, ?4, ?5, ?5)
        "#,
        params![enc_body, title.trim(), category.trim(), body_len, now],
    )?;
    Ok(conn.last_insert_rowid())
}

/// Save a clipboard entry as a note. Copies the raw payload so the note
/// survives history pruning. Returns the note's id (or `None` if the clip
/// did not exist).
pub fn save_from_clip(
    db: &DbHandle,
    clip_id: i64,
    title: &str,
    category: &str,
) -> Result<Option<i64>> {
    let now = Utc::now().timestamp_millis();
    let conn = db.lock();

    // Read the clip; if it's gone (just got pruned), surface gracefully.
    let row: Option<(String, String, String, i64)> = conn
        .query_row(
            r#"
            SELECT content_type, content_text, content_data, byte_size
            FROM entries WHERE id = ?1
            "#,
            params![clip_id],
            |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, Option<String>>(1)?.unwrap_or_default(),
                    r.get::<_, Option<String>>(2)?.unwrap_or_default(),
                    r.get::<_, i64>(3)?,
                ))
            },
        )
        .optional()?;
    let Some((ct, text, data, bytes)) = row else {
        return Ok(None);
    };

    // `text` and `data` came out of `entries` *already* encrypted (we
    // didn't run them through `row_to_entry`'s decrypt). Both tables
    // use the same cipher, so passing the ciphertext straight into
    // `notes.content_text` / `notes.content_data` is correct — the
    // value will round-trip cleanly when `row_to_note` decrypts it.
    conn.execute(
        r#"
        INSERT INTO notes (
            content_type, content_text, content_data,
            title, category, byte_size, created_at, updated_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?7)
        "#,
        params![ct, text, data, title.trim(), category.trim(), bytes, now],
    )?;
    Ok(Some(conn.last_insert_rowid()))
}

/// Update a note. Body edits are accepted only for text/html/rtf — for
/// image/files notes the body parameter is ignored (caller can still update
/// title and category).
pub fn update(
    db: &DbHandle,
    id: i64,
    title: &str,
    body: &str,
    category: &str,
) -> Result<()> {
    let now = Utc::now().timestamp_millis();
    let conn = db.lock();

    let ct: Option<String> = conn
        .query_row(
            "SELECT content_type FROM notes WHERE id = ?1",
            params![id],
            |r| r.get(0),
        )
        .optional()?;
    let Some(ct) = ct else {
        return Err(anyhow!("note {id} not found"));
    };

    let editable = matches!(ct.as_str(), "text" | "html" | "rtf");
    if editable {
        let body_len = body.len() as i64;
        let enc_body = crate::crypto::encrypt(body);
        conn.execute(
            r#"
            UPDATE notes
            SET title = ?1, category = ?2,
                content_text = ?3, content_data = ?3,
                byte_size = ?4, updated_at = ?5
            WHERE id = ?6
            "#,
            params![title.trim(), category.trim(), enc_body, body_len, now, id],
        )?;
    } else {
        conn.execute(
            r#"
            UPDATE notes
            SET title = ?1, category = ?2, updated_at = ?3
            WHERE id = ?4
            "#,
            params![title.trim(), category.trim(), now, id],
        )?;
    }
    Ok(())
}

pub fn delete(db: &DbHandle, id: i64) -> Result<()> {
    let conn = db.lock();
    conn.execute("DELETE FROM notes WHERE id = ?1", params![id])?;
    Ok(())
}

pub fn clear_all(db: &DbHandle) -> Result<()> {
    let conn = db.lock();
    conn.execute("DELETE FROM notes", [])?;
    Ok(())
}

pub fn get(db: &DbHandle, id: i64) -> Result<Option<Note>> {
    let conn = db.lock();
    let note = conn
        .query_row(
            r#"
            SELECT id, content_type, content_text, content_data,
                   title, category, byte_size, created_at, updated_at
            FROM notes WHERE id = ?1
            "#,
            params![id],
            row_to_note,
        )
        .optional()?;
    Ok(note)
}

/// Append-import — used by the backup-restore flow. Returns the new id.
/// Timestamps and content from the payload are preserved verbatim so the
/// roundtrip is lossless.
pub fn append_imported(
    db: &DbHandle,
    note: &Note,
) -> Result<i64> {
    // The note we're given has plaintext bodies (the JSON backup
    // format is plaintext). Encrypt before storing so the on-disk
    // shape matches everything else in this table.
    let enc_text = crate::crypto::encrypt(&note.content_text);
    let enc_data = crate::crypto::encrypt(&note.content_data);
    let conn = db.lock();
    conn.execute(
        r#"
        INSERT INTO notes (
            content_type, content_text, content_data,
            title, category, byte_size, created_at, updated_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
        "#,
        params![
            note.content_type.as_str(),
            enc_text,
            enc_data,
            note.title.trim(),
            note.category.trim(),
            note.byte_size,
            note.created_at,
            note.updated_at,
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

fn row_to_note(row: &rusqlite::Row<'_>) -> rusqlite::Result<Note> {
    let ct_str: String = row.get(1)?;
    let content_type = ContentType::from_str(&ct_str).unwrap_or(ContentType::Text);
    let raw_text = row.get::<_, Option<String>>(2)?.unwrap_or_default();
    let raw_data = row.get::<_, Option<String>>(3)?.unwrap_or_default();
    Ok(Note {
        id: row.get(0)?,
        content_type,
        content_text: crate::crypto::decrypt(&raw_text),
        content_data: crate::crypto::decrypt(&raw_data),
        title: row.get(4)?,
        category: row.get(5)?,
        byte_size: row.get(6)?,
        created_at: row.get(7)?,
        updated_at: row.get(8)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::NewClip;
    use parking_lot::Mutex;
    use rusqlite::Connection;
    use std::sync::Arc;

    /// Build an in-memory db with both `entries` and `notes` tables — needed
    /// because save_from_clip reads from entries.
    fn test_db() -> DbHandle {
        let conn = Connection::open_in_memory().expect("in-memory db");
        conn.execute_batch(
            r#"
            CREATE TABLE entries (
                id            INTEGER PRIMARY KEY AUTOINCREMENT,
                content_type  TEXT    NOT NULL,
                content_text  TEXT,
                content_data  BLOB,
                hash          TEXT    NOT NULL UNIQUE,
                byte_size     INTEGER NOT NULL,
                created_at    INTEGER NOT NULL,
                last_used_at  INTEGER NOT NULL
            );
            "#,
        )
        .unwrap();
        let db = Arc::new(Mutex::new(conn));
        init_table(&db).unwrap();
        db
    }

    fn add_clip(db: &DbHandle, text: &str) -> i64 {
        let now = Utc::now().timestamp_millis();
        let conn = db.lock();
        let _clip = NewClip {
            content_type: ContentType::Text,
            content_text: text.to_string(),
            content_data: text.to_string(),
            byte_size: text.len() as i64,
        };
        conn.execute(
            r#"
            INSERT INTO entries (
                content_type, content_text, content_data, hash,
                byte_size, created_at, last_used_at
            ) VALUES ('text', ?1, ?1, ?2, ?3, ?4, ?4)
            "#,
            params![text, format!("h-{text}"), text.len() as i64, now],
        )
        .unwrap();
        conn.last_insert_rowid()
    }

    #[test]
    fn create_text_inserts_a_note() {
        let db = test_db();
        let id = create_text(&db, "Hi", "hello world", "Greetings").unwrap();
        let note = get(&db, id).unwrap().unwrap();
        assert_eq!(note.title, "Hi");
        assert_eq!(note.content_text, "hello world");
        assert_eq!(note.content_data, "hello world");
        assert_eq!(note.category, "Greetings");
        assert_eq!(note.content_type, ContentType::Text);
        assert_eq!(note.byte_size, 11);
    }

    #[test]
    fn save_from_clip_copies_payload_and_returns_id() {
        let db = test_db();
        let clip_id = add_clip(&db, "snapshot");
        let note_id = save_from_clip(&db, clip_id, "", "")
            .unwrap()
            .expect("note id");
        let note = get(&db, note_id).unwrap().unwrap();
        assert_eq!(note.content_text, "snapshot");
        assert_eq!(note.category, "");
        assert_eq!(note.title, "");
    }

    #[test]
    fn save_from_clip_returns_none_when_clip_missing() {
        let db = test_db();
        let r = save_from_clip(&db, 9999, "", "").unwrap();
        assert!(r.is_none());
    }

    #[test]
    fn list_all_orders_by_updated_at_desc() {
        let db = test_db();
        let _a = create_text(&db, "a", "a", "").unwrap();
        // ensure a different timestamp by sleeping a millisecond's worth
        std::thread::sleep(std::time::Duration::from_millis(2));
        let b = create_text(&db, "b", "b", "").unwrap();
        let notes = list_all(&db).unwrap();
        assert_eq!(notes.len(), 2);
        assert_eq!(notes[0].id, b, "newest first");
    }

    #[test]
    fn list_categories_returns_distinct_non_empty_sorted() {
        let db = test_db();
        create_text(&db, "", "1", "Work").unwrap();
        create_text(&db, "", "2", "personal").unwrap();
        create_text(&db, "", "3", "Work").unwrap();
        create_text(&db, "", "4", "").unwrap();
        let cats = list_categories(&db).unwrap();
        // Sorted case-insensitive, no duplicates, no empty.
        assert_eq!(cats, vec!["personal".to_string(), "Work".to_string()]);
    }

    #[test]
    fn update_changes_title_body_and_category() {
        let db = test_db();
        let id = create_text(&db, "old", "old body", "A").unwrap();
        update(&db, id, "new", "new body", "B").unwrap();
        let note = get(&db, id).unwrap().unwrap();
        assert_eq!(note.title, "new");
        assert_eq!(note.content_text, "new body");
        assert_eq!(note.content_data, "new body");
        assert_eq!(note.category, "B");
        assert_eq!(note.byte_size, "new body".len() as i64);
    }

    #[test]
    fn update_ignores_body_for_image_notes() {
        let db = test_db();
        let now = Utc::now().timestamp_millis();
        // Manually craft an image note (base64 string in content_data).
        {
            let conn = db.lock();
            conn.execute(
                r#"
                INSERT INTO notes (
                    content_type, content_text, content_data,
                    title, category, byte_size, created_at, updated_at
                ) VALUES ('image', '', 'BASE64', '', '', 100, ?1, ?1)
                "#,
                params![now],
            )
            .unwrap();
        }
        let id = 1;
        update(&db, id, "captioned", "should be ignored", "Screenshots").unwrap();
        let note = get(&db, id).unwrap().unwrap();
        assert_eq!(note.title, "captioned");
        assert_eq!(note.category, "Screenshots");
        // Image data must not have been touched.
        assert_eq!(note.content_data, "BASE64");
        assert_eq!(note.byte_size, 100);
    }

    #[test]
    fn delete_removes_note() {
        let db = test_db();
        let id = create_text(&db, "x", "y", "").unwrap();
        delete(&db, id).unwrap();
        assert!(get(&db, id).unwrap().is_none());
    }

    #[test]
    fn clear_all_removes_every_note() {
        let db = test_db();
        create_text(&db, "1", "a", "").unwrap();
        create_text(&db, "2", "b", "X").unwrap();
        clear_all(&db).unwrap();
        assert!(list_all(&db).unwrap().is_empty());
    }

    #[test]
    fn append_imported_preserves_timestamps_and_payload() {
        let db = test_db();
        let n = Note {
            id: 0, // ignored
            content_type: ContentType::Text,
            content_text: "imported".into(),
            content_data: "imported".into(),
            title: "T".into(),
            category: "C".into(),
            byte_size: 8,
            created_at: 1_700_000_000_000,
            updated_at: 1_700_000_001_000,
        };
        let id = append_imported(&db, &n).unwrap();
        let got = get(&db, id).unwrap().unwrap();
        assert_eq!(got.content_text, "imported");
        assert_eq!(got.created_at, 1_700_000_000_000);
        assert_eq!(got.updated_at, 1_700_000_001_000);
    }
}
