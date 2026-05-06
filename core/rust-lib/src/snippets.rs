use anyhow::{anyhow, Result};
use chrono::Utc;
use rusqlite::{params, OptionalExtension};
use serde::{Deserialize, Serialize};

use crate::db::DbHandle;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snippet {
    pub id: i64,
    pub abbreviation: String,
    pub title: String,
    pub body: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Default, Clone, Serialize)]
pub struct ImportResult {
    /// Rows successfully written (insert or update by abbreviation).
    pub imported: usize,
    /// Rows that failed validation; details in `errors`.
    pub skipped: usize,
    /// Per-row error messages, prefixed with the index in the input.
    pub errors: Vec<String>,
}

pub fn init_table(db: &DbHandle) -> Result<()> {
    let conn = db.lock();
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS snippets (
            id           INTEGER PRIMARY KEY AUTOINCREMENT,
            abbreviation TEXT    NOT NULL UNIQUE,
            title        TEXT    NOT NULL DEFAULT '',
            body         TEXT    NOT NULL,
            created_at   INTEGER NOT NULL,
            updated_at   INTEGER NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_snippets_abbr ON snippets(abbreviation);
        "#,
    )?;
    Ok(())
}

pub fn list_all(db: &DbHandle) -> Result<Vec<Snippet>> {
    let conn = db.lock();
    let mut stmt = conn.prepare(
        "SELECT id, abbreviation, title, body, created_at, updated_at \
         FROM snippets ORDER BY abbreviation ASC",
    )?;
    let rows = stmt.query_map([], row_to_snippet)?;
    rows.collect::<rusqlite::Result<Vec<_>>>().map_err(Into::into)
}

/// Look up a single snippet by its abbreviation. Returns the case-sensitive
/// match if one exists; otherwise falls back to a case-insensitive match
/// (so the text expander matches `MFG` to a snippet stored as `mfg` if no
/// `MFG` exists). Empty / whitespace input returns `None`.
pub fn find_by_exact_abbreviation(db: &DbHandle, abbr: &str) -> Result<Option<Snippet>> {
    let trimmed = abbr.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }

    let conn = db.lock();
    let exact: Option<Snippet> = conn
        .query_row(
            "SELECT id, abbreviation, title, body, created_at, updated_at \
             FROM snippets WHERE abbreviation = ?1 LIMIT 1",
            params![trimmed],
            row_to_snippet,
        )
        .optional()?;
    if let Some(s) = exact {
        return Ok(Some(s));
    }

    let ci: Option<Snippet> = conn
        .query_row(
            "SELECT id, abbreviation, title, body, created_at, updated_at \
             FROM snippets WHERE LOWER(abbreviation) = LOWER(?1) LIMIT 1",
            params![trimmed],
            row_to_snippet,
        )
        .optional()?;
    Ok(ci)
}

/// Match abbreviation prefix first, then body/title contains — up to 10 results.
pub fn find_by_query(db: &DbHandle, query: &str) -> Result<Vec<Snippet>> {
    if query.is_empty() {
        return Ok(vec![]);
    }
    let q = query.to_lowercase();
    let prefix = format!("{}%", q);
    let contains = format!("%{}%", q);
    let conn = db.lock();
    let mut stmt = conn.prepare(
        r#"
        SELECT id, abbreviation, title, body, created_at, updated_at
        FROM snippets
        WHERE LOWER(abbreviation) LIKE ?1
           OR LOWER(title)        LIKE ?2
           OR LOWER(body)         LIKE ?2
        ORDER BY
            CASE WHEN LOWER(abbreviation) LIKE ?1 THEN 0 ELSE 1 END,
            abbreviation ASC
        LIMIT 10
        "#,
    )?;
    let rows = stmt.query_map(params![prefix, contains], row_to_snippet)?;
    rows.collect::<rusqlite::Result<Vec<_>>>().map_err(Into::into)
}

