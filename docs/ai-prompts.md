# Bundled AI prompt snippets

ClipSnap ships with **25 curated AI-prompt templates** that get seeded into your snippet table on first launch. Type the abbreviation in the search field (or use the text expander), press Enter, and the full structured prompt lands in your destination app — ready to hand to an LLM without further massaging.

Introduced in **v0.5.0**.

## Why bundled defaults?

Most LLM workflows live and die by prompt quality. Generic prompts get generic answers. The bundled templates are opinionated: each enforces sections, bullets, and output-format directives that have been shown to produce better-structured output across Claude / GPT / Gemini.

You can edit, delete, or extend them — they're regular snippets stored in the same SQLite table as your own. Nothing magical.

## Where they live

- **Bundled in the binary** via `include_str!` of [`core/rust-lib/src/seed/ai_prompts.json`](../core/rust-lib/src/seed/ai_prompts.json). No external file needed at runtime.
- **Seeded on first launch** by [`core/rust-lib/src/seed.rs`](../core/rust-lib/src/seed.rs)::`maybe_seed_defaults`. The `seed.default_snippets_v1` flag in the settings table tracks whether the seed has run; subsequent launches don't re-import (so user-deleted prompts stay deleted).
- **Restorable** via the Snippets-tab sidebar's **Restore defaults** button (rotate-counter-clockwise icon, next to Import). That re-imports all 25 with upsert-by-abbreviation semantics: your edits to a default get overwritten, but custom snippets with different abbreviations are untouched.

## The 25 prompts

Each row links you to its full source body in [`ai_prompts.json`](../core/rust-lib/src/seed/ai_prompts.json). The "What you get back" column summarises the structured output the prompt asks the LLM for — every prompt enforces sections, bullets, and a final output-format directive.

### Programming / DevX (8)

| Abbreviation | Title | What you get back |
|---|---|---|
| `aiplan` | Software architecture / implementation plan | Structured plan: Context, Approach, Files to modify, Existing utilities to reuse, Verification. Lists *specific* files and reuses prior abstractions instead of starting from scratch. |
| `aireview` | Senior code review | Findings ranked by severity across Correctness, Security, Performance, Maintainability — high-confidence issues only, no style nits. |
| `airefactor` | Refactor for clarity | A behaviour-preserving rewrite plus a diff-level summary of *what* changed and *why*; explicitly forbids feature creep. |
| `airegex` | Regex + explanation | The pattern (flavour-labelled), a token-by-token breakdown, a positive/negative test table, common gotchas. |
| `aisql` | SQL from natural language | The query (formatted), index/EXPLAIN notes, edge-case warnings, sample expected rows. |
| `aitest` | Test cases | Happy path + edge cases + failure modes, in your test framework's idiomatic style. Avoids redundant "test that 1+1==2" bloat. |
| `aimigration` | Safe + reversible DB migration | Phased plan: pre-deploy / deploy / post-deploy steps, rollback recipe, lock-impact estimate, smoke-test queries. |
| `aibench` | Microbenchmark setup | Benchmark code (criterion / vitest bench / pytest-benchmark), what it controls for, comparison baseline, expected-noise notes. |

### Web / Frontend (5)

| Abbreviation | Title | What you get back |
|---|---|---|
| `aithumb` | Open Graph thumbnail | A 1200×630 design spec: typography, contrast, focal hierarchy, fallback rules, A/B angles. Optimised for both small Slack-card and full-size renders. |
| `aimobile` | PWA mobile audit | Findings + fixes for manifest, viewport, touch targets, offline cache, install prompt, perceived perf. |
| `aia11y` | WCAG 2.2 AA audit | Findings ranked by Perceivable / Operable / Understandable / Robust; concrete CSS / ARIA fixes per item. |
| `aiseo` | SEO + meta audit | Indexability, on-page (titles/meta/canonical), content depth, structured data, internal linking — only *real* issues, no padding. |
| `aicomponent` | Reusable UI component | Props table (precise types), composition slots, a11y notes, states (loading/error/empty), test plan, usage examples. |

### IT Security (4)

