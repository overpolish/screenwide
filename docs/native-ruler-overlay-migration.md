<!-- SPDX-FileCopyrightText: 2026 overpolish -->
<!-- SPDX-License-Identifier: GPL-3.0-or-later -->

# Native ruler overlay migration

This is the cutover contract for Ruler. The existing React implementation is a legacy behavioural reference, not a visual template. Once the audit is accepted, every ruler DOM/SVG renderer, hook and TypeScript interaction model is removed before native implementation continues. This intentionally leaves Ruler incomplete for a checkpoint: an old overlay must never remain behind the GPU surface and accidentally participate in input, focus or rendering.

## Final ownership

Portable Rust owns the frozen multi-display document, measurements, guides, distance probes, radius stamps, label placement, hit testing, snapping, viewport transforms, history, hotkeys, clipboard actions, cursor policy, animation state and lifecycle. Shared OSC code owns semantic design tokens, text/icon atlases, material/translucent controls and platform-independent draw data. The macOS adapter owns desktop surfaces, Metal submission, native input, system cursors and theme/display notifications. Windows will implement the same adapter boundary later.

React owns no ruler UI. The single ruler webview may remain as an invisible native-surface owner, as Region and Text Recognition do, but its `/ruler` route renders `null`. The existing screenshot-session control plane may continue to invoke `set_ruler_screenshot_mode`; that lifecycle call does not render or store ruler state.

## Audited behaviour

### Session and desktop

- Starting Ruler dismisses incompatible capture overlays, captures one frozen snapshot per display, installs one session generation, raises the ruler and emits the shared capture lifecycle.
- The new ruler uses the standard desktop-union surface manager: one document, one input state and clipped per-display rendering across any number of monitors. The current per-monitor React documents and focus polling are not retained.
- Mixed DPI, display seams and display topology changes use the same shared projection and rebuild contracts as Region and OCR.
- Clicking/focusing another app dismisses the session. Escape dismisses from the first frame. Dismissal restores the system cursor, clears snapshots and removes every peer input surface.
- Quick Screenshot may temporarily preserve Ruler in the captured image. The ruler becomes passthrough, drops below Region, releases its cursor claim and resumes afterwards without losing its document or focus intent.

### Frozen-pixel analysis

- Rust already owns snapshot storage, generation invalidation, RGBA pixels, gradient maps and connected-component box detection in `src-tauri/src/ruler/{snapshot,analysis}.rs`.
- Native Ruler consumes these buffers directly; the snapshot, gradient and box IPC channels to React are deleted.
- Edge sensitivity has Clear edges, Balanced and Subtle edges modes, defaults to Balanced, cycles with `T`, and briefly presents the selected mode through the native translucent colour loupe. It affects only automatic transient rulers and the active guide preview. Stamped artifacts, detected boxes, labels and history remain unchanged.
- Cursor colour sampling returns an uppercase hex value. `Tab` copies it and briefly changes the readout to `Copied`.

### Viewport

- Zoom ranges from 1x to 16x around the pointer. macOS wheel pans unless the zoom modifier is active; pinch zooms. Middle-button drag pans. Double-click resets zoom and pan.
- Frozen pixels and every world-space artifact share one transform. Labels and cursor furniture remain crisp in screen space where appropriate.
- Pan is clamped so the frozen viewport cannot be moved beyond its zoomed extent.

### Measurements and snapping

- Primary-button drag creates a rectangular measurement. Rectangles smaller than two pixels on both axes are rejected.
- The draft snaps to detected component bounds using the existing edge and overlap heuristics. A worthwhile snap animates from the raw drag bounds to its settled bounds.
- Thin horizontal/vertical measurements label only their relevant dimension; normal boxes label `width × height px`.
- Persisted measurements use one Rust artifact identity for border and label hover. Hovering either drives the shared pulsing halo and makes that artifact the Delete/Backspace target; no persistent selection state exists. Copy uses the latest artifact. Live drafts reserve label width to avoid jitter, while stamped labels shrink to their exact text. Labels use temporary native drag ownership with portable world-space anchors.
- Optional centreline rendering shows measurement centres, sibling/content alignment accents, detected inner-object outlines and centre ticks. `M` toggles it. The legacy `B` detector-box diagnostic is deliberately excluded from the product interaction surface.

