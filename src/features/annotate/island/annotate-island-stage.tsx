// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useState } from "react";

import { ISLAND } from "../../../primitives/island/island-geometry";
import { StoryNotch } from "../../../storybook/story-notch";

import { AnnotateIsland, type AnnotateIslandProps } from "./annotate-island";

/** The island at the top of the canvas, under the notch when there is one.
 * Choices are kept by the story, so every one can be tried. */
export function AnnotateIslandStage(props: AnnotateIslandProps) {
  const [chosen, setChosen] = useState(props.settings);
  return (
    <div className="relative size-full" dir="ltr">
      <div
        className="absolute inset-x-0"
        style={{ top: props.notch ? 0 : ISLAND.screenGap }}
      >
        <AnnotateIsland
          {...props}
          onChange={(patch, immediate) => {
            props.onChange(patch, immediate);
            setChosen((current) => ({ ...current, ...patch }));
          }}
          settings={chosen}
        />
      </div>
      {props.notch ? <StoryNotch notch={props.notch} /> : null}
    </div>
  );
}
