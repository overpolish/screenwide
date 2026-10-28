// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../../i18n/i18n";

import type { SceneCorner } from "../../../../bindings/SceneCorner";
import type { ScenePreset } from "../../glyphs/scene-layout-drawing";

// What the scene's layouts and corners are called: a layout tile's name,
// and a corner as assistive technology reads it out.

export const presetName = (preset: ScenePreset) => {
  switch (preset) {
    case "split-two-thirds":
      return t("editor-island-scene-side-by-side");
    case "stacked":
      return t("editor-island-scene-stacked");
    case "picture-in-picture":
      return t("editor-island-scene-picture-in-picture");
    case "camera-only":
      return t("editor-island-scene-camera-only");
    case "screen-only":
      return t("editor-island-scene-screen-only");
  }
};

export const cornerName = (corner: SceneCorner) => {
  switch (corner) {
    case "top-left":
      return t("editor-island-scene-top-left");
    case "top-right":
      return t("editor-island-scene-top-right");
    case "bottom-left":
      return t("editor-island-scene-bottom-left");
    case "bottom-right":
      return t("editor-island-scene-bottom-right");
  }
};