#### Delivered bounding-box enhancements

- Inner/content box detection within stamped measurement bounds uses the former ruler behaviour as a functional reference without restoring its UI implementation.
- Measurement-centre detection, centre ticks and centreline/alignment accents include relationships between the outer measurement and detected inner content.
- Detection, geometry and toggle state live in portable Rust; shared OSC draw data and the platform compositor own presentation. No React overlay or interaction path returns.

### Automatic and stamped distance probes

- At idle, horizontal and vertical probes find the nearest gradient edges around the pointer and show live distances plus cursor dimensions.
- Holding Option/Alt clips automatic probes against committed guides and measurement edges.
- Holding `1` or `2` begins an x/y range probe. Pointer movement resamples the active scanline and shows its current distance label; key release stamps the combined range. Its native cursor range mode is active only while the key is held.
- Probe lines have end ticks, can cross with an exact visual exclusion, carry movable/hideable labels and participate in hover deletion and history. Hovering either a stamped line or its label resolves to the same Rust artifact target and suppresses the automatic transient probes. Dragged label anchors live in world coordinates and migrate between monitor material surfaces. Right-clicking either a label or its box/ruler geometry toggles label visibility.

### Guides

- Holding `V` or `H` previews a vertical/x-axis or horizontal/y-axis info guide snapped to the strongest nearby gradient with hysteresis. Clicking stamps it and releasing the key removes the preview.
- A hovered guide can be picked up and moved through the same snap path; there is no persistent guide selection state.
- Adjacent guides produce gap measurements. Their labels use the midpoint of the guides' original cross-axis anchors so they do not chase the pointer.
- Deleting a gap label deletes the newer of its two guides. Guides and guide gaps participate in label movement/hiding and history.

### Corner-radius stamps

- Holding `R` previews the detected corner radius under the pointer; clicking stamps it. Low-confidence results use an approximate label and differentiated line treatment.
- Radius artifacts contain the corner arc, movable/hideable label, hover/delete target and history entry. The label communicates the measured radius without centre or leader lines.

### Labels, selection and history

- Measurement, probe, guide-gap and radius labels choose an inside/outside placement that remains readable and clamp/flip within the visible viewport.
- Labels are draggable. Secondary click toggles a persisted label's visibility. Hovering a label selects its owning artifact for deletion and reveals the appropriate system cursor.
- Hovered artifact lines use a short animated deletion halo.
- Undo/redo covers measurements, guides, probes, radii, hidden labels and label offsets. History is capped at 100 document snapshots. Bare clicks/no-op gestures do not clear redo.
- `Command/Control+Z`, `Command/Control+Shift+Z`, and Windows-style `Control+Y` preserve their existing meanings.

### Remaining keys and cursor states

- `X` toggles the full-screen crosshair.
- Escape dismisses. Blur clears held guide/probe/radius tools.
- The normal ruler hides the system pointer and renders native cursor furniture. Guide placement/movement, radius mode, range probes and label interaction expose the relevant native cursor without an initial arrow flash or a stale global cursor claim after dismissal.

## New visual contract

- Rulers, distance probes, radius stamps, measurement bounds, centre accents and the optional crosshair use the theme-appropriate **primary** semantic palette.
- Guides and guide-derived gaps use the theme-appropriate **info** palette.
- Measurement/readout text uses the bundled **Roboto Mono** atlas with tabular numerals.
- Labels use a shared translucent, theme-reactive OSC treatment with semantic outline/text tokens. They are not copies of the old solid black/white SVG chip and do not create one lagging native material window per label.
- All strokes, dashes, caps, arcs and outlines use analytical anti-aliasing and target-display pixel snapping. Live theme switching redraws without rebuilding the session.
- The already-migrated bounding-box OSC is the starting primitive for box measurements; it is extended or parameterized rather than reimplemented.

Exact line weights, label metrics, opacity, motion and state treatments are redesigned and approved incrementally while each native feature lands.

## React deletion boundary

After approval of the strip checkpoint:

