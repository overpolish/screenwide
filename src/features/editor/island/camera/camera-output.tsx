// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Video } from "lucide-react";

import { t } from "../../../../i18n/i18n";
import { Alert } from "../../../../primitives/alert/alert";
import { IslandRowTitle } from "../../../../primitives/island/row-title/island-row-title";

import { CameraInVideoToggle } from "./camera-in-video-toggle";

import type { EditorIslandContent } from "../editor-island";

// The Camera island while the camera is saved as a file of its own. The
// camera is then not in the picture, so the island has nothing of its look
// to offer: only the toggle that draws it back in, and a note saying where
// it goes.

export function separateCameraIsland({
  onInVideoChange,
}: {
  onInVideoChange: (isInVideo: boolean) => void;
}): EditorIslandContent {
  return {
    actions: (
      <CameraInVideoToggle isInVideo={false} onChange={onInVideoChange} />
    ),
    key: "camera",
    section: (
      // The note is all the section holds, so it sets the island's width
      // rather than wrapping to the row's.
      <Alert className="px-inset-sm" role="status">
        {t("editor-island-camera-separate")}
      </Alert>
    ),
    title: (
      <IslandRowTitle
        glyph={<Video aria-hidden className="size-icon" />}
        title={t("editor-island-camera")}
      />
    ),
  };
}
