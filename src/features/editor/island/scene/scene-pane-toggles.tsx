// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Monitor, MonitorOff, Video, VideoOff } from "lucide-react";

import { t } from "../../../../i18n/i18n";
import { IconSwap } from "../../../../primitives/icon-swap/icon-swap";
import { OptionToggle } from "../../../annotations/options/option-toggle";

import { paneShownChoice, type SceneFramingProps } from "./scene-framing";

// Whether the scene in hand shows the camera and the screen, as glyphs at
// the end of the row, before deleting the scene: each pane's glyph while it
// shows, struck through once it is hidden, named for assistive technology
// alone. Turning one off a preset's scene gives it the layout of the pane
// left, and turning it back on returns to the layout it left; a custom
// scene keeps each pane where it was put. The camera's is offered only
// where a camera is in the video to show.

const GLYPHS = {
  camera: { hidden: VideoOff, shown: Video },
  screen: { hidden: MonitorOff, shown: Monitor },
};

function PaneGlyph({
  isShown,
  pane,
}: {
  isShown: boolean;
  pane: "camera" | "screen";
}) {
  const Glyph = GLYPHS[pane][isShown ? "shown" : "hidden"];
  return (
    <span className="relative size-icon">
      <IconSwap swapKey={isShown ? "shown" : "hidden"}>
        <Glyph aria-hidden className="size-icon" />
      </IconSwap>
    </span>
  );
}

export function ScenePaneToggles(props: SceneFramingProps) {
  const toggle = (pane: "camera" | "screen") => {
    const shown = paneShownChoice(props, pane);
    return shown === null ? null : (
      <OptionToggle
        glyph={(isOn) => <PaneGlyph isShown={isOn} pane={pane} />}
        isLabelHidden
        isOn={shown}
        key={pane}
        label={t(
          pane === "camera"
            ? "editor-island-scene-show-camera"
            : "editor-island-scene-show-screen",
        )}
        onChange={(isOn) => {
          props.onPaneShownChange(pane, isOn);
        }}
      />
    );
  };
  return (
    <>
      {toggle("camera")}
      {toggle("screen")}
    </>
  );
}
