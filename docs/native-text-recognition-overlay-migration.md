<!-- SPDX-FileCopyrightText: 2026 overpolish -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Native Text/OCR overlay migration

This is the cutover contract for Text/OCR. It is deliberately stricter than a visual-parity checklist: once a responsibility moves native, the React drawing and event path that previously owned it is removed in the same checkpoint. There must never be two interactive overlays, a hidden DOM fallback, or a DOM surface following native geometry.

## Final ownership

Rust owns the session state machine, desktop-space geometry, frozen snapshots, OCR/QR results, text ranges, hit testing, commands and lifecycle. The shared OSC modules own control metrics, semantic colours, interaction transitions and portable draw data. Platform adapters own only windows, native material hosts, GPU submission, system cursors and accessibility bridges.

React retains one app-level responsibility: the complex QR-details dialog. It receives an activated QR payload from Rust and may copy content or open a URL. It does not draw QR hotspots, snapshots, selection chrome, text regions, status UI or controls over the desktop.

## Current behaviour that must survive

### Session and region selection

- Starting OCR dismisses every other capture overlay and any earlier OCR session, captures a frozen image of every display, raises the overlay and emits the shared capture lifecycle.
- The desktop is one logical coordinate space. A drag may begin on one display, cross display boundaries and end on another. Per-display surfaces project only their intersecting piece of the one region, including correct hidden edges at seams.
- Mixed-scale display pieces compose from the frozen snapshots with the shared desktop capture plan. No active-monitor or monitor-selection step is exposed.
- A region smaller than 2 logical points in either dimension is rejected.
- The selecting state has a crosshair, frozen desktop pixels, a 20% black shade and the current native bounding-box OSC.
- Escape and Close dismiss every surface, cancel in-flight work, clear frozen images, restore pointer interaction and emit the inactive lifecycle exactly once.
- Display connection, disconnection, resolution, scale and arrangement changes rebuild the standard per-display surfaces. Frozen snapshots are recaptured; an existing desktop rect is preserved and clamped where possible, otherwise the session returns to selecting.

### Recognition states

- Finishing a valid region freezes that desktop rect, starts recognition and shows `Finding text and QR codes…` in a native material status surface.
- Recognition failure returns to selecting, keeps the session dismissible and presents the error without leaving a stale input surface.
- Ready keeps the selected frozen pixels visible and changes the bounding-box state from selecting/loading to the ready treatment.
- Reset clears capture, results, text ranges and errors, then returns to a new desktop-wide selecting state without closing the OCR session.

### Recognized text interaction

- Every recognized line has an idle primary region. Selected character ranges use the stronger selected primary treatment.
- Pointer down chooses the nearest text insertion position, pointer drag updates the live range, and pointer up commits it.
- Double-click selects the complete recognized line.
- Command/Control modifies pointer selection additively; otherwise a new selection replaces committed ranges.
- Command/Control+A selects from the beginning of the first line through the end of the last line.
- Command/Control+C copies the merged committed plus live ranges and dismisses OCR. Empty selections do nothing.
- Range extraction merges overlaps per line, preserves line breaks and uses UTF-16 character offsets supplied by the platform recognizer.
- When character boxes are unavailable, selection geometry falls back to a proportional slice of the line bounds.
- The pointer uses the text cursor over recognized text and the appropriate control cursor over QR/actions.

The algorithms currently in `src/features/text-recognition/text-selection.ts` move to portable Rust with parity tests over the old TypeScript fixtures before the TypeScript copy is deleted.

### QR interaction

- Each recognized QR result has a native hotspot projected inside the selected desktop region.
- QR hotspot styling must remain legible over both dark content and the white backgrounds typical of QR codes. Its semantic fill opacity and outline are tuned and tested against both extremes rather than inheriting the text-region treatment unchanged.
- Every decoded QR opens the React QR-details window first. Actionable URLs
  require an explicit confirmation from that window before the safe opener
  flow runs.
- Decode failures and unsupported structured payloads use the error treatment and retain the `Unsupported QR` label.
- The dialog retains its payload label, diagnostic/error text, selectable raw content, Close, Copy content and open/copy failure handling.
- The QR-details dialog remains React rather than moving into the GPU compositor. Its behaviour is already suitable, but its legacy presentation must be rebuilt from current UI components and tokens and covered by dedicated feature stories before the OCR migration is complete.
- Activating a URL successfully dismisses the OCR session. Copying content
  keeps QR Details open so its confirmation feedback remains visible.

