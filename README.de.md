<div align="right">

[🇬🇧 English](./README.md) · **🇩🇪 Deutsch**

</div>

<div align="center">
  <img src="docs/inspector-rust.png?v=3" alt="Built with Rust" width="600" />

  # Inspector Rust 🕵️‍♂️

  **Das Keyboard-first Clipboard-Hyper-Toolkit für Leute, die `Cmd+V` für ein Verb, ein Substantiv und eine Lebenseinstellung halten — Windows 11 & macOS, native, kein Electron, keine Cloud, kein Bullshit.**

  Das ist der Clipboard-Manager, der seine eigene Spezifikation aufgefressen hat.

  Drück **`Ctrl+Shift+V`** irgendwo auf deinem System → ein transparentes, rahmenloses Popup geistert über dem Monitor herein, auf dem dein Cursor wohnt, fokussiert automatisch ein Suchfeld, fuzzy-sucht durch **1 000 deduplizierte Clipboard-Einträge** nach Inhalt (Text, RTF, HTML-Preview, PNG-Bitmaps als Base64 in SQLite, Datei-Pfad-Listen), **AES-256-GCM-verschlüsselt at-rest** mit dem Schlüssel sicher im **OS-Keychain** (macOS Keychain / Windows Credential Manager / Linux Secret Service, mit 0600-Keyfile-Fallback für die Paranoiden). Drück **Enter** — der Eintrag wird zurück auf die Zwischenablage geschrieben, das Popup versteckt sich, wartet bis der Fokus sich gesetzt hat, und synthetisiert **`Cmd/Ctrl+V`** in genau die App, in der du gerade warst. Der ganze Loop dauert unter 200 ms (kalt) und braucht unter 50 MB RAM.

  Aber das ist nur der *erste* Tab. Es gibt vier.

  Das **Suchfeld funktioniert gleichzeitig als inline-Taschenrechner** (`2+2`, `sqrt(144)`, `sin(pi/2)`, Hex-Literale, Bit-Shifts — alles, was `mathjs` parsen kann), als **Farb-Konverter** (paste `#0078d4` oder `rgb(0,120,212)` und kriege jedes andere Format mit einem Klick), als **Snippet-Matcher** (tippe `aiplan` → der *aiplan*-AI-Prompt-Body schiebt sich an die Spitze der Liste, Enter pasted ihn), und als **Color Picker**-Button, der `NSColorSampler` auf macOS oder ein GDI-Screen-Overlay auf Windows fürs system-weite Pipettieren feuert. **Image-Einträge** kriegen ein Preview-Panel mit **Recolor** (9-Swatch-Logo-Einfärbung via Per-Pixel-Luminanz-Lerp), **Cut out background** (ein 4,5 MB großes U²-Net-ONNX-Modell, statisch ins Binary gelinkt via `ort`-Crate — echte Foto-Segmentierung, kein Chroma-Key, vergleichbar mit Pythons `rembg`, nur ohne Python), und **Save to Downloads** (`⌘S` schreibt das PNG auf die Platte, perfekt für das frisch eingefärbte Logo, das du gerade produziert hast).

  **`Ctrl+Shift+O`** feuert das **Bildschirm-Region-OCR**: zieh eine Marquee (`screencapture -i` — dasselbe Overlay wie `Cmd+Shift+4`), und Apples **Vision**-Framework läuft `VNRecognizeTextRequest` über die Auswahl mit `recognitionLevel = Accurate` und Sprach-Korrektur. Der erkannte Text landet auf deiner Zwischenablage, oben in der History (*gefixt in v0.14.2*), und das Source-PNG wird einen Slot darunter aufbewahrt, damit du eine andere Region nochmal OCR'en kannst ohne den Screenshot neu zu machen. Latein, CJK, Arabisch, Kyrillisch — was auch immer dein macOS-Vision-Install unterstützt. **`Ctrl+Shift+S`** (neu in **v0.15.0**) macht dieselbe Marquee, **lässt den OCR-Schritt aber weg** — reiner Region-Screenshot, PNG direkt auf Clipboard + History, sodass Diagramme, Buttons, Fotos und stille UI-Mockups da landen, wo du sie brauchst. Selbes TCC-Gate, selbes Threading-Modell, null Text erforderlich.

  Der **Text-Expander** hat *drei* Expansions-Modi nebeneinander. Der **suchbasierte** (immer an, null Permissions): tippe `mfg` ins Popup → matching Snippets blubbern nach oben → Enter pasted. Der **Abbreviation-Hotkey** (Default `Alt+1`, opt-in via Settings, beliebig konfigurierbar): tippe die Abbreviation in *irgendein* Textfeld, drück den Hotkey, Inspector Rust ersetzt sie in-place via macOS Accessibility API oder Windows UIA (mit AX-select-then-paste-Fallback für Electron / Chromium / Mac-Catalyst-Apps, die `AXValue` nur read-only freigeben — WhatsApp, Slack, Discord, VS Code — und einem Clipboard+Keystroke-Last-Resort für alles andere; der *Diagnose*-Button in Settings sagt dir, welcher Pfad benutzt wurde). Und die **Direct hotkey → snippet slots** (hinzugefügt in v0.13.0): binde einen Hotkey direkt an ein Snippet — `Alt+2` → der *aiplan*-Body — und Drücken pasted den Body **ohne dass eine Abbreviation getippt wurde**. Liest nichts, daher funktioniert es **in jeder App inklusive Terminals** (iTerm2, Terminal.app, kitty, Alacritty), wo der Abbreviation-Expander die Input-Zeile nicht sehen kann.

  Ausgeliefert mit **25 gebündelten AI-Prompt-Snippets** mit Prefix `ai*` über Programmierung, Web, IT-Security, Business, Daten und API-Design (`aiplan`, `aireview`, `airefactor`, `airegex`, `aisql`, `aitest`, `aimigration`, `aithumb`, `aithreat`, `aipentest`, `aibrief`, `aiml`, `aiapi`, `aiux`, `aimarketing`, …), jeder eine strukturierte Anweisungs-Hälfte, die du an deinen eigenen Prompt / Code / Kontext anhängst. Idempotentes Seeding (gelöschte Prompts bleiben gelöscht), Ein-Klick-*Restore defaults*, und live editierbar im **Snippets-Tab**. Der **Notes-Tab** macht aus jedem Clipboard-Eintrag ein permanentes, kategorisiertes Lesezeichen ohne History-Cap. Der **Backup-Tab** exportiert die gesamte Datenbank (History + Snippets + Notes) in eine versionierte JSON-Datei, die du auf einem anderen Rechner re-importieren kannst; Per-Sektion-Checkboxes lassen dich Snippets-only mit einem Kollegen teilen, ohne deine History zu leaken. **Backup ist by-design Plaintext-JSON**, damit es portabel zwischen Maschinen ist — die Re-Verschlüsselung findet auf dem Ziel-Keychain beim Import statt.

  Das Ganze läuft als **Menüleisten- / Tray-residenter Background-Prozess** (kein Dock-Icon auf macOS via `Accessory`-Activation-Policy, kein Taskbar-Icon auf Windows via `skipTaskbar`). **`Autostart on login`** (hinzugefügt in v0.14.0) ist ein Tray-sichtbares Check-Menü-Item + Settings-Toggle — macOS schreibt `~/Library/LaunchAgents/InspectorRust.plist`, Windows nutzt den Run-Key-Registry-Eintrag. Das Popup ist **per-Monitor aware** (öffnet auf dem Monitor mit dem Cursor, nicht immer dem Primären), **focus-loss-cancellable** (Klick außerhalb oder Esc zum Schließen, mit `suppress_hide`-Flag für native File-Dialogs, damit die nicht das Popup wegbouncen), und **fuzzy-search-as-you-type** via `fuse.js` mit virtualisierter Liste (`@tanstack/react-virtual`), die auch bei 1 000 Einträgen snappy bleibt.

  **Null Telemetrie. Null Network-Calls. Null Account.** Deine Daten liegen unter `~/Library/Application Support/InspectorRust/history.db` (macOS) oder `%APPDATA%\InspectorRust\history.db` (Windows) und nirgendwo sonst. Das 4,5 MB ONNX-Modell ist *gebündelt* — selbst Cutouts laufen offline. Die Vision-OCR ist *lokal* — Apples On-Device-ML, kein API-Key, kein Rate-Limit. Die Encryption-Keys verlassen nie deine Maschine, die Snippets synchronisieren nirgendwohin, die History gehört dir.

  Gebaut mit **Tauri 2** (WebView2 / WKWebView), **Rust** (Workspace: `core/rust-lib` ist die einzige geteilte Library, `win/src-tauri` + `macos/src-tauri` sind 2-Zeilen-Bundle-Shells), **React 19** + **TypeScript 5** + **Tailwind v4** + **Vite 7**, gepackt als **~5 MB MSI** (Windows) oder **~5 MB DMG** (macOS Apple Silicon). **110 Rust-Unit-Tests + 86 Frontend-Vitest-Tests** halten es ehrlich. **MIT-lizenziert**, hackbar, und kompromisslos gebaut für die Art Mensch, die schon Muskelgedächtnis für drei verschiedene Clipboard-Manager hat und von allen genervt ist.

  <!-- ── Status / release ─────────────────────────────────────── -->
  [![Version](https://img.shields.io/badge/version-0.17.0-blue?style=flat-square)](https://github.com/pepperonas/inspector-rust/releases)
  [![License: MIT](https://img.shields.io/badge/license-MIT-green?style=flat-square)](./LICENSE)
  [![CI](https://img.shields.io/github/actions/workflow/status/pepperonas/inspector-rust/ci.yml?branch=main&style=flat-square&label=CI)](https://github.com/pepperonas/inspector-rust/actions/workflows/ci.yml)
  [![Release](https://img.shields.io/github/actions/workflow/status/pepperonas/inspector-rust/release.yml?branch=main&style=flat-square&label=release)](https://github.com/pepperonas/inspector-rust/actions/workflows/release.yml)
  [![Latest Release](https://img.shields.io/github/v/release/pepperonas/inspector-rust?style=flat-square&label=download)](https://github.com/pepperonas/inspector-rust/releases/latest)
  [![Maintenance](https://img.shields.io/badge/maintained-yes-brightgreen?style=flat-square)](https://github.com/pepperonas/inspector-rust/commits/main)
  [![PRs welcome](https://img.shields.io/badge/PRs-welcome-brightgreen?style=flat-square)](./CONTRIBUTING.md)

  <!-- ── Platforms ────────────────────────────────────────────── -->
  [![Windows 11](https://img.shields.io/badge/Windows-11-0078D4?style=flat-square&logo=windows11&logoColor=white)](./win)
  [![macOS](https://img.shields.io/badge/macOS-10.15+-000000?style=flat-square&logo=apple&logoColor=white)](./macos)
  [![Apple Silicon](https://img.shields.io/badge/arm64-Apple%20Silicon-555555?style=flat-square&logo=apple&logoColor=white)](./macos)
  [![x86_64](https://img.shields.io/badge/x86__64-supported-555555?style=flat-square)](#)
  [![Linux](https://img.shields.io/badge/Linux-planned-orange?style=flat-square&logo=linux&logoColor=white)](#)

  <!-- ── Stack ────────────────────────────────────────────────── -->
  [![Tauri 2](https://img.shields.io/badge/Tauri-2-FFC131?style=flat-square&logo=tauri&logoColor=white)](https://tauri.app)
  [![Rust](https://img.shields.io/badge/Rust-stable-CE422B?style=flat-square&logo=rust&logoColor=white)](https://rustup.rs)
  [![React 19](https://img.shields.io/badge/React-19-61DAFB?style=flat-square&logo=react&logoColor=black)](https://react.dev)
  [![TypeScript 5](https://img.shields.io/badge/TypeScript-5-3178C6?style=flat-square&logo=typescript&logoColor=white)](https://www.typescriptlang.org)
  [![Vite 7](https://img.shields.io/badge/Vite-7-646CFF?style=flat-square&logo=vite&logoColor=white)](https://vitejs.dev)
  [![Tailwind CSS v4](https://img.shields.io/badge/Tailwind-v4-38BDF8?style=flat-square&logo=tailwindcss&logoColor=white)](https://tailwindcss.com)
  [![pnpm](https://img.shields.io/badge/pnpm-10-F69220?style=flat-square&logo=pnpm&logoColor=white)](https://pnpm.io)
  [![Node 20](https://img.shields.io/badge/Node-20+-339933?style=flat-square&logo=node.js&logoColor=white)](https://nodejs.org)
  [![SQLite](https://img.shields.io/badge/SQLite-bundled-003B57?style=flat-square&logo=sqlite&logoColor=white)](https://sqlite.org)
  [![ONNX Runtime](https://img.shields.io/badge/ONNX%20Runtime-bundled-005CED?style=flat-square&logo=onnx&logoColor=white)](https://onnxruntime.ai)
  [![Apple Vision](https://img.shields.io/badge/Apple%20Vision-Live%20Text-000000?style=flat-square&logo=apple&logoColor=white)](#)
  [![U2Net](https://img.shields.io/badge/U%C2%B2--Net-cutout-orange?style=flat-square)](https://github.com/xuebinqin/U-2-Net)

  <!-- ── Security & ergonomics ───────────────────────────────── -->
  [![AES-256-GCM](https://img.shields.io/badge/encryption-AES--256--GCM-darkgreen?style=flat-square&logo=letsencrypt&logoColor=white)](./docs/encryption.md)
  [![Keychain-backed](https://img.shields.io/badge/keys-OS%20keychain-555555?style=flat-square)](./docs/encryption.md)
  [![Local-first](https://img.shields.io/badge/local--first-yes-brightgreen?style=flat-square)](#)
  [![No telemetry](https://img.shields.io/badge/telemetry-none-brightgreen?style=flat-square)](#)
  [![Offline](https://img.shields.io/badge/network-not%20required-brightgreen?style=flat-square)](#)
  [![Power user](https://img.shields.io/badge/audience-power%20users-blueviolet?style=flat-square)](#)
  [![Keyboard-first](https://img.shields.io/badge/UX-keyboard--first-blueviolet?style=flat-square)](#)

  <!-- ── Quality ─────────────────────────────────────────────── -->
  [![ESLint](https://img.shields.io/badge/ESLint-flat%20config-4B32C3?style=flat-square&logo=eslint&logoColor=white)](https://eslint.org)
  [![Vitest](https://img.shields.io/badge/Vitest-3-6E9F18?style=flat-square&logo=vitest&logoColor=white)](https://vitest.dev)
  [![cargo test](https://img.shields.io/badge/cargo%20test-110%20passing-success?style=flat-square&logo=rust&logoColor=white)](#)
  [![vitest](https://img.shields.io/badge/vitest-86%20passing-success?style=flat-square&logo=vitest&logoColor=white)](#)
  [![cargo clippy](https://img.shields.io/badge/cargo%20clippy-D%20warnings-success?style=flat-square&logo=rust&logoColor=white)](#)
  [![tsc strict](https://img.shields.io/badge/tsc-strict-3178C6?style=flat-square&logo=typescript&logoColor=white)](#)
  [![Prettier](https://img.shields.io/badge/code%20style-Prettier-F7B93E?style=flat-square&logo=prettier&logoColor=black)](https://prettier.io)

  <!-- ── Community ───────────────────────────────────────────── -->
  [![Issues](https://img.shields.io/github/issues/pepperonas/inspector-rust?style=flat-square)](https://github.com/pepperonas/inspector-rust/issues)
  [![Closed issues](https://img.shields.io/github/issues-closed/pepperonas/inspector-rust?style=flat-square&color=success)](https://github.com/pepperonas/inspector-rust/issues?q=is%3Aissue+is%3Aclosed)
  [![PRs](https://img.shields.io/github/issues-pr/pepperonas/inspector-rust?style=flat-square)](https://github.com/pepperonas/inspector-rust/pulls)
  [![Stars](https://img.shields.io/github/stars/pepperonas/inspector-rust?style=flat-square)](https://github.com/pepperonas/inspector-rust/stargazers)
  [![Forks](https://img.shields.io/github/forks/pepperonas/inspector-rust?style=flat-square)](https://github.com/pepperonas/inspector-rust/network/members)
  [![Watchers](https://img.shields.io/github/watchers/pepperonas/inspector-rust?style=flat-square)](https://github.com/pepperonas/inspector-rust/watchers)
  [![Contributors](https://img.shields.io/github/contributors/pepperonas/inspector-rust?style=flat-square)](https://github.com/pepperonas/inspector-rust/graphs/contributors)
  [![Last commit](https://img.shields.io/github/last-commit/pepperonas/inspector-rust?style=flat-square)](https://github.com/pepperonas/inspector-rust/commits/main)
  [![Commit activity](https://img.shields.io/github/commit-activity/m/pepperonas/inspector-rust?style=flat-square)](https://github.com/pepperonas/inspector-rust/commits/main)
  [![Repo size](https://img.shields.io/github/repo-size/pepperonas/inspector-rust?style=flat-square)](https://github.com/pepperonas/inspector-rust)
  [![Code size](https://img.shields.io/github/languages/code-size/pepperonas/inspector-rust?style=flat-square)](https://github.com/pepperonas/inspector-rust)
  [![Top language](https://img.shields.io/github/languages/top/pepperonas/inspector-rust?style=flat-square)](https://github.com/pepperonas/inspector-rust)
  [![Languages](https://img.shields.io/github/languages/count/pepperonas/inspector-rust?style=flat-square)](https://github.com/pepperonas/inspector-rust)
  [![Lines](https://img.shields.io/tokei/lines/github/pepperonas/inspector-rust?style=flat-square&label=lines%20of%20code)](https://github.com/pepperonas/inspector-rust)
  [![Made with love](https://img.shields.io/badge/made%20with-%E2%99%A5-red?style=flat-square)](#)

  <!-- ── Architecture & build ────────────────────────────────── -->
  [![Monorepo](https://img.shields.io/badge/repo-pnpm%20workspace-F69220?style=flat-square&logo=pnpm&logoColor=white)](./pnpm-workspace.yaml)
  [![Workspace crates](https://img.shields.io/badge/cargo%20workspace-3%20crates-CE422B?style=flat-square&logo=rust&logoColor=white)](./Cargo.toml)
  [![Single binary](https://img.shields.io/badge/distribution-single%20binary-blue?style=flat-square)](#)
  [![Native](https://img.shields.io/badge/no-Electron-success?style=flat-square)](#)
  [![Memory](https://img.shields.io/badge/memory-%3C50%20MB-blue?style=flat-square)](#)
  [![Cold start](https://img.shields.io/badge/cold%20start-%3C200%20ms-blue?style=flat-square)](#)
  [![MSI size](https://img.shields.io/badge/MSI-~5%20MB-blue?style=flat-square&logo=windows&logoColor=white)](#)
  [![DMG size](https://img.shields.io/badge/DMG-~5%20MB-blue?style=flat-square&logo=apple&logoColor=white)](#)
  [![exe size](https://img.shields.io/badge/.exe-~14%20MB-blue?style=flat-square&logo=windows&logoColor=white)](#)

  <!-- ── Features (numerical) ────────────────────────────────── -->
  [![IPC commands](https://img.shields.io/badge/IPC%20commands-60-blueviolet?style=flat-square)](./core/rust-lib/src/commands.rs)
  [![Tauri events](https://img.shields.io/badge/events-10-blueviolet?style=flat-square)](#)
  [![Rust modules](https://img.shields.io/badge/Rust%20modules-22-CE422B?style=flat-square&logo=rust&logoColor=white)](./core/rust-lib/src)
  [![Snippets](https://img.shields.io/badge/AI%20prompts-25%20bundled-blueviolet?style=flat-square)](./docs/ai-prompts.md)
  [![Tabs](https://img.shields.io/badge/popup%20tabs-4-blueviolet?style=flat-square)](#)
  [![DB tables](https://img.shields.io/badge/SQLite%20tables-4-003B57?style=flat-square&logo=sqlite&logoColor=white)](./docs/encryption.md)
  [![Global shortcuts](https://img.shields.io/badge/global%20hotkeys-4-blueviolet?style=flat-square)](#)
  [![Snippet expansion modes](https://img.shields.io/badge/expansion%20modes-3-blueviolet?style=flat-square)](./docs/text-expander.md)
  [![Image formats](https://img.shields.io/badge/image%20formats-5-blueviolet?style=flat-square)](#)

  <!-- ── Standards / conventions ─────────────────────────────── -->
  [![SemVer](https://img.shields.io/badge/semver-2.0-blue?style=flat-square)](https://semver.org)
  [![Keep a Changelog](https://img.shields.io/badge/changelog-Keep%20a%20Changelog-orange?style=flat-square)](https://keepachangelog.com)
  [![Conventional Commits](https://img.shields.io/badge/commits-conventional-orange?style=flat-square)](https://www.conventionalcommits.org)
  [![ARIA](https://img.shields.io/badge/a11y-keyboard%20first-blueviolet?style=flat-square)](#)
  [![ADRs in CHANGELOG](https://img.shields.io/badge/ADRs-in%20CHANGELOG-orange?style=flat-square)](./CHANGELOG.md)

  <!-- ── Permissions / OS surfaces ───────────────────────────── -->
  [![macOS TCC: Accessibility](https://img.shields.io/badge/macOS%20TCC-Accessibility-000000?style=flat-square&logo=apple&logoColor=white)](./docs/text-expander.md)
  [![macOS TCC: Screen Recording](https://img.shields.io/badge/macOS%20TCC-Screen%20Recording-000000?style=flat-square&logo=apple&logoColor=white)](#)
  [![Autostart](https://img.shields.io/badge/autostart-LaunchAgent%20%2F%20RegRun-blue?style=flat-square)](./CHANGELOG.md)
  [![Tray icon](https://img.shields.io/badge/UI-tray%20resident-blue?style=flat-square)](#)

  <!-- ── Tech (extended) ─────────────────────────────────────── -->
  [![rusqlite](https://img.shields.io/badge/rusqlite-bundled-CE422B?style=flat-square&logo=rust&logoColor=white)](https://crates.io/crates/rusqlite)
  [![enigo](https://img.shields.io/badge/enigo-paste%20sim-CE422B?style=flat-square&logo=rust&logoColor=white)](https://crates.io/crates/enigo)
  [![clipboard-rs](https://img.shields.io/badge/clipboard--rs-event%20driven-CE422B?style=flat-square&logo=rust&logoColor=white)](https://crates.io/crates/clipboard-rs)
  [![ort](https://img.shields.io/badge/ort-ONNX%20Runtime-CE422B?style=flat-square&logo=rust&logoColor=white)](https://crates.io/crates/ort)
  [![ring](https://img.shields.io/badge/ring-AES--256--GCM-CE422B?style=flat-square&logo=rust&logoColor=white)](https://crates.io/crates/ring)
  [![objc2](https://img.shields.io/badge/objc2-Vision%20FFI-CE422B?style=flat-square&logo=rust&logoColor=white)](https://crates.io/crates/objc2)
  [![Fuse.js](https://img.shields.io/badge/fuse.js-fuzzy%20search-005571?style=flat-square)](https://www.fusejs.io)
  [![lucide-react](https://img.shields.io/badge/icons-lucide--react-F56565?style=flat-square)](https://lucide.dev)
  [![react-virtual](https://img.shields.io/badge/list-react--virtual-FF4154?style=flat-square&logo=react&logoColor=white)](https://tanstack.com/virtual)

  <!-- ── Vibes ───────────────────────────────────────────────── -->
  [![Inspired by Alfred](https://img.shields.io/badge/inspired%20by-Alfred-blueviolet?style=flat-square)](#)
  [![Mouse-free](https://img.shields.io/badge/mouse-not%20required-brightgreen?style=flat-square)](#)
  [![Self-hosted](https://img.shields.io/badge/data-on%20your%20disk-brightgreen?style=flat-square)](#)
  [![Free forever](https://img.shields.io/badge/free-forever-brightgreen?style=flat-square)](./LICENSE)
  [![Made in Germany](https://img.shields.io/badge/made%20in-Germany-FFCE00?style=flat-square)](#)

  Drück `Ctrl+Shift+V` → suchen → einfügen. Inspiriert von Alfreds Clipboard-Viewer auf macOS, eingedampft auf ein Tool, das du auf jeder Maschine behalten kannst.
</div>

---

## Download

**Aktueller Release:** [![Latest Release](https://img.shields.io/github/v/release/pepperonas/inspector-rust?style=flat-square&label=latest&color=green)](https://github.com/pepperonas/inspector-rust/releases/latest) — siehe [CHANGELOG](./CHANGELOG.md) für die Neuerungen.

| Plattform | Datei | Hinweise |
|-----------|-------|----------|
| **Windows 11 / 10** | [`InspectorRust_<ver>_x64_en-US.msi`](https://github.com/pepperonas/inspector-rust/releases/latest) | MSI-Installer — fügt Startmenü-Eintrag & Uninstaller hinzu |
| **Windows 11 / 10** | [`inspector-rust.exe`](https://github.com/pepperonas/inspector-rust/releases/latest) | Standalone-Exe — keine Installation nötig |
| **macOS 10.15+ (Apple Silicon)** | [`InspectorRust_<ver>_aarch64.dmg`](https://github.com/pepperonas/inspector-rust/releases/latest) | DMG für arm64-Macs |
| **macOS Intel** | — | Aus Source bauen: [`macos/README.md`](./macos/README.md) |
| **Linux** | — | Geplant für einen späteren Release |

> **macOS-Gatekeeper-Hinweis.** Local-Build-Releases sind **nicht Apple-signiert**. Beim ersten Start weigert sich macOS, die App zu öffnen — Rechtsklick → **Öffnen** → bestätigen, oder **Systemeinstellungen → Datenschutz & Sicherheit → "Trotzdem öffnen"**. Dann **zwei** TCC-Permissions erteilen:
> - **Bedienungshilfen** — nötig für Paste (`enigo` synthetisiert Cmd+V) und den system-weiten Text-Expander (Cmd+Shift+← / Cmd+C / Cmd+V-Zyklus).
> - **Bildschirmaufnahme** — nötig für den OCR-Shortcut (`screencapture -i` wird Inspector Rust zugeordnet und macOS verweigert es ohne diese Berechtigung). Der Settings-Tab zeigt beide als zusammenklappbare amber Banner mit Ein-Klick-Sprung zum richtigen Privacy-Pane.
>
> Komplettes Setup in [`macos/README.md`](./macos/README.md).

---

## Plattform-Support

| Plattform  | Status              | Verzeichnis           |
|------------|---------------------|-----------------------|
| Windows 11 | ✅ implementiert     | [`win/`](./win)         |
| macOS      | ✅ implementiert     | [`macos/`](./macos)     |
| Linux      | 🟡 geplant          | `linux/` (noch nicht) |

Die gesamte App-Logik lebt in [`core/`](./core) — ein einzelnes Frontend (`core/frontend`) und eine einzelne Rust-Library (`core/rust-lib`), die plattformübergreifend geteilt werden. Jedes OS hat seine eigene dünne Bundle-Shell mit plattformspezifischen Details (Installer-Config, Icons, Capabilities). Um eine neue Plattform hinzuzufügen, siehe [`CONTRIBUTING.md`](./CONTRIBUTING.md#adding-a-new-platform-shell-linux-etc).

## Workflow

Inspector Rust ist für *einen* Workflow gebaut: **`Ctrl+Shift+V` → tippen → Enter**. Der Hotkey öffnet ein rahmenloses Popup über dem aktiven Monitor; was du tippst, wird fuzzy durchsucht über Clipboard-History, Snippets, Calc-Ergebnisse und Farbwerte; Enter fügt den Top-Match in die zuvor fokussierte App ein. Keine Maus, keine Menü-Bäume, keine Per-App-Integrationen.

Drei weitere globale Shortcuts feuern von überall — Inspector Rusts Fenster muss nicht offen oder fokussiert sein:

- **`Ctrl+Shift+O`** — Bildschirm-Region-**OCR**. Marquee ziehen, Apple Vision erkennt den Text in der Region, der Text landet auf deiner Zwischenablage + oben in der History.
- **`Ctrl+Shift+S`** *(v0.15.0+)* — Bildschirm-Region-**Screenshot**. Gleiche Marquee, aber kein OCR-Schritt: das aufgenommene PNG geht direkt auf die Zwischenablage und in die History. Nimm das, wenn die Region keinen Text hat (Diagramm, Button, Foto, UI-Mockup) oder du das Bild selbst willst.
- **`Ctrl+Shift+C`** *(v0.17.0+)* — **Eyedropper**. Cursor wird zur NSColorSampler-Lupe (macOS) / GDI-Overlay (Windows); ein Klick auf ein Pixel und der Hex-Code (`#RRGGBB`) landet auf der Zwischenablage + History. Kein Popup, kein Modal — fire-and-forget.

Literal Control auf jedem OS — dieselbe Taste auf Windows und macOS. OCR + Screenshot benötigen das macOS-**Bildschirmaufnahme**-TCC-Grant; der Eyedropper nicht.

Alles andere (Snippets-Verwaltung, Notes, Settings, Image-Tools) lebt im selben Popup hinter Tabs oben rechts — es gibt kein separates Fenster zum Alt-Tabben. **Settings → Keyboard shortcuts** trägt das komplette Cheat-Sheet.

## Features & Shortcuts auf einen Blick

<div align="center">
  <img src="docs/ir-ff-w1024-optimized.png?v=1" alt="Inspector Rust — Keyboard-first Clipboard-Toolkit" width="600" />
</div>

### 🔥🔥🔥 Globale Hotkeys — fire and forget, von überall 🔥🔥🔥

| Shortcut | Aktion | Benötigt (macOS) |
|----------|--------|------------------|
| `Ctrl+Shift+V` | Popup auf dem aktiven Monitor öffnen | — |
| `Ctrl+Shift+O` | Bildschirm-Region-**OCR** → Text auf Clipboard + History | Bildschirmaufnahme |
| `Ctrl+Shift+S` *(v0.15.0+)* | Bildschirm-Region-**Screenshot** → PNG auf Clipboard + History (kein OCR) | Bildschirmaufnahme |
| `Ctrl+Shift+C` *(v0.17.0+)* | **Eyedropper** → Hex (`#RRGGBB`) auf Clipboard + History | — |
| `Alt+1` *(Default, konfigurierbar, opt-in)* | Snippet-Abbreviation in-place expandieren | Bedienungshilfen |
| *(user-konfigurierbar)* | **Direct hotkey → snippet** — bestimmten Snippet-Body pasten | Bedienungshilfen |

Literal Control auf jedem OS. Dieselbe Taste auf Windows und macOS. Der Expander-Hotkey ist opt-in (aus, bis du ihn in Settings → Text expander konfigurierst).

### Popup-Shortcuts — wenn das Popup offen ist

| Shortcut | Aktion |
|----------|--------|
| `↑` `↓` | In der Liste navigieren |
| `Enter` | Ausgewählten Eintrag pasten (respektiert das Plain-Text-Setting) |
| `Shift+Enter` | Mit Originalformatierung pasten (überschreibt das Plain-Text-Setting einmalig) |
| `Esc` | Popup schließen |
| `⌘B` / `Ctrl+B` | **Hintergrund freistellen** beim ausgewählten Image-Eintrag (ML — U²-Net) |
| `⌘S` / `Ctrl+S` | **Bild in Downloads speichern** (PNG unverändert) |

### Komplette Feature-Matrix

| Feature | Wo triggern | Doku |
|---------|-------------|------|
| Clipboard-History (Text/RTF/HTML/PNG/Files, 1 000 Einträge, dedupliziert) | `Ctrl+Shift+V` → suchen | core |
| Fuzzy-Suche (`fuse.js`, Threshold 0.4) | Im Suchfeld tippen | core |
| **Inline-Taschenrechner** | Ausdruck im Suchfeld tippen (`2+2`, `sqrt(9)`, `sin(pi/2)`, `0xff << 4`, …) | core |
| **Farb-Konverter** | `#RRGGBB` / `rgb(…)` / `hsl(…)` im Suchfeld tippen → Swatch + alle Formate | [colors.md](./docs/colors.md) |
| **HSV-Color-Picker-Modal** | History-Tab → *Color Picker*-Button → Hue-Slider + Swatch + Hex/RGB/HSL-Tabs | [colors.md](./docs/colors.md) |
| **Screen-Eyedropper** (Modal) | *Color Picker*-Modal → *Pick from screen* (macOS `NSColorSampler`-Lupe / Windows GDI-Overlay) | [colors.md](./docs/colors.md) |
| **Eyedropper — globaler Hotkey** *(v0.17.0+)* | `Ctrl+Shift+C` oder Tray *Pick Color* → Hex direkt aufs Clipboard, kein Popup | [colors.md](./docs/colors.md) |
| Snippet-Search-as-you-type | Snippet-Abbreviation im Popup-Suchfeld tippen | [text-expander.md](./docs/text-expander.md) |
| Abbreviation-Expander (system-weit) | Abbreviation in irgendein Textfeld tippen → `Alt+1` (Default) | [text-expander.md](./docs/text-expander.md) |
| Direct hotkey → snippet *(v0.13.0+)* | User-bound globaler Hotkey | [text-expander.md](./docs/text-expander.md) |
| 25 gebündelte AI-Prompt-Snippets (`ai*`) | Snippets-Tab; Search / Abbreviation / Direct-Slot | [ai-prompts.md](./docs/ai-prompts.md) |
| Snippets CRUD + JSON-Import | Snippets-Tab → Formular / Import-Button | [snippets-import.md](./docs/snippets-import.md) |
| Notes — kategorisierte persistente Bookmarks | Notes-Tab (Tray: *Manage Notes*) | [notes.md](./docs/notes.md) |
| Clip als Note speichern | Hover über History-Zeile → Bookmark-Icon | [notes.md](./docs/notes.md) |
| **Bildschirm-Region-OCR** *(macOS, v0.9.0+)* | `Ctrl+Shift+O` oder Tray *OCR Region* | core |
| **Bildschirm-Region-Screenshot** *(macOS, v0.15.0+)* | `Ctrl+Shift+S` oder Tray *Screenshot Region* | core |
| **Bild-Recolor** (Logo-Tinten, Chromaticity-gated) | Preview-Pane bei Image-Eintrag → Swatch / Hex | core |
| **ML-Hintergrund-Cutout** (U²-Net-ONNX, ~4,5 MB embedded) | Preview-Pane → *Cut out background* oder `⌘B` | core |
| Bild in Downloads speichern | Preview-Pane oder `⌘S` (PNG unverändert) | core |
| Backup — Single-File-JSON-Export/Import (History + Snippets + Notes, per-Sektion ankreuzbar) | Settings → Backup & restore | [backup.md](./docs/backup.md) |
| Plain-Text-only Paste *(Default an, v0.4.0+)* | Settings → Paste (Shift+Enter überschreibt einmal) | core |
| Autostart bei Login *(v0.14.0+)* | Settings → Startup *oder* Tray-Checkmark | core |
| Clipboard-Capture pausieren | Tray → *Pause Capture* | core |
| History löschen (mit Bestätigung) | Tray → *Clear History…* | core |
| **AES-256-GCM at-rest** (alle Bodies) *(v0.6.0+)* | Automatisch; Key im OS-Keychain | [encryption.md](./docs/encryption.md) |
| Per-Monitor-Popup-Placement | Automatisch (öffnet auf Monitor mit Cursor) | core |
| Multi-Tab-UI | Popup oben-rechts Tabs: History · Snippets · Notes · Settings | core |
| Permissions-UX (TCC-Banner + 1-s-Polling + `tccutil reset`-Recovery) | Settings → Permissions-Sektion *(macOS)* | core |
| Keyboard-Shortcuts-Cheat-Sheet | Settings → *Keyboard shortcuts* (OS-adaptive Glyphen) | core |
| About-Dialog | Settings → About | core |

## Features

### Clipboard-Core
- **Globaler Hotkey** `Ctrl+Shift+V` öffnet das Popup zentriert auf dem Monitor mit dem Cursor.
- **Erfasst** Text, RTF, HTML, Bilder (PNG, ≤ 5 MB), und Datei-Listen via OS-nativen Clipboard-Events (kein Polling). Image-vor-Files-Priorität auf macOS, sodass Finder-Image-Copies als Bitmaps landen, nicht als Pfade.
- **Fuzzy-Suche** (`fuse.js`, Threshold 0.4) rankt Matches während du tippst. Virtualisierte Liste, per-Content-Type Preview-Panel.
- **Auto-Paste** — Enter pasted via `enigo`-simuliertem `Ctrl+V` / `Cmd+V` in die zuvor fokussierte App. Shift+Enter überschreibt das Plain-Text-Setting und pasted mit Originalformatierung.
- **SQLite-Store** unter `%APPDATA%\InspectorRust\history.db` / `~/Library/Application Support/InspectorRust/history.db`. SHA-256-dedupliziert, Cap bei 1 000 Einträgen.
- **AES-256-GCM at-rest** seit v0.6.0 — Text-/HTML-/RTF-/Image-Bodies, Snippet-Bodies, Note-Bodies. Schlüssel im OS-Keychain (Keychain / Credential Manager / Secret Service), 0600-Keyfile-Fallback. Volle Referenz: [`docs/encryption.md`](./docs/encryption.md).
- **Time-Chip** (v0.10.3) — der relative Time-Hint auf jeder Zeile (`just now`, `1h ago`) wird zu einem winzigen klickbaren Button: Hover zeigt sowohl `Captured` als auch `Last used` als absolute Timestamps in einem Tooltip; Klick schaltet den Chip selbst zwischen relativer und absoluter Anzeige um.

### Text-Expander (Snippets, v0.2 — system-weit v0.2.7, Hotkey-Überholung v0.12.0, Direct Slots v0.13.0)
- **Expansion im Popup** — tippe eine Abbreviation ins Suchfeld; matching Snippets erscheinen über Clipboard-Einträgen; Enter pasted den Body.
- **Abbreviation-Expander** — tippe die Abbreviation in *irgendein* Textfeld, drücke den konfigurierten Hotkey (Default `Alt+1`, opt-in via Settings; Ein-Klick-Presets `Alt+1` / `Alt+2` / `Alt+3`, oder beliebige Kombination aufnehmen), Inspector Rust ersetzt sie in-place. Drei Pfade: AX/UIA in-place-Ersatz (native Apps — keine Clipboard-Berührung, kein Flicker, verifiziert durch erneutes Lesen des Werts); AX-select-then-paste-over-selection für Electron / Chromium / Mac-Catalyst-Apps, die `AXValue` read-only freigeben (WhatsApp, Slack, Discord, VS Code — v0.12.0); und ein Clipboard+Keystroke-Fallback für alles andere. Der Diagnose-Button in Settings sagt, welcher Pfad benutzt wurde.
  - *Warum `Alt+1` und nicht `Alt+Backquote`?* Der alte Default war auf deutschen ISO-MacBooks unerreichbar (die physische `^`-Taste meldet sich als `IntlBackslash`). Ziffernreihe-Tasten sind layout-stabil überall. Ein un-customised alter Install wird einmal beim Upgrade auf `Alt+1` migriert (überschreibt keinen Wert, den du absichtlich neu gewählt hast).
- **Direct hotkey → snippet slots (v0.13.0)** — binde einen Hotkey direkt an ein Snippet (Settings → *Direct hotkey → snippet*); Drücken pasted den Body am Cursor mit **keiner getippten Abbreviation**. Liest nichts vom fokussierten Feld — schreibt nur den Body auf die Zwischenablage, synthetisiert Paste, stellt die Zwischenablage wieder her — funktioniert daher in **jeder** App, **inklusive Terminals** (iTerm2, Terminal.app, …), wo der Abbreviation-Expander die Input-Zeile nicht sehen kann. Kollisionen mit Popup-/OCR-/Abbreviation-Hotkeys werden abgelehnt.
- **Laut bei Permission-Fail (macOS, v0.12.0)** — wenn Accessibility nicht erteilt ist, no-opt der Hotkey nicht länger still: Inspector Rust öffnet sein Popup, wechselt auf Settings und zeigt ein amber Banner mit `Force re-grant` → `Restart now`. (Selbes Pattern wie OCR-/Paste-Banner. Direct Slots nutzen dasselbe Gate + Banner.)
- **Snippets-Tab** zum Erstellen/Editieren/Löschen mit zweispaltigem Formular. **JSON-Import** via Snippets → Import (`docs/snippets-import.md`, thematische Samples in `docs/examples/snippets/`).
- Caveat: der **Abbreviation**-Expander kann auf einer Terminal-Befehlszeile nicht funktionieren (keine AX-exponierte Input-Zeile, kein GUI-"vorheriges Wort markieren" auf einem Shell-Prompt — benutze in Terminals einen *Direct hotkey → snippet*-Slot oder das Popup). Image-/Files-Snippets werden nicht expandiert (nur Text).
- Volle Referenz: [`docs/text-expander.md`](./docs/text-expander.md).

### 25 gebündelte AI-Prompt-Snippets (v0.5.0, überarbeitet v0.12.0)
First-Launch seedet deine Snippet-Tabelle mit `ai*`-prefixed Prompts über Programmierung, Web, IT-Security, Business, Daten und API-Design (`aiplan`, `aireview`, `airefactor`, `airegex`, `aisql`, `aitest`, `aimigration`, `aithumb`, `aithreat`, `aipentest`, `aibrief`, `aiml`, `aiapi`, …). Jeder Prompt ist die **strukturierte Anweisungs-Hälfte only** — keine `[REQUIREMENT]`-artigen Fill-in-Slots (entfernt in v0.12.0). Du hängst ihn an deinen eigenen Prompt / Code / Kontext an und das LLM nimmt das Thema von dort auf. Idempotent (gelöschte Prompts bleiben gelöscht), wiederherstellbar von der Snippets-Sidebar — existierende Installs klicken *Restore defaults*, um den v0.12.0-Stil aufzugreifen. Komplette Liste: [`docs/ai-prompts.md`](./docs/ai-prompts.md).

### Inline-Taschenrechner (v0.2.5)
Tippe einen Mathe-Ausdruck ins Suchfeld, das Ergebnis erscheint als oberster Listen-Eintrag — Alfred-Style. Enter zum Pasten.

- Operatoren `+ - * / % ^`, unär `+/-`, Klammern. Zahlen: int/dezimal/wissenschaftlich/`1_000`-gruppiert. Konstanten: `pi`/`π`, `tau`, `e`. Funktionen: `sqrt`, `cbrt`, `abs`, `sign`, `floor`/`ceil`/`round`, `ln`/`log`/`log2`, `exp`, Trig + Hyperbolisch + Invers, `min`/`max`/`pow`/`mod`.
- Aktiviert nur bei Ausdrücken mit mindestens einem Operator/Function/Konstante — pure Zahlen und Text triggern nicht. Force-Evaluation einer Literale mit `=`-Prefix (`=pi`).
- Sicherer Recursive-Descent-Parser in [`calc.ts`](./core/frontend/src/lib/calc.ts), kein `eval`. 27 Tests.

### Farb-Tools (v0.4.0 → v0.5.2)
- **Inline-Hex-Preview** — tippe `#3366FF` (auch `3366ff`, `#abc`, `#abcdef12`) → Swatch + Hex + RGB-Zeile oben → Enter pasted Großbuchstaben `#RRGGBB`.
- **HSV-Picker-Modal** — Hue-Slider, großes Swatch, Output-Tabs für Hex / RGB / HSL, Zwei-Klick-Auswahl (kein stiller Default), Copy via Tauri-Clipboard-Plugin (umgeht WKWebView-Restriktionen).
- **Pixel vom Bildschirm picken** — sample irgendein Pixel auf dem Desktop. macOS: Apples `NSColorSampler`-Lupe. Windows: Fullscreen-Overlay + `GetPixel`. Modul: [`screen_picker.rs`](./core/rust-lib/src/screen_picker.rs).
- Frontend in [`colors.ts`](./core/frontend/src/lib/colors.ts) + [`ColorPickerModal.tsx`](./core/frontend/src/components/ColorPickerModal.tsx). 32 Tests. Referenz: [`docs/colors.md`](./docs/colors.md).

### Bildschirm-Region-OCR (v0.9.0, macOS)
Drück `Ctrl+Shift+O` (oder nutze den Tray-Eintrag **OCR Region**) → Marquee über jeden Text auf dem Bildschirm ziehen → Inspector Rust läuft Apple Vision über die Auswahl und schreibt den erkannten Text direkt auf deine Zwischenablage. Der Text landet oben in der History; das Source-PNG wird als separater Image-Eintrag direkt darunter aufbewahrt, sodass du eine andere Region nochmal OCR'en kannst ohne den Screenshot neu zu machen, und Enter auf dem auto-selected Top-Eintrag pasted den **Text**, nicht den Screenshot (Ordering gefixt in v0.14.2). Der Hotkey ist **literal Control** auch auf macOS (v0.14.1+ — frühere Builds nutzten `⌘⇧O`, was mit IDE-Bindings kollidierte).

- **Region-Picker** — nutzt `screencapture -i` (dasselbe Binary wie Cmd+Shift+4), sodass die Marquee-UX die polierte ist, die User schon kennen. Esc cancelt sauber.
- **Engine** — Visions `VNRecognizeTextRequest` mit accuracy=Accurate + Sprach-Korrektur; selbe Engine, die Apple Live Text antreibt. Kein Model-Bundling, kein Netzwerk.
- **Sprachen** — was auch immer dein macOS-Vision-Install unterstützt (Latein + CJK + Arabisch + Kyrillisch auf macOS 13+).
- **Windows** — Implementierung steht aus (wird `Windows.Media.Ocr` benutzen).
- Module: [`region_picker.rs`](./core/rust-lib/src/region_picker.rs), [`ocr.rs`](./core/rust-lib/src/ocr.rs).

### Image-Tools — Recolor + ML-Cutout + Save (v0.7.0 → v0.10.x)
Auf ausgewählten Image-Einträgen zeigt das Preview-Panel drei Aktionen:

- **Recolor** (v0.7.0) — für überwiegend graustufige PNGs (Logos / Icons / Silhouetten), 9 Preset-Swatches + Custom-Hex färben das Bild. RGB-Lerp von Target → Weiß pro Pixel-Luminanz, Alpha bleibt erhalten. Gesättigte Fotos werden automatisch aus der Toolbar versteckt (Chromaticity-Gate). Fügt die getintete Version als neuen History-Eintrag hinzu; das Original bleibt.
- **Cut out background** (v0.10.0) — lässt das **U²-Net (U2Netp) ONNX-Model** (~4,5 MB embedded) über das Bild laufen, um das Foreground-Subject zu detektieren; Output ist ein transparentes PNG, gespeichert nach `~/Downloads/<name>-cutout-<ts>.png`. Shortcut `Cmd/Ctrl+B`. Funktioniert mit echten Fotos (Flugzeug am Himmel, Person vor unruhigem Hintergrund, …) — selbe Architektur wie Pythons `rembg`, nur ohne Python. Inference läuft via `ort` (ONNX Runtime, statisch ins Binary gelinkt).
- **Save to Downloads** (v0.10.1) — drop den ausgewählten Image-Eintrag auf die Platte als `~/Downloads/inspector-rust-image-<ts>.png` unverändert. Shortcut `Cmd/Ctrl+S`. Companion zum Recolor: wähle den frisch-getinteten History-Eintrag, drück `Cmd+S`, deine Datei liegt in Downloads.
- **Inputs:** PNG, JPEG, WebP, GIF, BMP — für Clipboard-Image-Einträge *und* Single-File-Files-Einträge (eine aus Finder kopierte JPG funktioniert also auch). Output ist immer RGBA-PNG.
- Module: [`recolor.rs`](./core/rust-lib/src/recolor.rs), [`cutout_ml.rs`](./core/rust-lib/src/cutout_ml.rs). Der Legacy-Chroma-Key-Cutout in [`cutout.rs`](./core/rust-lib/src/cutout.rs) wird als Fast-Path-Option behalten, aber per Default nicht benutzt. 16-MP-Cap auf Inputs. Gebündeltes Model: [`core/rust-lib/models/u2netp.onnx`](./core/rust-lib/models/u2netp.onnx) (Apache-2.0).

### Notes (v0.2.6)
Persistente, kategorisierte Clipboard-Items in einer separaten SQLite-Tabelle — **nicht** unterworfen dem 1 000-Einträge-Pruning.

- **Bookmark aus History** — Hover über jede Zeile → Bookmark-Icon → Eintrag landet in Notes/`Uncategorized`. Entkoppelt vom Source-Clip; überlebt Pruning.
- **Notes-Tab** — drei Panes: Kategorien-Sidebar (mit Counts; virtuelle `All` / `Uncategorized`), Liste, Detail/Edit. Frei-formige Kategorien (`<datalist>`-Autocomplete). Editierbare Bodies für Text/HTML/RTF; Image-/Files-Notes sind read-only. Per-Row-Delete + Clear All mit Bestätigung.
- **+ New Note** für from-Scratch-Einträge. Tray-Shortcut: **Manage Notes** öffnet das Popup direkt hier.
- Referenz: [`docs/notes.md`](./docs/notes.md).

### Backup — Single-File-JSON-Export/Import (v0.2.6+)
Settings-Tab → *Backup & restore* → History / Snippets / Notes einzeln ankreuzen → Export in eine JSON-Datei. Import merged zurück: Snippets upsert nach Abbreviation, History upsert nach SHA-256, Notes appended. Versioniertes Schema — neuere Backups werden abgelehnt statt still zu kappen. Referenz: [`docs/backup.md`](./docs/backup.md).

### Plain-Text-Paste (Default an, v0.4.0)
HTML- / RTF-Clipboard-Einträge werden zur Paste-Zeit auf ihren Text-Preview gestrippt, sodass Copy-aus-Word / -Browser / -Mail nicht länger Styling in andere Apps leakt. Toggle in Settings → Paste. Shift+Enter im Popup überschreibt für einen Paste.

### Permissions-UX (v0.11.0)
Inspector Rust braucht **zwei** unabhängige macOS-TCC-Grants — Accessibility (Paste + Text-Expander) und Bildschirmaufnahme (OCR + Screenshot-Region). Der Settings-Tab zeigt jeden als zusammenklappbares amber Banner, das:

- Laut bleibt (Border + Warn-Icon + primärer `Open System Settings`-Button), wenn fehlend, aber per Default zu einer einzelnen Zeile kollabiert, damit die Page nicht zugemüllt ist.
- Pre-checked, bevor der relevante native Call invoked wird. OCR returnt eine `screen.permission_denied`-Sentinel statt still zu failen, wenn Bildschirmaufnahme verweigert ist; ein Tauri-Event öffnet das Popup + flippt ein In-App-Toast-Banner, das auf den richtigen Pane zeigt.
- Pollt das Grant einmal pro Sekunde, solange nicht erteilt, sodass das Badge ~1 s nach dem Toggle in den System Settings auf grün flippt — kein Panel-Reload nötig.
- Jedes Banner hat einen `tccutil reset`-Recovery-Button für den "Toggle ist an, aber der laufende Prozess sieht immer noch denied"-Stale-cdhash-State.

### Discoverability (v0.10.7)
- **Footer-Hints** — `⌃⇧O OCR` + `⌃⇧S Shot` + `⌃⇧C Color` neben dem `⏎ Paste · ↑↓ Navigate · Esc Close`-Strip gerendert, sodass User alle globalen Shortcuts jedes Mal sehen, wenn sie das Popup öffnen.
- **Settings → Keyboard shortcuts** — Drei-Gruppen-Cheat-Sheet (Global / Popup-Nav / Image-Actions), das jeden Shortcut der App abdeckt. Modifier-Glyphs (`⌘` vs `Ctrl`, `⇧` vs `Shift`, `⌥` vs `Alt`) passen sich ans laufende OS an via dem `IS_MAC`-Helper in [`core/frontend/src/lib/platform.ts`](./core/frontend/src/lib/platform.ts).
- **About-Dialog** — Settings → About öffnet ein Modal mit Version, License, Jahr, Zielgruppe und einer tabellarischen Tech-Stack-Übersicht.

### System-Tray + Multi-Monitor
- **Tray-Menü:** Open · Manage Snippets · Manage Notes · **OCR Region (Ctrl+Shift+O)** · **Screenshot Region (Ctrl+Shift+S)** *(v0.15.0+)* · **Pick Color (Ctrl+Shift+C)** *(v0.17.0+)* · Pause Capture · ☑/☐ Start with Windows / Start at Login (Checkmark spiegelt State seit v0.14.0) · Clear History · Quit.
- **Autostart bei Login** (v0.14.0) — Toggle in Settings → Startup oder vom Tray-Menü. macOS schreibt `~/Library/LaunchAgents/InspectorRust.plist`; Windows nutzt den Run-Key-Registry-Eintrag. App startet hidden im Tray, sodass sie bereit ist, wenn der Popup-Hotkey trifft.
- **Multi-Monitor-Placement:** Popup öffnet auf dem Monitor mit dem Cursor, horizontal zentriert, ~⅓ von oben, geclamped auf die Bounds des aktiven Monitors (wichtig bei Mixed-DPI-Setups).

## Repository-Layout

```
inspector-rust/
├── core/
│   ├── frontend/            # React 19 + TS + Tailwind v4 (plattformübergreifend)
│   │   └── src/
│   │       ├── components/  # SearchBar, HistoryList/Item, PreviewPanel, SnippetsPanel, NotesPanel, …
│   │       ├── hooks/       # useClipboardHistory, useFuzzySearch, useSnippets, useNotes, useKeyboardNav
│   │       └── lib/         # ipc.ts, types.ts, calc.ts (Alfred-Style-Evaluator), format.ts
│   └── rust-lib/            # Geteilte Rust-App-Logik
│       ├── build.rs         # Linkt das macOS Vision-Framework für OCR
│       ├── models/
│       │   └── u2netp.onnx  # U²-Net Cutout-Model (~4,5 MB, Apache-2.0)
│       └── src/
│           ├── lib.rs                # Tauri-Builder, Plugin-/Tray-Setup, invoke_handler
│           ├── commands.rs           # alle #[tauri::command]-Wrapper
│           ├── models.rs             # ContentType / ClipEntry / NewClip + Caps
│           ├── db.rs                 # entries-Table, Hash-Dedup, Prune
│           ├── crypto.rs             # AES-256-GCM At-Rest-Encryption + OS-Keychain-Key
│           ├── snippets.rs           # snippets-Table, JSON-Upsert, Exakt-Abbreviation-Lookup
│           ├── seed.rs               # Default-AI-Prompt-Snippets, gebündelt beim First-Launch
│           ├── notes.rs              # notes-Table, Kategorien, save_from_clip
│           ├── backup.rs             # Full-App-Export/Import (versioniertes JSON)
│           ├── settings.rs           # Key/Value-Store (Expander-Hotkey + Zukunfts-Prefs)
│           ├── ui_state.rs           # suppress_hide-Flag für Native-Modal-Interaktion
│           ├── expander.rs           # Trigger-basierter Text-Expander (AX/UIA primary, Clipboard-Fallback)
│           ├── text_field/           # FieldAccess-Trait + macOS-AX + Windows-UIA-Implementierungen
│           ├── paste.rs              # write_to_clipboard + enigo-Paste-Shortcut
│           ├── hotkey.rs             # Global Ctrl+Shift+V + Ctrl+Shift+O + Ctrl+Shift+S + Ctrl+Shift+C + Expander-Hotkey + Direct Slots
│           ├── clipboard_watcher.rs  # Event-getriebene Capture, RTF-Stripping (Image > Files-Priorität)
│           ├── recolor.rs            # Image-Tint (Lerp Target ↔ Weiß nach Per-Pixel-Luminanz)
│           ├── cutout.rs             # Legacy Chroma-Key-Cutout (als Fast-Path-Option behalten)
│           ├── cutout_ml.rs          # U²-Net-basierter Subject-Cutout via `ort` (ONNX Runtime)
│           ├── screen_picker.rs      # Farb-Eyedropper (NSColorSampler / GDI-Overlay)
│           ├── region_picker.rs      # screencapture-i-Wrapper — geteilt von OCR- + Screenshot-Pipelines
│           ├── ocr.rs                # Apple Vision (VNRecognizeTextRequest)-Wrapper
│           └── screen_recording.rs   # macOS-Bildschirmaufnahme-TCC-Permission-API — gated OCR + Screenshot
├── win/                     # Windows-spezifische Bundle-Shell
│   ├── README.md            # Windows-Install- & Build-Details
│   ├── package.json         # Tauri-CLI-Entry
│   └── src-tauri/           # main.rs, Cargo.toml, tauri.conf.json, capabilities/, icons/
├── macos/                   # macOS-spezifische Bundle-Shell
│   ├── README.md            # macOS-Install, Gatekeeper, Accessibility, Troubleshooting
│   ├── package.json
│   └── src-tauri/           # entitlements.plist, tauri.conf.json (dmg+app), capabilities/
├── .github/
│   └── workflows/
│       ├── ci.yml           # Rust- + Frontend-Tests bei jedem Push/PR
│       └── release.yml      # Baut Bundles und published GitHub-Release bei v*-Tags
├── docs/
│   ├── spec.md              # Originale Produkt-Spezifikation
│   ├── snippets-import.md   # JSON-Snippet-Import — Schema, Semantik, Beispiele
│   ├── notes.md             # Notes-Feature — Kategorien, Edit-Semantik, IPC-Surface
│   ├── backup.md            # Full-App-Export/Import — Schema, Merge-Semantik, jq-Rezepte
│   ├── text-expander.md     # System-weiter Expander — Workflow, Hotkey-Format, Per-OS-Caveats
│   ├── colors.md            # Inline-Hex-Preview + Custom-HSV-Picker + System-Eyedropper
│   ├── ai-prompts.md        # 25 gebündelte Default-AI-Prompt-Snippets
│   ├── encryption.md        # AES-256-GCM At-Rest-Encryption — Threat-Model, Key-Storage, Migration
│   ├── RELEASING.md         # Release-Procedure
│   └── examples/
│       └── snippets/        # 5 thematische JSON-Beispiele + eigene README
├── scripts/
│   └── check.sh             # cargo clippy + tsc + eslint
├── Cargo.toml               # Rust-Workspace (Members: core/rust-lib, win/src-tauri, macos/src-tauri)
├── pnpm-workspace.yaml      # pnpm-Workspace (core/frontend, win, macos)
└── package.json             # Root-Scripts: dev:{win,macos}, build:{win,macos}, lint, typecheck, format, test, check
```

## Quick Start

### Prerequisites

| Tool | Version | Hinweise |
|------|---------|----------|
| [Rust](https://rustup.rs/) | stable | MSVC-Toolchain auf Windows; `rustup component add clippy` ausführen |
| [Node.js](https://nodejs.org/) | 20+ | |
| [pnpm](https://pnpm.io/) | 10+ | `npm install -g pnpm` |

Plattformspezifische Prerequisites:
- **Windows** → [`win/README.md`](./win/README.md) (WiX, MSVC-Build-Tools, WebView2)
- **macOS** → [`macos/README.md`](./macos/README.md) (Xcode CLT, Gatekeeper, Accessibility-Permission)

### Install & run

```bash
pnpm install          # installiert den ganzen Workspace (CI nutzt --frozen-lockfile)

# Windows
pnpm dev:win          # tauri dev — Live-Reload
pnpm build:win        # → target/release/bundle/msi/InspectorRust_x.x.x_x64_en-US.msi

# macOS
pnpm dev:macos                      # tauri dev — Live-Reload
pnpm build:macos                    # → target/release/bundle/{macos/InspectorRust.app, dmg/InspectorRust_x.x.x_<arch>.dmg}
bash scripts/install-macos.sh       # build + re-sign + install nach /Applications + launch
bash scripts/install-macos.sh --reset  # …auch tccutil-reset stale Accessibility-Grants (nach First-Run nutzen)
```

> Warum der `install-macos.sh`-Helper? Ohne Apple-Developer-ID kriegt jedes frische `pnpm build:macos` einen neuen zufälligen Signing-Identifier, was macOS-TCC dazu bringt, jedes Rebuild als neue App zu behandeln und erneut nach Accessibility zu fragen. Das Script erzwingt einen stabilen ad-hoc Bundle-Identifier, sodass das Grant über Rebuilds hinweg überlebt. Voller Hintergrund: [`macos/README.md` — Accessibility-Permission](./macos/README.md#why-the-dialog-re-appears-after-every-rebuild-and-how-to-stop-that).

> Jede Plattform muss auf ihrem nativen Host gebaut werden (Windows für MSI, macOS für DMG/`.app`). Cross-Compilation wird nicht unterstützt.

### Snippet-Import

In Inspector Rust: Popup öffnen (`Ctrl+Shift+V`) → **Snippets**-Tab → **Import** → eine `.json`-Datei auswählen. Der native File-Picker öffnet (NSOpenPanel auf macOS, OpenFileDialog auf Windows); existierende Abbreviations werden in-place upsert, sodass Re-Import derselben Datei idempotent ist.

**Ready-to-import-Samples** in [`docs/examples/snippets/`](./docs/examples/snippets/):

| Datei | Snippets | Thema |
|-------|----------|-------|
| [`getting-started.json`](./docs/examples/snippets/getting-started.json) | 3 | Adresse, E-Mail, deutsche Signatur — First-Import-Test |
| [`signatures.json`](./docs/examples/snippets/signatures.json) | 4 | E-Mail-Signaturen (DE/EN, kurz, OOO-Template) |
| [`dev.json`](./docs/examples/snippets/dev.json) | 8 | Shebang, MIT-Header, fn-Skeletons, gitignore, Commit-Msg |
| [`markdown.json`](./docs/examples/snippets/markdown.json) | 5 | Headings, Table, `<details>`, PR-Body |
| [`wrapped-form.json`](./docs/examples/snippets/wrapped-form.json) | 2 | Demonstriert `{ "snippets": [...] }`-Shape |

Siehe [`docs/snippets-import.md`](./docs/snippets-import.md) für das volle Schema, die Field-Semantik, das sqlite3+jq-Export-Rezept und Tips/Anti-Patterns.

### Notes & Backup

Notes haben ihren eigenen Tab; die Kategorien-Sidebar hat **+ New Note** und **Clear All**. Backup lebt jetzt im **Settings**-Tab.

- **Clipboard-Eintrag als Note speichern:** Hover über jede History-Zeile → Bookmark-Icon klicken → Eintrag landet im `Uncategorized`-Bucket des Notes-Tabs. Verschiebe ihn durch Editieren der Note in eine Kategorie.
- **Full-Backup exportieren:** Settings-Tab → **Backup & restore** → was exportieren ankreuzen (Clipboard-History / Snippets / Notes — alle Default an) → **Export…** → Pfad wählen. Inspector Rust schreibt eine einzelne JSON-Datei (Default-Name `inspector-rust-backup-<timestamp>.json`); ungeticked Sektionen werden als leere Arrays geschrieben, sodass du Snippets teilen kannst ohne deine Zwischenablage zu leaken.
- **Backup importieren:** Settings-Tab → **Backup & restore** → **Import…** → die JSON-Datei wählen. Snippets und History mergen nach ihren natürlichen Keys (Abbreviation / SHA-256-Hash); Notes werden appended. Notes- / Snippets- / History-Tabs aktualisieren sich automatisch.

Volle Feature-Referenz: [`docs/notes.md`](./docs/notes.md). Backup-Datei-Schema und Merge-Semantik: [`docs/backup.md`](./docs/backup.md).

### Tests

```bash
pnpm test               # Frontend-Unit-Tests (vitest + happy-dom) — 86 Tests
cargo test --workspace  # Rust-Unit-Tests — 110 Tests (db, snippets, notes, backup, settings, expander, text_field, seed, hotkey-Parser, clipboard_watcher, models, recolor, cutout, cutout_ml)
```

Die gleichen Commands laufen in [GitHub-Actions-CI](./.github/workflows/ci.yml) bei jedem Push und PR.

### Statische Analyse

```bash
pnpm check            # cargo clippy (Workspace) + tsc --noEmit + eslint
```

## Bekannte Einschränkungen

| Einschränkung | Detail |
|---------------|--------|
| **Scope der At-Rest-Encryption** | Sensitive Inhalte (Clipboard-Text/-HTML/-RTF/-Bilder, Snippet-Bodies, Note-Bodies) sind AES-256-GCM-verschlüsselt at-rest mit einem per-Install zufälligen 256-Bit-Key (v0.6.0+). Key lebt im OS-Keychain; fällt zurück auf eine 0600-Keyfile im Data-Dir, wenn der Keychain nicht verfügbar ist. **Nicht verschlüsselt:** Timestamps, Content-Type-Tags, Dedup-Hashes, Snippet-Abbreviations, Note-Titles/-Kategorien — keines davon verrät Clipboard-Inhalt. Volle Referenz: [`docs/encryption.md`](./docs/encryption.md). |
| **Keine Sensitive-App-Detection** | Inspector Rust erfasst alles ohne Filterung. |
| **Kein Cloud-Sync** | Kein automatischer Sync oder Multi-Device-Support — aber der [Backup](./docs/backup.md)-Export/Import gibt dir eine portable JSON-Datei, die du manuell zwischen Maschinen bewegen kannst. |
| **File-Paste-Fallback** | Das Setzen von File-List-Clipboard-Payloads aus Rust wird nicht universell unterstützt; Inspector Rust fällt zurück darauf, die Newline-joined Liste der Pfade als Text zu pasten. |
| **Expander in Terminals: nimm einen Direct Slot** | Der *Abbreviation*-Expander macht nichts auf einer Terminal-Befehlszeile (Terminal.app, iTerm2, kitty, …) — Terminals exponieren die Input-Zeile nicht via Accessibility, und ein Shell-Prompt hat kein GUI-"vorheriges Wort markieren". Nimm dort einen **Direct hotkey → snippet**-Slot (v0.13.0 — pasted ohne irgendwas zu lesen, funktioniert überall) oder das Popup (`Ctrl+Shift+V` → suchen → Enter). Electron- / Chromium- / Mac-Catalyst-Apps (WhatsApp, Slack, VS Code, …) *werden* vom Abbreviation-Expander seit v0.12.0 unterstützt, via einen AX-select-then-paste-Pfad. |
| **macOS Accessibility** | Paste-Simulation (`enigo`) und der system-weite Text-Expander brauchen Accessibility-Zugriff. Erteile es einmal in Systemeinstellungen → Datenschutz & Sicherheit → Bedienungshilfen. Falls fehlend, zeigt Inspector Rust ein amber Banner mit einem `Open Settings`-Button beim nächsten Paste-Versuch — und seit v0.12.0 auch beim Drücken des Expander-Hotkeys — statt still zu failen oder den System-Dialog erneut zu feuern (v0.5.1 / v0.12.0). |
| **macOS Bildschirmaufnahme** | OCR (`Ctrl+Shift+O`) **und** Screenshot-Region (`Ctrl+Shift+S`, v0.15.0+) brauchen beide Bildschirmaufnahme-Zugriff — `screencapture -i` wird Inspector Rust zugeordnet und macOS verweigert es ohne das Grant. Pre-checked via `CGPreflightScreenCaptureAccess`; fehlende Permission öffnet das Popup + zeigt ein amber Banner, das auf den richtigen Privacy-Pane zeigt (v0.11.0). |
| **macOS unsigned Build** | Release-Builds sind nicht notarized. macOS warnt eventuell "unidentified developer" — Rechtsklick auf die App und **Open** wählen, um Gatekeeper beim ersten Launch zu umgehen. |
| **macOS Rebuild ⇒ Re-Grant** | `cdhash` ändert sich bei jedem source-affecting Rebuild, was vorherige TCC-Grants invalidiert. `scripts/install-macos.sh` skipt das Re-Signing, wenn der Source-Hash unverändert ist, sodass casual Rebuilds überleben; echte Source-Änderungen brauchen weiterhin Re-Granting. |
| **OCR + Screenshot sind aktuell macOS-only** | Region-Capture (geteilt von OCR und der v0.15.0-Screenshot-Pipeline) plus Vision-OCR shippen nur auf macOS. Windows-/Linux-Invocations von `ocrRegion()` / `screenshotRegion()` geben einen strukturierten `"not implemented on this platform"`-Error zurück; der Workspace baut cross-platform via Stubs. Windows-Region-Capture wird `Graphics.Capture` und Vision-äquivalentes OCR via `Windows.Media.Ocr` in einem Follow-up-Release nutzen. |

## Beiträge

Beiträge sind willkommen — siehe [`CONTRIBUTING.md`](./CONTRIBUTING.md) für den Dev-Workflow, Code-Style und wie man IPC-Commands oder neue Plattform-Shells hinzufügt.

## Releasing

Push ein `v*`-Tag, um den [Release-Workflow](https://github.com/pepperonas/inspector-rust/actions/workflows/release.yml) zu triggern, der die Windows- und macOS-Bundles baut und an einen GitHub-Release attached. Volle Procedure (Version-Bumps, Pre-Flight-Checks, Troubleshooting) in [`docs/RELEASING.md`](./docs/RELEASING.md).

## Changelog

Siehe [`CHANGELOG.md`](./CHANGELOG.md) — jeder Release ist dokumentiert mit dem, was hinzugefügt, gefixt wurde, und etwaige bekannte Issues zu der Zeit.

## License

[MIT](./LICENSE) — © 2026 Martin Pfeffer

Ein privates Open-Source-Side-Project — gebaut an Wochenenden und Abenden, gemacht mit ❤️.
