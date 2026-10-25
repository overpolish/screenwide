// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ANNOTATE_SCENE } from "./annotate-scene";
import { OCR_SCENE } from "./ocr-scene";
import { RULER_SCENE } from "./ruler-scene";

import type { ToolScene } from "./scene-kit";

// The tools Tools shows at work, each drawn by the app on its own scene.

export type PreviewTool = "annotate" | "ruler" | "textRecognition";

export const TOOL_SCENES: Record<PreviewTool, ToolScene> = {
  annotate: ANNOTATE_SCENE,
  ruler: RULER_SCENE,
  textRecognition: OCR_SCENE,
};
