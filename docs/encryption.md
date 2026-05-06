# At-rest encryption

Introduced in **v0.6.0**.

Sensitive content stored in `history.db` is encrypted with **AES-256-GCM**. The previous limitation — anyone with read access to your profile could open the SQLite file and see every clipboard text, snippet body, and note in plaintext — is closed.

## Threat model

This protects against:

- **Other applications** running as your user that try to read the DB directly.
- **Backups** of your home directory (e.g., Time Machine, Dropbox, rsync to NAS) silently capturing the file.
- **A drive image** taken without your login session (the image carries the file but not the unlocked keychain).
- **Casual filesystem-level access** — `cat history.db | strings` on a USB stick, a forensics tool dump, etc.

This does **not** protect against:

- An attacker with **active code execution as your user** while you're logged in. They can read the keychain entry and the keyfile fallback. Encryption is one layer; sandboxing / process isolation is a different one.
- Cloud providers given your full backup *plus* keychain (e.g., iCloud Backup with Keychain Sync). If you trust the cloud you trust both, and they decrypt together.
- Memory dumps while ClipSnap is running (the cipher key and decrypted strings are in process memory).

## What's encrypted

| Table | Columns | Why |
|---|---|---|
| `entries` | `content_text`, `content_data` | The clipboard payload itself — text, HTML, RTF, base64 of image bytes, JSON of file paths. |
| `snippets` | `body` | Snippet templates often contain emails, signatures, API keys, "boilerplate" auth strings. |
| `notes` | `content_text`, `content_data` | Same content types as entries — anything you bookmarked from the clipboard plus from-scratch text notes. |

## What stays plaintext

By design — these are metadata that don't reveal the actual clipboard content:

| Field | Reason |
|---|---|
| Timestamps (`created_at`, `last_used_at`, `updated_at`) | Used for sorting, ordering, dedup tie-breaks. Doesn't leak content. |
| `content_type` (`text` / `html` / `rtf` / `image` / `files`) | Needed for routing logic (preview pane, paste shortcut). 5-element vocabulary, no information value. |
| `hash` (SHA-256 of plaintext) | Used for deduplication. Reveals "did you copy this same string twice" but not the string itself. |
| `byte_size` | UI display ("1.6 KB"). Already implicitly leaked by ciphertext size. |
| `snippets.abbreviation`, `snippets.title` | These are how you *find* a snippet. Encrypting them would mean either decrypting the whole table on every search (slow) or building a separate searchable index (complex). The body is the sensitive part anyway. |
| `notes.title`, `notes.category` | Same reasoning — they're how you navigate; the body is what you wanted protected. |

## Storage format

Each encrypted value is stored as TEXT prefixed with a version marker:

```text
"v1:" + base64( 12-byte random nonce || aes-gcm ciphertext+tag )
```

- **Prefix `v1:`** — explicit version. Future format changes get `v2:` and the decrypt path can dispatch on it.
- **Random nonce** — fresh 12 bytes per encryption. Required for AES-GCM safety; same plaintext encrypts to a different ciphertext every time.
- **Auth tag** — built into AES-GCM; appended to the ciphertext by the `aes-gcm` crate. Tampering is detected and decryption returns an error.
- **Base64** — keeps the column TEXT (no schema migration to BLOB). ~33% size overhead, fine for our 1000-entry cap.

Legacy plaintext rows (no `v1:` prefix) are detected on read and returned as-is, so the migration step can run lazily without breaking anything in between.

## Key storage

A single 256-bit AES key per install. On first launch the app:

