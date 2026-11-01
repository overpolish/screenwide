// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../i18n/i18n";
import { Alert } from "../../../primitives/alert/alert";

/** Over a scene, which may crop, pad, round and turn the screen its own way,
 * crop the camera, and round the camera and have it follow the face or not,
 * what an island sets for those is that scene's, while everything else on
 * it holds for the whole recording; this says so at the island's foot. It
 * takes the section's width and adds none of its own, so the lines above it
 * set the island's width and the words wrap to it. */
export function SceneOnlyNote({
  layer,
}: {
  layer: "camera" | "crop" | "screen";
}) {
  return (
    <Alert className="w-0 min-w-full px-inset-sm" role="status">
      {layer === "camera"
        ? t("editor-island-camera-in-scene")
        : layer === "crop"
          ? t("editor-island-crop-in-scene")
          : t("editor-island-screen-in-scene")}
    </Alert>
  );
}
