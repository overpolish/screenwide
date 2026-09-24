// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/**
 * The parts of an annotation's dress that more than one feature chooses.
 *
 * The editor dresses the annotation in hand and the live overlay dresses the
 * next stroke, so the controls and the values they carry are shared rather than
 * owned by either. The twins of `AnnotationHead` and `AnnotationAlign` in
 * `src-tauri/src/editor/annotations/model.rs`.
 */

/** Which ends of an arrow carry a head. */
export type AnnotationHead = "none" | "end" | "both";

/** How a text box lines up its lines against each other. */
export type AnnotationAlign = "left" | "center" | "right";

/**
 * Which shape a tool draws. The twin of `AnnotationKind` in
 * `src-tauri/src/editor/annotations/kind.rs`, and the one spelling of the
 * union: the editor, the live overlay and the shared controls all name a
 * kind from here rather than respelling it.
 */
export type AnnotationKind = "arrow" | "counter" | "text";
