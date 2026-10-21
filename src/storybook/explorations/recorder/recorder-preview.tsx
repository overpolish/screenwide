// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useState } from "react";
import { RadioButton, RadioField, RadioGroup } from "react-aria-components";

import { cn, focusStyles } from "../../../lib/styling";
import { Button } from "../../../primitives/button/button";
import { TargetStill } from "../shared/desktop";

import {
  AREA,
  DISPLAY,
  DISPLAYS,
  type TargetId,
  WINDOW,
} from "./recorder-data";

// What the recording will show. With several displays the stills are the
// choice itself, the chosen one outlined in the primary colour as anything
// chosen is. A window or an area is chosen on the screen, so under its still
// is its name and a way to point again.

const STILL = "aspect-video overflow-hidden rounded-panel";

function DisplayPicker() {
  const [chosen, setChosen] = useState(DISPLAY.id);
  return (
    <RadioGroup
      aria-label="Display"
      className="grid grid-cols-2 gap-inline"
      onChange={setChosen}
      orientation="horizontal"
      value={chosen}
    >
      {DISPLAYS.map(({ id, name }) => (
        <RadioField className="min-w-0" key={id} value={id}>
          <RadioButton className="group flex w-full cursor-default flex-col gap-inline rounded-panel outline-none">
            <TargetStill
              className={cn(
                STILL,
                "outline-offset-2 transition-opacity",
                "opacity-60 group-data-[hovered]:opacity-100",
                "group-data-[selected]:opacity-100 group-data-[selected]:outline-2 group-data-[selected]:outline-primary group-data-[selected]:outline-solid",
                focusStyles,
                "group-data-[focus-visible]:ring-3",
              )}
              target="display"
            />
            <span
              className="truncate text-center text-caption text-fg-secondary group-data-[selected]:text-fg"
              dir="auto"
            >
              {name}
            </span>
          </RadioButton>
        </RadioField>
      ))}
    </RadioGroup>
  );
}

function Chosen({ target }: { target: "area" | "window" }) {
  return (
    <div className="flex flex-col gap-inline">
      <TargetStill className={STILL} target={target} />
      <div className="flex items-center gap-inline">
        <span className="flex min-w-0 grow items-baseline gap-inline">
          <span className="truncate text-body" dir="auto">
            {target === "window" ? `${WINDOW.app}: ${WINDOW.name}` : "Area"}
          </span>
          <span className="shrink-0 text-caption text-fg-tertiary tabular-nums">
            {target === "window" ? WINDOW.size : AREA.size}
          </span>
        </span>
        <Button>Change</Button>
      </div>
    </div>
  );
}

export function RecorderPreview({
  canRecordScreen,
  displayCount,
  isCameraOn,
  target,
}: {
  canRecordScreen: boolean;
  displayCount: number;
  isCameraOn: boolean;
  target: TargetId;
}) {
  if (target === "none") {
    // Camera only shows its feed; sound only has nothing to show.
    return isCameraOn ? (
      <TargetStill className={STILL} target="camera" />
    ) : null;
  }
  if (!canRecordScreen) {
    return (
      <div className="flex aspect-video flex-col items-center justify-center gap-separate rounded-panel bg-recess p-inset text-center">
        <span className="text-body text-fg-secondary">
          Screenwide needs permission to record your screen.
        </span>
        <Button>Open System Settings</Button>
      </div>
    );
  }
  if (target === "display") {
    return displayCount > 1 ? (
      <DisplayPicker />
    ) : (
      <TargetStill className={STILL} target="display" />
    );
  }
  return <Chosen target={target} />;
}
