// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  ArrowUpRight,
  Captions,
  Crop,
  Keyboard,
  LayoutPanelLeft,
  MousePointer2,
  MousePointerClick,
  Proportions,
  ScrollText,
  SquareDashedMousePointer,
} from "lucide-react";
import { Fragment, useState } from "react";

import { IconToggleButton } from "../../../primitives/button/icon-button";
import { Toolbar } from "../../../primitives/toolbar/toolbar";

import type { EditorKind } from "./title-bar";

type Tool = { icon: typeof Crop; label: string };

const GROUPS: Record<EditorKind, Tool[][]> = {
  // A recording without a picture has nothing to put captions on; its words
  // are its transcript.
  audio: [[{ icon: ScrollText, label: "Transcript" }]],
  recording: [
    [
      { icon: MousePointer2, label: "Select" },
      { icon: SquareDashedMousePointer, label: "Marquee" },
    ],
    [
      // The frame's proportions are what the tool changes; scenes arrange
      // the screen and camera, so a layout rather than a clapperboard; and
      // cursor effects show a click, so they do not read as Select.
      { icon: Proportions, label: "Frame" },
      { icon: LayoutPanelLeft, label: "Scene" },
      { icon: Crop, label: "Crop" },
    ],
    [
      { icon: MousePointerClick, label: "Cursor" },
      { icon: Keyboard, label: "Keyboard" },
      { icon: ScrollText, label: "Transcript" },
      { icon: Captions, label: "Captions" },
    ],
  ],
  screenshot: [
    [
      { icon: MousePointer2, label: "Select" },
      { icon: SquareDashedMousePointer, label: "Marquee" },
    ],
    [
      { icon: Proportions, label: "Resize canvas" },
      { icon: Crop, label: "Crop" },
    ],
  ],
};

const ANNOTATIONS = "Annotations";

function GroupRule() {
  return <span aria-hidden className="mx-auto my-1 h-px w-4 bg-divider" />;
}

// Every picture tool that is not an annotation, in groups, beside the
// picture: one tool in hand at a time, in the primary colour. The
// annotations have one button at the end: it opens the ring, and shows the
// last annotation tool taken up, so a second press draws another. Holding on
// the picture opens the same ring where the pointer is.
export function PictureTools({
  isRingOpen = false,
  kind,
}: {
  kind: EditorKind;
  isRingOpen?: boolean;
}) {
  const [chosen, setChosen] = useState(isRingOpen ? ANNOTATIONS : "Select");
  return (
    <Toolbar
      aria-label="Picture tools"
      className="shrink-0"
      orientation="vertical"
    >
      {GROUPS[kind].map((group, index) => (
        <Fragment key={group[0]?.label}>
          {index > 0 ? <GroupRule /> : null}
          {group.map(({ icon: Icon, label }) => (
            <IconToggleButton
              aria-label={label}
              isSelected={chosen === label}
              key={label}
              onChange={() => {
                setChosen(label);
              }}
            >
              <Icon />
            </IconToggleButton>
          ))}
        </Fragment>
      ))}
      {kind === "audio" ? null : (
        <>
          <GroupRule />
          <IconToggleButton
            aria-label="Annotations: Arrow"
            isSelected={chosen === ANNOTATIONS}
            onChange={() => {
              setChosen(ANNOTATIONS);
            }}
          >
            <ArrowUpRight />
          </IconToggleButton>
        </>
      )}
    </Toolbar>
  );
}