| Abbreviation | Title | What you get back |
|---|---|---|
| `aithreat` | STRIDE threat model | DFD sketch (ASCII/Mermaid), per-element STRIDE table, top-N prioritised mitigations, residual risks. |
| `aipentest` | Web pentest checklist | OWASP Top 10–mapped checklist tailored to the actual app (skips irrelevant checks), with concrete test commands per item. |
| `aiauth` | Auth design | Threat scope, identity provider choice, session strategy, MFA, endpoint table, authorisation model, OpenAPI sketch. |
| `aigdpr` | GDPR / privacy review | Per-article walkthrough (Art. 6 lawful basis, data minimisation, retention, DSAR, transfers, DPIA trigger). Skips inapplicable areas. |

### Business workflows (4)

| Abbreviation | Title | What you get back |
|---|---|---|
| `aibrief` | Project brief | One-page brief: Problem, Objective, Scope (in/out), Success criteria, Risks. Designed for fast alignment, not as a contract. |
| `airfp` | RFP evaluation matrix | Weighted criteria (sum=100), per-vendor scorecards, hard-fails surfaced, recommendation with second-place rationale. |
| `aiokr` | OKRs for a team / quarter | 3–5 Objectives × 3 Key Results each. KRs are outcome-not-output, measurable, time-bound, with stretch %. |
| `aichange` | Change management plan | What's changing, audience analysis, comms timeline, training plan, rollback recipe, success metrics. |

### Data analysis (3)

| Abbreviation | Title | What you get back |
|---|---|---|
| `aidataq` | Data quality audit | Findings across Completeness / Validity / Uniqueness / Consistency / Timeliness / Accuracy, each with sample queries and remediation. |
| `aiml` | ML pipeline design | Problem classification, data plan, feature engineering, model choice + baseline, evaluation, serving, monitoring, MLOps. |
| `aidashboard` | Dashboard (KPIs + layout) | Audience analysis, KPI hierarchy (north star → leading → lagging), layout sketch, drill-paths, refresh cadence. |

### Architecture / API (1)

| Abbreviation | Title | What you get back |
|---|---|---|
| `aiapi` | REST API design | Resources, endpoint table (verbs/paths/auth/rate limits), error model, idempotency, versioning, OpenAPI 3.1 outline. |

## Using a prompt

Three ways to put a default prompt to work:

### 1. Search field activation
Open the popup (`Ctrl+Shift+V`), type `aiplan`, press Enter. The full prompt body lands in the previously focused app.

### 2. Snippet expander hotkey (Settings → Text expander)
With the expander enabled, type `aiplan` directly in any text field, press your configured hotkey (default `Alt+Backquote` on a German keyboard / `Alt+\`` on US), the abbreviation gets replaced in place with the full prompt.

### 3. Snippets-tab paste button
Open the popup → **Snippets** tab → click any prompt to select → click **Paste** in the detail pane.

## Editing the prompts

Open the **Snippets** tab → click any `ai*` prompt → edit Abbreviation / Title / Body in the right pane → **Save**.

Common customisations:
- Translate to your preferred language (we ship English because it's the highest-floor across LLMs)
- Add `[CONTEXT]` placeholders specific to your codebase / domain
- Tighten section requirements that don't apply to your work

If you wreck a prompt and want it back, click **Restore defaults** in the sidebar — it'll upsert all 25 to their bundled versions while leaving your custom snippets alone.

## Adding your own AI prompts

Same pattern as any other snippet:
- Click **+ New Snippet** in the Snippets tab
- Pick an abbreviation (the `ai*` prefix isn't reserved — use `mfg-en`, `meeting-notes`, whatever fits your workflow)
- Write the body — for AI prompts, structure it the way the bundled ones are: clear sections, bulleted requirements, an explicit output-format directive at the end

Or batch-import via **Import…** with a JSON file matching the snippet schema (see [`docs/snippets-import.md`](./snippets-import.md)).

## Versioning

The bundled set is versioned via the `seed.default_snippets_v1` settings flag. If a future release ships an updated prompt library, the flag suffix changes (`_v2`, etc.), forcing a re-seed for everyone. We'll only do that for genuinely substantial improvements — not for minor wording tweaks — because the re-seed will overwrite users' edits to default-keyed prompts.

If you're maintaining your own version of a default prompt, give it a different abbreviation (`aiplan-mine`, `aiplan-de`) so it survives any future re-seed.

## See also

- [`docs/snippets-import.md`](./snippets-import.md) — bulk JSON import format used by the seed
- [`docs/text-expander.md`](./text-expander.md) — how to trigger snippets from outside the popup
- [`docs/backup.md`](./backup.md) — how to back up / share your customised prompt library
