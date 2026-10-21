// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { AudioMeter } from "../../../primitives/audio-meter/audio-meter";
import { Button } from "../../../primitives/button/button";
import { IconToggleButton } from "../../../primitives/button/icon-button";
import { ListBoxItem } from "../../../primitives/listbox-item/listbox-item";
import { Select } from "../../../primitives/select/select";

import type { InputSource } from "./recorder-data";

// One input to a row: its switch, the device it records from, and its level
// beside it, so a silent microphone shows before recording, not after. The
// switch is the input's own glyph, filled in the primary colour while it is
// on and struck through while it is off. The device's list opens in the
// native panel. An input that cannot record says why where its device would
// be, and that becomes the way to fix it.

export type InputAvailability = "no-device" | "not-allowed" | "ready";

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
      <span className="flex h-control-height grow items-center px-inset-sm text-body text-fg-tertiary">
        No {input.label.toLowerCase()} connected
      </span>
    );
  }
  if (availability === "not-allowed") {
    return (
      <span className="flex grow items-center justify-between gap-inline ps-inset-sm">
        <span className="text-body text-fg-tertiary">Not allowed</span>
        <Button>Allow access</Button>
      </span>
    );
  }
  return (
    <Select
      aria-label={`${input.label} device`}
      className="min-w-0 grow"
      defaultValue="device"
      isDisabled={!isOn}
      items={[{ id: "device", label: input.device }]}
    >
      {(item: { id: string; label: string }) => (
        <ListBoxItem id={item.id}>{item.label}</ListBoxItem>
      )}
    </Select>
  );
}

export function InputRow({
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
    <div className="flex items-center gap-inline">
      <IconToggleButton
        aria-label={input.label}
        isDisabled={!isReady}
        isSelected={isOn}
        onChange={onChange}
      >
        <Icon />
      </IconToggleButton>
      <Device availability={availability} input={input} isOn={isOn} />
      {/* The meter's SVG is inline; as a block it takes no line-box
          height. The camera has no level, so it keeps the space. */}
      <span
        aria-hidden={!hasLevel}
        className={hasLevel ? "flex [&_svg]:block" : "invisible flex"}
      >
        <AudioMeter
          aria-label={`${input.label} level`}
          decibels={input.decibels ?? -90}
          disabled={!isOn}
          hidePeakTick
          hideTicks
          length={28}
          orientation="vertical"
          thickness={4}
        />
      </span>
    </div>
  );
}
