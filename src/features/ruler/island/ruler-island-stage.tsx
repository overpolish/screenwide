// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useState } from "react";

import { ISLAND } from "../../../primitives/island/island-geometry";
import { StoryNotch } from "../../../storybook/story-notch";

import { RulerIsland, type RulerIslandProps } from "./ruler-island";

import type { RulerIslandState } from "../../../bindings/RulerIslandState";

/** The island at the top of the canvas, under the notch when there is one.
 * The story plays the session, so every tool and toggle can be tried. */
export function RulerIslandStage(props: RulerIslandProps) {
  const [state, setState] = useState(props.state);
  const change = (patch: Partial<RulerIslandState>) => {
    setState((current) => ({ ...current, ...patch }));
  };
  return (
    <div className="relative size-full" dir="ltr">
      <div
        className="absolute inset-x-0"
        style={{ top: props.notch ? 0 : ISLAND.screenGap }}
      >
        <RulerIsland
          {...props}
          onAction={(action) => {
            props.onAction(action);
            if (action === "toggleCenterlines") {
              change({ centerlines: !state.centerlines });
            } else if (action === "toggleCrosshair") {
              change({ crosshair: !state.crosshair });
            }
          }}
          onTolerance={(tolerance) => {
            props.onTolerance(tolerance);
            change({ tolerance });
          }}
          onTool={(tool) => {
            props.onTool(tool);
            change({ tool });
          }}
          state={state}
        />
      </div>
      {props.notch ? <StoryNotch notch={props.notch} /> : null}
    </div>
  );
}