1. Tries to read the key from the **OS keychain**:
   - macOS: Keychain (`Security.framework`)
   - Windows: Credential Manager (`wincred.h`)
   - Linux: Secret Service (GNOME Keyring / KWallet)

   Service `io.celox.clipsnap`, account `history-db-key-v1`. The crate that abstracts all three is [`keyring`](https://crates.io/crates/keyring) v3.

2. Falls back to a **0600 keyfile** at `<data-dir>/.dbkey`:
   - macOS: `~/Library/Application Support/ClipSnap/.dbkey`
   - Windows: `%APPDATA%\ClipSnap\.dbkey`
   - Linux: `~/.local/share/ClipSnap/.dbkey`

3. If neither exists yet, generates a fresh random key with `rand::thread_rng().fill_bytes(...)`, stores it in *both* the keychain and the keyfile.

Why both? Robustness. The keychain is the secure store; the keyfile is a backup that lets the app keep working if the keychain is locked or unavailable (notably on Linux without a Secret Service implementation, or after a corrupted keychain).

> **Caveat.** Storing the key in a 0600 keyfile next to the DB means the *file-system-access* attacker we wanted to defend against can in fact get it. The keychain is the real defence; the keyfile is graceful-degradation. If you want strict protection, ensure your keychain is initialised before first launch and remove the keyfile manually after.

## Migration

Existing v0.5.x users have a plaintext DB. On the first launch with v0.6.0+:

1. App opens the DB normally.
2. After the schema setup, the app walks `entries`, `snippets`, `notes` and re-encrypts every row whose target column doesn't already start with `v1:`.
3. Migration is **idempotent** — a second run finds all rows already prefixed with `v1:` and does nothing.

Migration runs in the same WAL transaction as the rest of startup. There's no separate "encrypted_v1" flag because the prefix self-identifies; this means there's nothing to undo if you ever want to roll back to v0.5.x. (You'd lose access to encrypted rows because v0.5.x doesn't know how to decrypt them. Don't roll back.)

## Backup compatibility

The Settings → Backup → Export flow writes a **plaintext JSON file**, not an encrypted dump. Reasoning:

- Backups are user-controlled — they choose where the file lands. If they need it encrypted, they encrypt the JSON themselves (`gpg --symmetric backup.json`, ZIP-with-password, etc.).
- A backup that re-uses the install's AES key would be useless on any other machine or after a key reset.
- A backup format with its own crypto would force the user to remember another passphrase.

When importing a v0.6.0 backup file, the values get re-encrypted on the way in with the *target* install's key. Backups are interoperable across installs without reusing keys.

## What if I lose access?

Three loss scenarios, ranked by survivability:

1. **Lost the keychain entry only** (e.g. `tccutil` reset, keychain corruption). The keyfile fallback at `<data-dir>/.dbkey` still has the key. The app reads from there on next launch and re-stores it in the keychain.
2. **Lost the keyfile only** (e.g., manual delete, partial restore from backup). Keychain still has it; the app re-writes the keyfile.
3. **Lost both** (full disk wipe, reset, etc.). The encrypted rows are unrecoverable. ClipSnap will boot, generate a fresh key, and treat the existing rows as undecryptable garbage. They show up as empty / corrupted text.
   - Mitigation: take a Backup → Export *before* any reset. The export writes plaintext that survives key loss.

## Implementation

Source: [`core/rust-lib/src/crypto.rs`](../core/rust-lib/src/crypto.rs) (~280 LOC, 6 unit tests).

Tests cover:

- Roundtrip — `encrypt(s)` → `decrypt(...)` → `s`.
- Legacy plaintext passthrough — strings without the `v1:` prefix are returned unchanged on `decrypt`.
- Empty string roundtrip.
- Fresh-nonce — same plaintext encrypts to two different ciphertexts on consecutive calls.
- Tampered-ciphertext rejection — flipping a base64 byte produces a decryption error.
- Wrong-key rejection — decryption fails when the cipher was built with a different key.

The encrypt/decrypt path is wired into `db.rs::row_to_entry`, `snippets.rs::row_to_snippet`, `notes.rs::row_to_note` (read paths), and into the corresponding `INSERT` / `UPDATE` statements (write paths). When the global cipher isn't initialised — only happens in unit tests that build in-memory DBs without app setup — encrypt/decrypt are no-ops, so existing tests continue to work without mocking.

## Why not SQLCipher?

[SQLCipher](https://www.zetetic.net/sqlcipher/) is the standard SQLite encryption layer. It's transparent, AES-256, battle-tested. Trade-offs that pushed us to column-level instead:

- **Build complexity.** SQLCipher requires switching from `rusqlite` with bundled SQLite to `rusqlite` with `bundled-sqlcipher-vendored-openssl`. That vendors OpenSSL, adds 30+ seconds to clean compiles, and doubles the binary size.
- **Full-file encryption is heavier than we need.** Schema, table names, indexes, and metadata don't need protecting; only content does. Encrypting only the sensitive columns is faster and lets timestamps / type tags / hashes stay queryable.
- **Migration would require a full DB rewrite.** SQLCipher can't be enabled on an existing plaintext DB — you'd have to `ATTACH` an encrypted DB and copy. That's a one-time operation but it doubles peak disk usage.
- **Dump diagnostics still work.** With column-level encryption, `sqlite3 history.db ".schema"` and `SELECT id, content_type, hash FROM entries` still produce useful output. SQLCipher's dump is opaque.

If a future version needs full-file encryption (e.g., to also hide the schema / abbreviation list from filesystem readers), SQLCipher is the obvious upgrade path.

## See also

- [`docs/backup.md`](./backup.md) — how to back up your data; the relationship between encryption and the JSON export format.
- [`docs/spec.md`](./spec.md) — the broader storage architecture this fits into.
- [`core/rust-lib/src/crypto.rs`](../core/rust-lib/src/crypto.rs) — the implementation.
