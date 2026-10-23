// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";

/** Whether the overlay is drawing, which is when the island carries the
 * annotate tools. Changes arrive as `annotate://drawing`. */
export const isAnnotateDrawing = () => invoke<boolean>("annotate_drawing");

/** A press on the island is done with: the keyboard goes back to the
 * picture, where the letters pick tools up. */
export const returnAnnotateKeyboard = () =>
  invoke<null>("return_annotate_keyboard");

export const undoAnnotation = () => invoke<null>("undo_annotation");

export const clearAnnotations = () => invoke<null>("clear_annotations");

/** Leaves the overlay, as Escape does. */
export const dismissAnnotate = () => invoke<null>("dismiss_annotate");
