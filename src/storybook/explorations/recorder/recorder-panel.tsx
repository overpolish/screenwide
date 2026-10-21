// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { SlidersHorizontal } from "lucide-react";
import { useState } from "react";

import { Button } from "../../../primitives/button/button";
import { IconButton } from "../../../primitives/button/icon-button";
import { Keyboard, Shortcut } from "../../../primitives/keyboard/keyboard";
import { PillGroup } from "../../../primitives/pill-group/pill-group";
import { Switch } from "../../../primitives/switch/switch";
import { SettingRow } from "../shared/setting-row";

import { InputTile } from "./input-tile";
import {
  DEFAULT_INPUTS,
  type InputId,
  INPUTS,
  type TargetId,
  TARGETS,
} from "./recorder-data";
import { type InputAvailability, InputRow } from "./recorder-inputs";
import { RecorderPreview } from "./recorder-preview";

// The panel's content only: the window draws its own material and corners.
// It answers "what will this record?" before anything starts: what to
// record as one segmented choice, a live preview of it, and each input with
// its device and level. Choosing Window or Area hands over to the screen to
// point at one, unless one was chosen before: the last window or area comes
// back, with Change to point again. It is put away like a popover, by
// Escape or a click elsewhere, so it needs no close button. Record is the
// panel's one primary action, with its shortcut on it.

export type RecorderPanelProps = {
  /** Inputs that cannot record; every input is ready unless listed. */
  availability?: Partial<Record<InputId, InputAvailability>>;
  canRecordScreen?: boolean;
  /** Connected displays; more than one turns the preview into a picker. */
  displayCount?: number;
  initialInputs?: Partial<Record<InputId, boolean>>;
  initialTarget?: TargetId;
  /** The inputs one to a row, or side by side as tiles, to compare. */
  inputLayout?: "rows" | "tiles";
  isOptionsOpen?: boolean;
  isReplayOn?: boolean;
};

export function RecorderPanel({
  availability = {},
  canRecordScreen = true,
  displayCount = 1,
  initialInputs = {},
  initialTarget = "display",
  inputLayout = "rows",
  isOptionsOpen = false,
  isReplayOn: initialReplay = false,
}: RecorderPanelProps) {
  const [target, setTarget] = useState<TargetId>(initialTarget);
  const [inputs, setInputs] = useState({ ...DEFAULT_INPUTS, ...initialInputs });
  const [isReplayOn, setIsReplayOn] = useState(initialReplay);
  const hasScreen = target !== "none";
  const canRecord =
    (canRecordScreen || !hasScreen) &&
    (hasScreen || inputs.camera || inputs.microphone || inputs.sound);

  return (
    <div className="flex w-80 flex-col gap-separate p-inset text-fg">
      <PillGroup
        aria-label="Record"
        className="w-full"
        itemClassName="flex-1"
        items={TARGETS}
        onSelectionChange={(id) => {
          setTarget(id as TargetId);
        }}
        selected={target}
      />
      <RecorderPreview
        canRecordScreen={canRecordScreen}
        displayCount={displayCount}
        isCameraOn={inputs.camera && availability.camera === undefined}
        target={target}
      />
      <div
        className={
          inputLayout === "tiles"
            ? "grid grid-cols-3 gap-inline"
            : "flex flex-col gap-inline"
        }
      >
        {INPUTS.map((input) => {
          const Input = inputLayout === "tiles" ? InputTile : InputRow;
          return (
            <Input
              availability={availability[input.id]}
              input={input}
              isOn={inputs[input.id]}
              key={input.id}
              onChange={(isOn) => {
                setInputs((current) => ({ ...current, [input.id]: isOn }));
              }}
            />
          );
        })}
      </div>
      <SettingRow description="Keeps the last 30 seconds" title="Replay buffer">
        {(labelledBy) => (
          <>
            {isReplayOn ? <Button>Save</Button> : null}
            <Switch
              aria-labelledby={labelledBy}
              isSelected={isReplayOn}
              onChange={setIsReplayOn}
            />
          </>
        )}
      </SettingRow>
      <div className="flex gap-inline">
        {hasScreen ? (
          <Button isDisabled={!canRecordScreen}>Screenshot</Button>
        ) : null}
        <Button
          className="grow justify-between"
          color="primary"
          isDisabled={!canRecord}
        >
          Record
          <Shortcut className="text-on-primary opacity-70">
            <Keyboard className="text-on-primary" variant="plain">
              Shift
            </Keyboard>
            <Keyboard className="text-on-primary" variant="plain">
              Command
            </Keyboard>
            <Keyboard className="text-on-primary" variant="plain">
              R
            </Keyboard>
          </Shortcut>
        </Button>
        <IconButton
          aria-expanded={isOptionsOpen}
          aria-label="Options"
          className={isOptionsOpen ? "bg-fill" : undefined}
          variant="ghost"
        >
          <SlidersHorizontal />
        </IconButton>
      </div>
    </div>
  );
}