pub fn create(db: &DbHandle, abbreviation: &str, title: &str, body: &str) -> Result<i64> {
    let now = Utc::now().timestamp_millis();
    let enc_body = crate::crypto::encrypt(body);
    let conn = db.lock();
    conn.execute(
        "INSERT INTO snippets (abbreviation, title, body, created_at, updated_at) \
         VALUES (?1, ?2, ?3, ?4, ?4)",
        params![abbreviation.trim(), title.trim(), enc_body, now],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn update(db: &DbHandle, id: i64, abbreviation: &str, title: &str, body: &str) -> Result<()> {
    let now = Utc::now().timestamp_millis();
    let enc_body = crate::crypto::encrypt(body);
    let conn = db.lock();
    conn.execute(
        "UPDATE snippets SET abbreviation = ?1, title = ?2, body = ?3, updated_at = ?4 \
         WHERE id = ?5",
        params![abbreviation.trim(), title.trim(), enc_body, now, id],
    )?;
    Ok(())
}

pub fn delete(db: &DbHandle, id: i64) -> Result<()> {
    let conn = db.lock();
    conn.execute("DELETE FROM snippets WHERE id = ?1", params![id])?;
    Ok(())
}

/// Insert a snippet by abbreviation, or overwrite the existing row with the
/// same abbreviation. `created_at` is preserved on update.
pub fn upsert_by_abbreviation(
    db: &DbHandle,
    abbreviation: &str,
    title: &str,
    body: &str,
) -> Result<()> {
    let now = Utc::now().timestamp_millis();
    let enc_body = crate::crypto::encrypt(body);
    let conn = db.lock();
    conn.execute(
        r#"
        INSERT INTO snippets (abbreviation, title, body, created_at, updated_at)
        VALUES (?1, ?2, ?3, ?4, ?4)
        ON CONFLICT(abbreviation) DO UPDATE SET
            title      = excluded.title,
            body       = excluded.body,
            updated_at = excluded.updated_at
        "#,
        params![abbreviation.trim(), title.trim(), enc_body, now],
    )?;
    Ok(())
}

/// Shape accepted in the JSON file. `title` is optional for convenience.
#[derive(Debug, Deserialize)]
struct ImportSnippet {
    abbreviation: String,
    #[serde(default)]
    title: String,
    body: String,
}

/// Allow either a top-level array, or `{ "snippets": [...] }`. Anything else
/// produces a parse error.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum ImportPayload {
    Wrapped { snippets: Vec<ImportSnippet> },
    Bare(Vec<ImportSnippet>),
}

impl ImportPayload {
    fn into_snippets(self) -> Vec<ImportSnippet> {
        match self {
            ImportPayload::Wrapped { snippets } => snippets,
            ImportPayload::Bare(v) => v,
        }
    }
}

/// Parse a JSON document and upsert each snippet by abbreviation.
///
/// Accepts two top-level shapes:
/// 1. Bare array: `[{abbreviation, title?, body}, ...]`
/// 2. Wrapped:    `{"snippets": [...]}`
///
/// Per-row failures (empty fields, DB errors) are collected in the result
/// rather than aborting the whole import.
pub fn import_from_json(db: &DbHandle, json: &str) -> Result<ImportResult> {
    let payload: ImportPayload = serde_json::from_str(json)
        .map_err(|e| anyhow!("invalid JSON: {e}"))?;
    let entries = payload.into_snippets();

    let mut result = ImportResult::default();
    for (idx, item) in entries.into_iter().enumerate() {
        let abbr = item.abbreviation.trim();
        let body = item.body.as_str();
        if abbr.is_empty() {
            result.skipped += 1;
            result
                .errors
                .push(format!("#{idx}: missing abbreviation"));
            continue;
        }
        if body.trim().is_empty() {
            result.skipped += 1;
            result
                .errors
                .push(format!("#{idx} ({abbr}): body is empty"));
            continue;
        }
        match upsert_by_abbreviation(db, abbr, &item.title, body) {
            Ok(()) => result.imported += 1,
            Err(e) => {
                result.skipped += 1;
                result.errors.push(format!("#{idx} ({abbr}): {e}"));
            }
        }
    }
    Ok(result)
}

fn row_to_snippet(row: &rusqlite::Row<'_>) -> rusqlite::Result<Snippet> {
    let raw_body: String = row.get(3)?;
    Ok(Snippet {
        id: row.get(0)?,
        abbreviation: row.get(1)?,
        title: row.get(2)?,
        body: crate::crypto::decrypt(&raw_body),
        created_at: row.get(4)?,
        updated_at: row.get(5)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use parking_lot::Mutex;
    use rusqlite::Connection;
    use std::sync::Arc;

    fn test_db() -> DbHandle {
        let conn = Connection::open_in_memory().expect("in-memory db");
        let db = Arc::new(Mutex::new(conn));
        init_table(&db).unwrap();
        db
    }

    #[test]
    fn import_bare_array_inserts_each_row() {
        let db = test_db();
        let json = r#"
            [
                {"abbreviation": "mfg",  "title": "Greeting", "body": "Mit freundlichen Grüßen"},
                {"abbreviation": "addr", "body": "Some Street 1"}
            ]
        "#;
        let r = import_from_json(&db, json).unwrap();
        assert_eq!(r.imported, 2);
        assert_eq!(r.skipped, 0);
        assert!(r.errors.is_empty());
        assert_eq!(list_all(&db).unwrap().len(), 2);
    }

    #[test]
    fn import_wrapped_object_form_works() {
        let db = test_db();
        let json = r#"
            { "snippets": [{"abbreviation": "x", "body": "y"}] }
        "#;
        let r = import_from_json(&db, json).unwrap();
        assert_eq!(r.imported, 1);
        assert_eq!(list_all(&db).unwrap().len(), 1);
    }

    #[test]
    fn import_skips_rows_with_missing_fields() {
        let db = test_db();
        let json = r#"
            [
                {"abbreviation": "ok",   "body": "valid"},
                {"abbreviation": "",     "body": "no abbreviation"},
                {"abbreviation": "noBody","body": ""}
            ]
        "#;
        let r = import_from_json(&db, json).unwrap();
        assert_eq!(r.imported, 1);
        assert_eq!(r.skipped, 2);
        assert_eq!(r.errors.len(), 2);
        assert_eq!(list_all(&db).unwrap().len(), 1);
    }

    #[test]
    fn import_overwrites_existing_abbreviation() {
        let db = test_db();
        create(&db, "mfg", "Old", "Old body").unwrap();

        let json = r#"[{"abbreviation": "mfg", "title": "New", "body": "New body"}]"#;
        let r = import_from_json(&db, json).unwrap();
        assert_eq!(r.imported, 1);

        let rows = list_all(&db).unwrap();
        assert_eq!(rows.len(), 1, "no duplicate row created");
        assert_eq!(rows[0].title, "New");
        assert_eq!(rows[0].body, "New body");
    }

    #[test]
    fn import_invalid_json_returns_err() {
        let db = test_db();
        let r = import_from_json(&db, "not json");
        assert!(r.is_err());
    }

    #[test]
    fn import_trims_abbreviation_whitespace() {
        let db = test_db();
        let json = r#"[{"abbreviation": "  spaced  ", "body": "x"}]"#;
        import_from_json(&db, json).unwrap();
        let rows = list_all(&db).unwrap();
        assert_eq!(rows[0].abbreviation, "spaced");
    }

    #[test]
    fn find_by_exact_abbreviation_case_sensitive_first() {
        let db = test_db();
        create(&db, "MFG", "shouty", "uppercase body").unwrap();
        create(&db, "mfg", "lower", "lowercase body").unwrap();
        // Exact case wins over the case-insensitive fallback.
        let hit = find_by_exact_abbreviation(&db, "MFG").unwrap().unwrap();
        assert_eq!(hit.body, "uppercase body");
        let hit = find_by_exact_abbreviation(&db, "mfg").unwrap().unwrap();
        assert_eq!(hit.body, "lowercase body");
    }

    #[test]
    fn find_by_exact_abbreviation_falls_back_to_ci() {
        let db = test_db();
        create(&db, "mfg", "lower", "lowercase body").unwrap();
        // No "MFG" row exists — falls back to "mfg".
        let hit = find_by_exact_abbreviation(&db, "MFG").unwrap().unwrap();
        assert_eq!(hit.body, "lowercase body");
    }

    #[test]
    fn find_by_exact_abbreviation_returns_none_for_empty_input() {
        let db = test_db();
        create(&db, "mfg", "", "x").unwrap();
        assert!(find_by_exact_abbreviation(&db, "").unwrap().is_none());
        assert!(find_by_exact_abbreviation(&db, "   ").unwrap().is_none());
    }

    #[test]
    fn find_by_exact_abbreviation_trims_whitespace_before_lookup() {
        let db = test_db();
        create(&db, "mfg", "", "x").unwrap();
        let hit = find_by_exact_abbreviation(&db, "  mfg \n").unwrap().unwrap();
        assert_eq!(hit.abbreviation, "mfg");
    }

    #[test]
    fn find_by_exact_abbreviation_returns_none_for_unknown_abbr() {
        let db = test_db();
        create(&db, "mfg", "", "x").unwrap();
        assert!(find_by_exact_abbreviation(&db, "nope").unwrap().is_none());
    }
}
