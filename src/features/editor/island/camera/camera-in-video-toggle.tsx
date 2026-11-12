// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../../i18n/i18n";
import { OptionToggle } from "../../../annotations/options/option-toggle";

import { CameraInVideoGlyph } from "./camera-glyphs";

// Whether the camera is drawn into the video or saved as a file of its own,
// at the end of the Camera island's row, as Show shortcuts is on the
// Keyboard island's. It is the Export island's choice too ("Save the camera
// as its own file"), so the two always agree.

/** Draws the camera into the video, or saves it beside the video. */
export function CameraInVideoToggle({
  isInVideo,
  onChange,
}: {
  isInVideo: boolean;
  onChange: (isInVideo: boolean) => void;
}) {
  return (
    <OptionToggle
      glyph={(isOn) => <CameraInVideoGlyph isInVideo={isOn} />}
      isLabelHidden
      isOn={isInVideo}
      label={t("editor-island-camera-in-video")}
      onChange={onChange}
    />
  );
}
