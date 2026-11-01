// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { AnnotationKind } from "../../../annotations/types";

/** What a press on the recording's picture does: Select, a pen, the
 * marquee, or a mode's own drag, the frame's (`canvas`), the crop's or the
 * scene's. Select is what is in hand when nothing else is. */
export type RecordingCanvasTool =
  AnnotationKind | "canvas" | "crop" | "marquee" | "scene" | "select";
