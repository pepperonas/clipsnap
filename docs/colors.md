# Hex colors — inline preview + picker + screen eyedropper

Three small but useful tools that landed in **v0.4.0** (preview, picker), evolved through **v0.5.0** (custom HSV modal), **v0.5.1** (click-to-select UX), and **v0.5.2** (system-wide eyedropper).

## Inline hex preview

Type a hex color in the search input and ClipSnap surfaces a color row at the top of the list — same pattern as the inline calculator.

| You type            | Result                                              |
|---------------------|-----------------------------------------------------|
| `#3366FF`           | Color row, canonical `#3366FF`                      |
| `3366ff`            | Color row, normalised to `#3366FF` (uppercase)      |
| `#abc`              | Expanded to `#AABBCC`                               |
| `#abcd`             | 4-digit RGBA short form, alpha = 0xDD/0xFF          |
| `#3366FF80`         | 8-digit RGBA, alpha ≈ 0.50                          |
| `abc` *(no hash)*   | **Rejected** — too ambiguous with search input      |
| `f00d` *(no hash)*  | **Rejected**                                        |
| `#xyzabc`           | Rejected (non-hex chars)                            |

**Activation.** Press <kbd>Enter</kbd> on a color row → ClipSnap pastes the **canonical** `#RRGGBB` (or `#RRGGBBAA` if alpha < 1), uppercase, into the previously focused app.

**Preview pane** for a selected color row:

- Big 128 px swatch with the hex overlaid, foreground auto-picked black or white via WCAG relative-luminance for readability.
- Three rows below: `Hex`, `RGB`, `HSL` — each with a copy-to-clipboard button.

**Implementation.** All of this is pure frontend in [`core/frontend/src/lib/colors.ts`](../core/frontend/src/lib/colors.ts). 24 vitest cases cover valid / invalid input, canonical formatting, RGB ↔ HSL conversion, and the WCAG-based foreground picker.

## Color picker modal (v0.5.0+)

The History tab's toolbar has a **Color picker** button (palette icon, next to the clip count). Click it → an in-app HSV modal opens.

### Why a custom modal, not `<input type="color">`?

v0.4.0 originally used a hidden `<input type="color">` to summon the OS-native picker (NSColorPanel / Win32 ChooseColor / GTK ColorChooser). It turned out to be unreliable in WKWebView (Tauri's macOS renderer):

- The OS picker often didn't open at all from a hidden input.
- When it did, `navigator.clipboard.writeText` was *blocked* because the input's `change` event fires outside the user-gesture window.

The v0.5.0 custom modal runs entirely in the WebView and writes through `@tauri-apps/plugin-clipboard-manager`'s `writeText` (which is not subject to browser-API restrictions). Now reliable on all platforms.

### Layout

| Region | Description |
|---|---|
| **Header** | Title, eyedropper button (v0.5.2), close (Esc) button. |
| **Saturation/Value picker** | 2D area, ~440 × 176 px. Background driven by current hue. Click to set saturation (x) and value (y). |
| **Hue slider** | Horizontal rainbow strip. Click or drag to set hue. |
| **Big preview swatch** | Shows the picked color with hex overlaid. Foreground picked via WCAG luminance (black on light, white on dark). |
| **Hex input** | Editable. Accepts `#RRGGBB`, `RRGGBB`, `#RGB`, `#RGBA`, `#RRGGBBAA` — same parser as the inline preview. |
| **Format tabs** | `HEX` / `RGB` / `HSL`. Switches the readout in the strip below the input. |
| **Action row** | Close (Esc) and **Copy `<format>`** buttons. Flashes "Copied!" green for 2 s on success. |

### Click-to-select UX (v0.5.1)

Opening the modal puts it in a *no selection yet* state:

- Empty hex input.
- Dashed-border placeholder swatch reading "Click in the picker above (or type a hex) to select a color".
- Format readout shows `—`.
- **Copy** button is disabled.
- SV-picker crosshair indicator is hidden.

The **first** click in the SV picker (or hue slider drag, or hex typed) is the selection. This matches the user's mental model: clicking the toolbar button = *click 1, opens the modal*; the next click = *click 2, picks the color*. Closing and re-opening the modal resets to no-selection.

### Internal state

HSV (hue, saturation, value) is the source of truth — it maps cleanly to a 2D picker plus 1D hue slider. RGB and HSL outputs are derived on demand. There's a tiny rounding loss in the HSV → RGB round-trip for arbitrary inputs (1 unit at most), which is invisible to the eye.

## System-wide screen eyedropper (v0.5.2)

The modal's header has a **Pick from screen** button (pipette icon) that lets you sample a color from **anywhere on the desktop**, not just inside ClipSnap's own UI.

### Behaviour

- Click **Pick from screen** → the loupe magnifier appears under the cursor.
- Move anywhere on screen — the loupe follows, magnifying the pixels under the cursor for precise targeting.
- Click → samples the pixel under the cursor; the picked hex is inserted into the modal automatically (HSV state, hex input, all three format tabs all populate).
- Press <kbd>Esc</kbd> → cancels (no change to modal state).

The popup window stays visible during the pick; if it covers the area you want to sample, drag it out of the way first. (Hiding the popup turned out to break NSColorSampler's loupe rendering on macOS Tahoe — no key window means no loupe.)

### Per-platform implementation

| OS      | Mechanism |
|---------|-----------|
| **macOS** | Apple's `NSColorSampler` (AppKit, 10.15+) — the same magnifier loupe that Pages, Keynote, and Sketch use. Invoked via `objc2` 0.6 + `block2` 0.6 raw `msg_send`. The app briefly promotes its activation policy from `Accessory` to `Regular` so the loupe renders, then demotes back 500 ms after the popup is restored. |
| **Windows** | A fullscreen layered overlay window + `GetPixel` on the desktop DC. Esc to cancel. Spawned on a worker thread so the message loop doesn't block the Tauri UI. |

### Architecture

The IPC `pick_screen_color` is *fire-and-forget*: it kicks off the platform sampler and returns immediately. When the user clicks (or cancels), Rust emits a Tauri event `color-picked` with payload `string | null` (`null` = cancelled / failed). The frontend's `useEffect` listener picks it up and updates the modal's state.

This decoupling matters because:

- The macOS `NSColorSampler` is asynchronous (block-based callback). Trying to make it look synchronous would block the Tauri main thread and freeze the UI while waiting for the user.
- The Windows side runs a blocking message loop on a worker thread. Same async interface keeps the IPC consistent across platforms.

The popup auto-hide handler is suppressed while the eyedropper is up (via `UiState.suppress_hide`), so focus loss to the sampler overlay doesn't tear down the popup.

### Sources

- [`core/rust-lib/src/screen_picker.rs`](../core/rust-lib/src/screen_picker.rs) — both platform implementations, ~180 lines.
- [`core/rust-lib/src/commands.rs::pick_screen_color`](../core/rust-lib/src/commands.rs) — IPC orchestrator.
- [`core/frontend/src/components/ColorPickerModal.tsx`](../core/frontend/src/components/ColorPickerModal.tsx) — modal + event listener + eyedropper button.

## See also

- [`core/frontend/src/lib/calc.ts`](../core/frontend/src/lib/calc.ts) — the sibling inline calculator. Same Alfred-inspired pattern.
- [`docs/text-expander.md`](./text-expander.md) — the other "triggered by typing" feature.
