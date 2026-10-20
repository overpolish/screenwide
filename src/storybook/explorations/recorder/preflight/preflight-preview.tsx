// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useState } from "react";
import { RadioButton, RadioField, RadioGroup } from "react-aria-components";

import { Button } from "../../../../components/base/button/button";
import { cn } from "../../../../lib/styling";
import {
  AREA,
  DISPLAY,
  DISPLAYS,
  type TargetId,
  WINDOW,
} from "../shared/recorder-data";
import { TargetStill } from "../shared/recorder-parts";

// What the recording will show. With several displays the stills are the
// choice itself: press the one to record. A window or an area is chosen on
// the screen, so beneath its still is the name and a way to point again.

function DisplayPicker() {
  const [chosen, setChosen] = useState(DISPLAY.id);
  return (
    <RadioGroup
      aria-label="Display"
      className="grid grid-cols-2 gap-control"
      onChange={setChosen}
      value={chosen}
    >
      {DISPLAYS.map(({ id, name }) => (
        <RadioField className="min-w-0" key={id} value={id}>
          <RadioButton className="group flex w-full flex-col gap-tight rounded-panel outline-none">
            <TargetStill
              className={cn(
                "aspect-video overflow-hidden rounded-panel opacity-70 transition-opacity group-data-[hovered]:opacity-100",
                id === chosen &&
                  "opacity-100 ring-2 ring-primary ring-offset-2",
                "group-data-[focus-visible]:ring-2 group-data-[focus-visible]:ring-primary",
              )}
              target="display"
            />
            <span
              className={cn(
                "truncate text-center text-footnote",
                id === chosen ? "text-content-fg" : "text-content-fg-secondary",
              )}
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
    <div className="flex flex-col gap-control">
      <TargetStill
        className="aspect-video overflow-hidden rounded-panel"
        target={target}
      />
      <div className="flex items-center gap-control">
        <span className="flex min-w-0 grow items-baseline gap-tight ps-control-inset">
          <span className="truncate text-body" dir="auto">
            {target === "window" ? `${WINDOW.app}: ${WINDOW.name}` : "Area"}
          </span>
          <span className="shrink-0 text-subheadline text-content-fg-secondary tabular-nums">
            {target === "window" ? WINDOW.size : AREA.size}
          </span>
        </span>
        <Button>Change</Button>
      </div>
    </div>
  );
}

export function PreflightPreview({
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
      <TargetStill
        className="aspect-video overflow-hidden rounded-panel"
        target="camera"
      />
    ) : null;
  }
  if (!canRecordScreen) {
    return (
      <div className="flex aspect-video flex-col items-center justify-center gap-control rounded-panel bg-fill-quaternary p-section text-center">
        <span className="text-body text-content-fg-secondary">
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
      <TargetStill
        className="aspect-video overflow-hidden rounded-panel"
        target="display"
      />
    );
  }
  return <Chosen target={target} />;
}
