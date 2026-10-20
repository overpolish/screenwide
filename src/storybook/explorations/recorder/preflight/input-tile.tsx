// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ChevronDown } from "lucide-react";
import { Button, ToggleButton } from "react-aria-components";

import { cn } from "../../../../lib/styling";
import { AudioMeter } from "../../../../primitives/audio-meter/audio-meter";

import type { InputSource } from "../shared/recorder-data";

// One input as a tile split in two, as Control Center splits its modules:
// the upper half turns the input on and off, the lower half is one wide
// button that opens the native device menu, and the level runs beneath it so
// a silent microphone shows before recording, not after. An input that
// cannot record says why in the lower half, which becomes the way to fix it.

export type InputAvailability = "no-device" | "not-allowed" | "ready";

const FOCUS_RING =
  "outline-none data-[focus-visible]:ring-2 data-[focus-visible]:ring-primary data-[focus-visible]:ring-inset";

function LowerHalf({
  availability,
  input,
  isOn,
}: {
  availability: InputAvailability;
  input: InputSource;
  isOn: boolean;
}) {
  if (availability === "no-device") {
    return (
      <span className="px-control-inset py-tight text-footnote text-content-fg-tertiary">
        None connected
      </span>
    );
  }
  if (availability === "not-allowed") {
    return (
      <Button
        className={cn(
          "px-control-inset py-tight text-start text-footnote text-primary data-[hovered]:bg-fill-tertiary",
          FOCUS_RING,
        )}
      >
        Allow access
      </Button>
    );
  }
  return (
    <Button
      aria-label={`${input.label} device`}
      className={cn(
        "flex items-center gap-0.5 px-control-inset py-tight text-footnote text-content-fg-secondary data-[hovered]:bg-fill-tertiary",
        FOCUS_RING,
      )}
    >
      <span className="min-w-0 grow truncate text-start" dir="auto">
        {isOn ? input.device : "Off"}
      </span>
      <ChevronDown className="size-3 shrink-0" />
    </Button>
  );
}

export function InputTile({
  availability = "ready",
  input,
  isOn: isChosen,
  onChange,
}: {
  input: InputSource;
  isOn: boolean;
  onChange: (isOn: boolean) => void;
  availability?: InputAvailability;
}) {
  const isReady = availability === "ready";
  const isOn = isReady && isChosen;
  const Icon = isOn ? input.icon : input.offIcon;
  const hasLevel = input.decibels !== undefined;

  return (
    <div
      className={cn(
        "flex min-w-0 flex-col overflow-hidden rounded-panel",
        isOn ? "bg-fill" : "bg-fill-quaternary",
      )}
    >
      <ToggleButton
        className={cn(
          "flex flex-col items-start gap-tight px-control-inset pt-control-inset pb-tight text-body data-[hovered]:bg-fill-tertiary",
          FOCUS_RING,
        )}
        isDisabled={!isReady}
        isSelected={isOn}
        onChange={onChange}
      >
        <Icon
          className={cn(
            "size-icon",
            isOn ? "text-content-fg" : "text-content-fg-tertiary",
          )}
        />
        <span className={cn(!isOn && "text-content-fg-secondary")}>
          {input.label}
        </span>
      </ToggleButton>
      <LowerHalf availability={availability} input={input} isOn={isOn} />
      <span
        aria-hidden={!hasLevel}
        // The meter's SVG is inline; as a block it takes no line-box height.
        className={cn(
          "flex px-control-inset pb-control-inset [&_svg]:block",
          !hasLevel && "invisible",
        )}
      >
        <AudioMeter
          aria-label={`${input.label} level`}
          decibels={input.decibels ?? -90}
          disabled={!isOn}
          hidePeakTick
          hideTicks
          length="100%"
          thickness={3}
        />
      </span>
    </div>
  );
}