### Controls and accessibility

The ready-state native material dock contains, in order:

1. `Copy all`: compact neutral Button, Copy icon, copies `result.text`, dismisses.
2. `Copy as paragraph`: compact neutral Button, Pilcrow icon, copies the current selection or all text with line breaks collapsed, dismisses.
3. `Recognize another area`: compact neutral IconButton, RotateCcw icon, resets.
4. `Close text recognition`: compact neutral IconButton, X icon. First press arms it for two seconds and swaps to the error-coloured Trash icon; the second press dismisses. Timeout restores the idle state.

Before results are ready, the same two-stage Close control remains reachable in a small native material host. Every control exposes its current accessible label and disabled/armed state through the platform accessibility adapter.

Button, IconButton and material metrics/colours are taken from `src-tauri/src/osc/controls`; new tokens are added there only as a migrated OSC uses them. Icons come from one shared 4× Lucide alpha atlas decoded by Rust and uploaded once by each platform renderer. `ConfirmAction` is a reusable Rust control wrapper configured with arbitrary idle/armed icons, semantic colours and timeout; OCR only supplies the action invoked after confirmation.

Material-backed OSC controls always use their semantic solid/translucent fill. Ghost controls are intentionally absent from the native OSC contract because transparency exposes the backing material and produces a different visual surface; React may continue using ghost controls inside ordinary UI containers.

## Responsibility and deletion map

| Current React responsibility | Native destination | React deletion after parity |
| --- | --- | --- |
| Per-monitor canvas snapshot transport and painting | OCR session snapshot store plus standard per-display GPU surfaces | `frozen-monitor-snapshot.tsx`, `image-url.ts`, `getTextRecognitionSnapshot` API |
| Pointer region creation, crosshair and shade | Shared desktop region controller/surfaces | Selecting handlers and state in `text-recognition-window.tsx` |
| Selection frame and loading/ready border | Shared bounding-box OSC with OCR state | `SelectionFrame` usage |
| Cropped PNG round-trip into an `<img>` | Native frozen desktop composition | `CapturedTextRegion.imagePng`, object-URL handling and `<img>` |
| Loading label and error banner | Retained native material status surfaces | Corresponding DOM status nodes |
| Line and selected-range rectangles | Shared native rectangle primitives | `SelectionOverlay` usage |
| Text hit testing, ranges and keyboard shortcuts | Portable Rust OCR selection model | `text-selection.ts` and window keyboard/pointer handlers |
| Ready toolbar layout and actions | Native material control dock using shared Button/IconButton OSC | `text-recognition-actions.tsx`, toolbar measurement/layout |
| QR hotspot drawing and pointer activation | Native QR hotspot primitive plus Rust activation | `qr-code-overlay.tsx` |
| QR details | Dedicated React dialog window/route | Retained as a renamed dialog-only component |
| OCR overlay window lifecycle | Native OCR surface/session manager | Per-monitor OCR webview construction and overlay route |

The following legacy shared production components become unused when the OCR cutover completes and are deleted with their legacy stories:

- `src/components/shared/canvas-tools/canvas-toolbar.tsx`
- `src/components/shared/canvas-tools/selection-frame.tsx`
- `src/components/shared/canvas-tools/selection-overlay.tsx`

`ConfirmActionButton` is not deleted: recording and export UI still use it. OCR merely stops importing it once the native two-stage IconButton is live.

## Incremental cutover gates

### 1. Desktop selection and capture

Status: implemented on macOS. The native Region context now has a Text/OCR purpose, frozen display surfaces, Rust-owned completion, cross-display still composition and automatic topology restart. The React snapshot channel, monitor-local capture command and selecting pointer/render path have been deleted.

- Reuse the Region overlay's per-display surface manager, desktop coordinate conventions, cursor guard, Escape handling and topology observer.
- Store all display snapshots and compose one cross-display frozen selection.
- Verify selection, cancellation, topology changes and mixed 1x/2x seams.
- Then remove React snapshot rendering, shade, selecting-state frame, pointer region handlers and the monitor-local capture IPC. React must not remain as a passive visual follower.

