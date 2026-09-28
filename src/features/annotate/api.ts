// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";

/** The plate reporting the size it needs. The window is sized and placed to
 * it, and shown once it has been. */
export const resizeAnnotateToolbar = (width: number, height: number) =>
  invoke<null>("resize_annotate_toolbar", { height, width });

/** A press on the toolbar was released. If it was a drag, Rust pulls the
 * window back on screen and keeps its place against the display it landed
 * on. */
export const finishAnnotateToolbarDrag = () =>
  invoke<null>("finish_annotate_toolbar_drag");

/** A field on the plate took focus. The toolbar's window refuses the keyboard
 * until asked for it, so until this the keystrokes go to the picture and pick
 * up tools instead of arriving in the field. */
export const beginAnnotateToolbarTyping = () =>
  invoke<null>("begin_annotate_toolbar_typing");

/** The field is done with: the keyboard goes back to the picture. */
export const endAnnotateToolbarTyping = () =>
  invoke<null>("end_annotate_toolbar_typing");

export const undoAnnotation = () => invoke<null>("undo_annotation");

export const clearAnnotations = () => invoke<null>("clear_annotations");

/** Leaves the overlay, as Escape does. */
export const dismissAnnotate = () => invoke<null>("dismiss_annotate");
