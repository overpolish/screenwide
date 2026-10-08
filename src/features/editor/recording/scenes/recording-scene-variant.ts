// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { SceneBubbleSize } from "../../../../bindings/SceneBubbleSize";
import type { SceneCameraSize } from "../../../../bindings/SceneCameraSize";
import type { SceneCorner } from "../../../../bindings/SceneCorner";
import type { SceneVariant } from "../../../../bindings/SceneVariant";

export type { SceneBubbleSize, SceneCameraSize, SceneCorner, SceneVariant };

export const DEFAULT_CAMERA_SIZE: SceneCameraSize = "third";
export const DEFAULT_CORNER: SceneCorner = "bottom-right";
export const DEFAULT_BUBBLE_SIZE: SceneBubbleSize = "large";