### 2. Recognition state and result geometry

Status: implemented on macOS. OCR consumes the Rust-composed frozen capture without a PNG/frontend round trip. Portable Rust state projects line and QR geometry into desktop coordinates; each native display surface clips and draws its intersecting geometry with shared semantic tokens. Loading and errors use the retained native material host, while ready uses the native selection border. The React snapshot, frame, shade, loading/error and result-rectangle components and their legacy stories have been deleted. React temporarily retains only the input paths scheduled for gates 3–5.

- Feed the composed frozen capture directly to existing Rust OCR and QR recognition.
- Draw loading/error/ready state, line rectangles and QR hotspots natively on every intersecting display surface.
- Then remove the corresponding React state chrome and overlay nodes.

### 3. Native text selection

Status: implemented on macOS. Portable Rust now owns UTF-16 insertion hit testing, additive/live ranges, selected geometry, merged text extraction and paragraph conversion. Ready OCR surfaces accept pointer and keyboard input on every display, render selected ranges through the shared GPU palette, and own text/arrow cursor transitions. The legacy React selection state, handlers and `text-selection.ts` implementation have been deleted.

- Port insertion hit testing, ordered ranges, rectangle extraction, overlap merging, selected text and paragraph conversion to portable Rust.
- Prove fixture parity with the TypeScript implementation, then add pointer, double-click, modifier, select-all and copy integration tests.
- Then delete `text-selection.ts` and all React text pointer/keyboard handling.

### 4. Native control dock

Status: implemented on macOS. The ready-state controls use four independent material-backed Button/IconButton OSC surfaces with no enclosing toolbar chrome. Portable Rust owns placement, the shared supersampled Lucide atlas, interaction tokens and the generic animated two-stage `ConfirmAction`; OCR owns only its copy/reset/dismiss callbacks. The React toolbar, DOM measurement path and legacy `CanvasToolbar` component/stories have been deleted.

- Use the retained material-backed control surface proven by Recenter.
- Add only the required Copy, Pilcrow, RotateCcw, X and Trash icon assets and the two-second armed-control state.
- Verify dock placement above/below and clamping against the desktop union, hover/press/armed transitions, live theme switching and accessibility.
- Then delete `text-recognition-actions.tsx`, `CanvasToolbar` and toolbar DOM measurement.

### 5. QR dialog split and final webview removal

Status: React QR hotspot rendering, the legacy OCR window component, frontend
recognition kickoff, result events and their obsolete IPC surface have been
deleted on macOS. The redesigned QR Details window is the only retained React
feature UI. One transparent `text-recognition` webview owns keyboard focus and
the native root surface; compositor-created peer panels cover every additional
display. No per-monitor React or webview overlay remains.

- Route native hotspot activation to one dialog-only React surface.
- Preserve QR payload classification tests and opener/copy failure behaviour.
- Redesign the retained dialog with the current UI system and add feature stories for its normal, unsupported, decode-error and copy/open-failure states.
- Do not move the dialog itself into the GPU compositor; only the desktop hotspot and activation routing remain native.
- Delete `TextRecognitionWindow`, obsolete IPC commands/types and
  `QrCodeOverlay`, then collapse the per-monitor OCR webviews to one native
  owner.
- Confirm no OCR canvas, image, desktop-sized DOM node or pointer handler remains in the frontend.

## Parity gate for every checkpoint

- One authoritative region/session state and one input owner.
- No frame-delayed DOM geometry echo and no second overlay behind the native one.
- Multi-display, mixed-scale and display-topology tests pass.
- Escape/Close/Reset leave no visible or input-blocking surface.
- Theme changes update live without reconstructing the session.
- Capture and OCR results use frozen pixels; overlays never contaminate them.
- Rust unit tests, platform shader compilation, strict lint and the affected app build pass before user runtime testing.

## Not part of this migration

- OCR recognition quality and platform Vision/Windows OCR algorithms remain unchanged.
- QR-details remains a normal redesigned React window rather than an OSC.
- Windows native OSC plumbing remains deferred until the three macOS overlay migrations are complete, but every state, geometry and draw contract added here must remain portable Rust so Windows is primarily platform plumbing.
