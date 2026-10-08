// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/**
 * The parts of an annotation's dress that more than one feature chooses.
 *
 * The editor dresses the annotation in hand and the live overlay dresses the
 * next stroke, so the controls and the values they carry are shared rather than
 * owned by either. The types are generated from the Rust models by ts-rs.
 */

import type { AnnotationAlign } from "../../../bindings/AnnotationAlign";
import type { AnnotationHead } from "../../../bindings/AnnotationHead";
import type { AnnotationKind } from "../../../bindings/AnnotationKind";
import type { AnnotationRedaction } from "../../../bindings/AnnotationRedaction";

export type {
  AnnotationAlign,
  AnnotationHead,
  AnnotationKind,
  AnnotationRedaction,
};
