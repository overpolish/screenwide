// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ChevronsUpDown } from "lucide-react";
import { Button } from "react-aria-components";

import { cn, elementFocusVisible, focusStyles } from "../../../lib/styling";
import { AudioMeter } from "../../../primitives/audio-meter/audio-meter";
import { IconToggleButton } from "../../../primitives/button/icon-button";

import type { InputSource } from "./recorder-data";
import type { InputAvailability } from "./recorder-inputs";

// The inputs side by side as three tiles, Control Center's way, to set
// against the rows: each tile its switch and name, the device under them as
// a small pop-up whose list opens in the native panel, and the level along
// its foot. Denser than the rows and read across rather than down; the
// device name has a third of the width, so a long one is cut short.

const DEVICE =
  "flex h-part-height min-w-0 cursor-default items-center gap-1 rounded-part px-1 text-caption outline-none data-[hovered]:bg-fill";

function Device({
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
      <span className="flex h-part-height items-center px-1 text-caption text-fg-tertiary">
        None connected
      </span>
    );
  }
  if (availability === "not-allowed") {
    return (
      <Button
        className={cn(DEVICE, "text-fg", focusStyles, elementFocusVisible)}
      >
        Allow access
      </Button>
    );
  }
  return (
    <Button
      aria-label={`${input.label} device`}
      className={cn(
        DEVICE,
        "text-fg-secondary",
        focusStyles,
        elementFocusVisible,
      )}
    >
      <span className="min-w-0 truncate" dir="auto">
        {isOn ? input.device : "Off"}
      </span>
      <ChevronsUpDown className="size-3 shrink-0 text-fg-tertiary" />
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
    <div className="flex min-w-0 flex-col gap-1 rounded-panel bg-fill p-inset-sm">
      <span className="flex min-w-0 items-center gap-inline">
        <IconToggleButton
          aria-label={input.label}
          isDisabled={!isReady}
          isSelected={isOn}
          onChange={onChange}
        >
          <Icon />
        </IconToggleButton>
        <span
          className={cn(
            "min-w-0 truncate text-label",
            isOn ? "text-fg" : "text-fg-secondary",
          )}
        >
          {input.label}
        </span>
      </span>
      <Device availability={availability} input={input} isOn={isOn} />
      <span
        aria-hidden={!hasLevel}
        className={cn("flex px-1 [&_svg]:block", !hasLevel && "invisible")}
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