- Delete the complete `src/features/ruler` legacy UI/interaction tree, including its stories and TypeScript tests.
- Replace `RulerWindow` in `src/App.tsx` with a null native-host route.
- Retain only the screenshot-session lifecycle call. Move its tiny Tauri API wrapper out of the deleted ruler feature tree if needed.
- Remove obsolete ruler commands from the Tauri invoke handler: `get_ruler_cursor_position`, `set_ruler_cursor_visible`, `set_ruler_cursor_range_active`, `get_ruler_snapshot`, `get_ruler_gradients`, `get_ruler_boxes`, `copy_ruler_value`, and any frontend-only start/cancel command no longer referenced.
- Keep Rust snapshot/analysis code, shortcut/tray launch, overlay exclusion, screenshot-mode lifecycle, capture lifecycle and cursor guard. These become direct native dependencies rather than IPC endpoints.
- Replace per-monitor `ruler-*` webviews and `focus.rs` polling with one owner plus compositor peer panels when the native surface is introduced.

## Native delivery order after stripping

1. Standard desktop surface, frozen snapshots, lifecycle, Escape, cursor and topology; no ruler artifacts yet.
2. Shared snapshot texture plus cursor colour/readout and live crosshair.
3. Bounding-box measurement, Rust snapping and settle animation.
4. Automatic/stamped distance probes and copy/delete/history.
5. Info guides, moving, gap labels and guide deletion semantics.
6. Radius preview/stamps, centreline aids and remaining view toggles.
7. Label drag/hide, full undo/redo, screenshot preservation, accessibility and final legacy/IPC residue audit.

Every step is a user-testable checkpoint. Windows compositor plumbing remains deferred until all three macOS overlays are complete, while state, geometry, input semantics and draw contracts remain portable Rust.

### Checkpoint status

