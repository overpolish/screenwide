// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { SlidersHorizontal } from "lucide-react";
import { useState } from "react";

import { Button } from "../../../../components/base/button/button";
import { IconButton } from "../../../../components/base/button/icon-button";
import {
  Keyboard,
  Shortcut,
} from "../../../../components/base/keyboard/keyboard";
import { Switch } from "../../../../components/base/switch/switch";
import { ControlRow } from "../../../../components/shared/control-row/control-row";
import { cn } from "../../../../lib/styling";
import {
  DEFAULT_INPUTS,
  type InputId,
  INPUTS,
  type TargetId,
} from "../shared/recorder-data";

import { type InputAvailability, InputTile } from "./input-tile";
import { PreflightPreview } from "./preflight-preview";
import { TargetPicker } from "./target-picker";

// The panel's content only: the window draws its own material and corners.
// It answers "what will this record?" before anything starts: the preview is
// live, and each input shows its device and level. Choosing Window or Area
// hands over to the screen to point at one, unless one was chosen before:
// the last window or area comes back, with Change to point again. It is
// put away like a popover, by Escape or a click elsewhere, so it needs no
// close button.

export type PreflightPanelProps = {
  /** Inputs that cannot record; every input is ready unless listed. */
  availability?: Partial<Record<InputId, InputAvailability>>;
  canRecordScreen?: boolean;
  /** Connected displays; more than one turns the preview into a picker. */
  displayCount?: number;
  initialInputs?: Partial<Record<InputId, boolean>>;
  initialTarget?: TargetId;
  isOptionsOpen?: boolean;
  isReplayOn?: boolean;
};

export function PreflightPanel({
  availability = {},
  canRecordScreen = true,
  displayCount = 1,
  initialInputs = {},
  initialTarget = "display",
  isOptionsOpen = false,
  isReplayOn: initialReplay = false,
}: PreflightPanelProps) {
  const [target, setTarget] = useState<TargetId>(initialTarget);
  const [inputs, setInputs] = useState({ ...DEFAULT_INPUTS, ...initialInputs });
  const [isReplayOn, setIsReplayOn] = useState(initialReplay);
  const hasScreen = target !== "none";
  const canRecord =
    (canRecordScreen || !hasScreen) &&
    (hasScreen || inputs.camera || inputs.microphone || inputs.sound);

  return (
    <div className="flex w-80 flex-col gap-section p-section text-content-fg">
      <TargetPicker onChange={setTarget} value={target} />
      <PreflightPreview
        canRecordScreen={canRecordScreen}
        displayCount={displayCount}
        isCameraOn={inputs.camera && availability.camera === undefined}
        target={target}
      />
      <div className="grid grid-cols-3 gap-control">
        {INPUTS.map((input) => (
          <InputTile
            availability={availability[input.id]}
            input={input}
            isOn={inputs[input.id]}
            key={input.id}
            onChange={(isOn) => {
              setInputs((current) => ({ ...current, [input.id]: isOn }));
            }}
          />
        ))}
      </div>
      <ControlRow description="Keeps the last 30 seconds" title="Replay buffer">
        {(controlProps) => (
          <div className="flex items-center gap-control">
            {isReplayOn ? <Button>Save</Button> : null}
            <Switch
              {...controlProps}
              isSelected={isReplayOn}
              onChange={setIsReplayOn}
            />
          </div>
        )}
      </ControlRow>
      <div className="flex gap-control">
        {hasScreen ? (
          <Button isDisabled={!canRecordScreen}>Screenshot</Button>
        ) : null}
        <Button
          className="grow justify-between"
          color="primary"
          isDisabled={!canRecord}
        >
          Record
          <Shortcut className="opacity-70">
            <Keyboard variant="plain">Shift</Keyboard>
            <Keyboard variant="plain">Command</Keyboard>
            <Keyboard variant="plain">R</Keyboard>
          </Shortcut>
        </Button>
        <IconButton
          aria-expanded={isOptionsOpen}
          aria-label="Options"
          className={cn(isOptionsOpen && "bg-fill")}
        >
          <SlidersHorizontal />
        </IconButton>
      </div>
    </div>
  );
}
