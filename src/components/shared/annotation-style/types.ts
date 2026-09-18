// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/**
 * The parts of an annotation's dress that more than one feature chooses.
 *
 * The editor dresses the annotation in hand and the live overlay dresses the
 * next stroke, so the controls and the values they carry are shared rather than
 * owned by either. The twin of `AnnotationHead` in
 * `src-tauri/src/editor/annotations/model.rs`.
 */

/** Which ends of an arrow carry a head. */
export type AnnotationHead = "none" | "end" | "both";