1. **Implemented on macOS:** one null React owner hosts the standard native desktop-union surface and compositor peer panels. Frozen snapshots, native crosshair input, deferred global Escape, complete peer teardown, capture lifecycle, screenshot passthrough and generation-safe topology rebuilding are active. No ruler artifact rendering or React fallback exists.
2. **Implemented on macOS:** frozen-pixel colour sampling, the translucent Roboto Mono readout, `Tab` copy feedback and the optional full-desktop crosshair share the native surface and live theme path.
3. **Implemented on macOS:** measurement drafts, component snapping, settle animation and cross-display bounds render through the shared ruler box primitive.
4. **Implemented on macOS:** the portable Rust artifact document persists multiple measurements, provides stable artifact identity and hover targeting, and owns capped snapshot undo/redo. The compositor consumes a variable-length artifact draw list; Delete/Backspace, `Command/Control+C`, undo and redo route directly through the native ruler input path.
5. **Current test checkpoint:** each display owns an independent portable Rust viewport with pointer-centred 1×–16× zoom, clamped pan and monitor-local reset. macOS routes plain wheel pan, Control-wheel/pinch zoom, middle-button pan and double-click reset into that shared state. Frozen pixels, sampling, measurements and labels use the same per-display projection, including artifacts clipped across a monitor boundary.
6. **Implemented on macOS:** idle horizontal and vertical probes use a per-display edge index built once from the retained Rust gradient maps, render as uninterrupted Metal lines with endpoint ticks, and project through the active monitor viewport. Live probes carry no individual labels; their combined `width × height px` remains in the native colour readout. All transient probe chrome participates in Quick Screenshot suppression.
7. **Current test checkpoint:** holding `1` or `2` starts a shared Rust horizontal/vertical range, pointer movement resamples the active scanline and presents its current distance label, and matching key-up stamps the combined ruler. Stamped probes persist across displays and viewports, use stationary translucent Roboto Mono labels, and resolve line and label hover to the same Rust target for transient suppression, deletion, copy and capped undo/redo. They remain visible when Quick Screenshot suppresses transient cursor furniture.
8. **Current test checkpoint:** measurement borders and their native material labels resolve to the same ephemeral Rust hover target and use a single-coverage analytical pulsing halo. Newly drawn boxes and stamped rulers stop highlighting when the pointer leaves; there is deliberately no persistent selection state or native selected flag. Delete prioritizes the hovered artifact and copy uses the latest. Hover flags do not invalidate label content, avoiding material-surface flicker; stamped labels discard the live draft's reserved character width.
9. **Current test checkpoint:** measurement and stamped-probe labels share portable Rust anchor/visibility state. Native material labels advertise movement with an open-hand cursor and use closed-hand drag ownership observed by the native event monitor while AppKit retains the complete mouse sequence. They retain the pointer grab offset, move between display surfaces through world-space projection, and record one history entry only after crossing the drag threshold. Right-clicking either a label or its box/ruler geometry toggles visibility at the previous anchor. Movement and visibility both participate in undo/redo. Hovering any committed artifact or label suppresses the complete transient probe presentation, including the colour loupe.
10. **Current test checkpoint:** holding `V` or `H` starts a portable vertical/horizontal guide preview on the monitor under the pointer. The preview searches frozen gradient data near the pointer, retains a snapped edge through a wider hysteresis release radius, migrates between monitors, and suppresses automatic probes. Clicking stamps an info-coloured guide into the shared document and capped history while keeping the held preview active; releasing the key removes only the preview. Guide movement, gap labels and guide targeting follow in the next checkpoint.
11. **Current test checkpoint:** `T` cycles shared portable edge sensitivity from the default Balanced through Subtle edges, Clear edges and back to Balanced. Switching is immediate: the retained Balanced index remains available, while alternate modes scan only the current cursor row and column from frozen gradients; an active guide preview resnaps within its local search radius. Stamped rulers, guides, measurements, detected boxes, labels and history are never rebuilt or mutated. The redesigned theme-reactive colour loupe briefly presents the selected mode and participates in transient-chrome suppression.
12. **Current test checkpoint:** stamped info guides are ephemeral hover targets with axis-appropriate resize cursors and can be dragged through the same frozen-gradient snapping path used by guide previews. The drag records one history snapshot only after crossing the movement threshold and can migrate between monitor surfaces. Adjacent same-axis guides derive an info-coloured gap ruler with endpoint ticks and an exact translucent Roboto Mono distance label. Moving the label moves the gap ruler on its cross-axis; hiding the label hides that complete derived ruler. Gap labels share the standard hover, open/closed-hand movement, right-click visibility, Delete and undo/redo paths; deleting a gap removes the newer guide and reconciles neighbouring gaps. Holding Option/Alt makes automatic transient probes stop at the nearest same-monitor stamped guides. No persistent selected state is introduced.
13. **Current test checkpoint:** holding `R` runs the portable frozen-gradient corner fitter under the pointer and previews a primary-coloured analytical quarter arc. Each sensitivity level has a frozen candidate cache, so the active Clear/Balanced/Subtle mode affects both candidate discovery and curve fitting without mutating stamps. The base curve and single-coverage halo are evaluated and derivative-antialiased in Metal after device-pixel snapping, avoiding segmented joins at high zoom. Clicking while held stamps the detected radius; release removes only the preview. Low-confidence fits use an approximate Roboto Mono label and broken-line treatment. Radius visuals deliberately contain no centre or label-leader lines. Radius geometry and labels use the same viewport projection, material surface, ephemeral hover halo, open/closed-hand label movement, right-click visibility, Delete, copy and capped undo/redo document paths as the existing ruler artifacts. The colour loupe is idle-only and hides for every held tool, drawing gesture and artifact/label drag.
14. **Current test checkpoint:** committed measurements derive centre aids from a cached portable Rust analysis of the retained Balanced component boxes. Nearby parts cluster into at most twelve visual inner objects; the outer container, specks and duplicate self-bounds are excluded. Faint primary inner outlines gain centre ticks on aligned axes, while measurement centre lines strengthen when their axis aligns with the inner-content union or a sibling measurement. Settle animation moves only the outer centre lines and withholds fixed inner outlines until landing. macOS projects and pixel-snaps the shared draw records through Metal across zoom and monitor surfaces. A non-repeating `M` key toggles the complete aid view, enabled by default; no detector diagnostic or React path is restored.
15. **Final macOS audit checkpoint:** the `/ruler` frontend route is a null native host and retains only the shared screenshot/cancel control plane. The obsolete ruler cursor swizzle and unused measurement payload in the shared native input ABI are removed, with matching Rust and Objective-C layout assertions. Native colour, measurement, distance, guide-gap and radius readouts expose semantic static-text accessibility labels and current values without changing pointer interaction. The legacy `B` detector diagnostic remains intentionally excluded. The macOS Ruler implementation is ready for runtime sign-off before Windows compositor plumbing begins.
