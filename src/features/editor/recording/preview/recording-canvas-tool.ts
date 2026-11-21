// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { AnnotationKind } from "../../../annotations/types";

/** What a press on the recording's picture does: Select, a pen, or the
 * frame's own drag (`canvas`) or the crop of the layer chosen. Select is
 * what is in hand when nothing else is. */
export type RecordingCanvasTool = AnnotationKind | "canvas" | "crop" | "select";
