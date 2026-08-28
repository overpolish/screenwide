// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  AppWindowMac,
  AudioLines,
  Camera,
  Monitor,
  SquareDashed,
} from "lucide-react";

import { RadioGroup } from "../../../components/base/radio-group/radio-group";
import { RecordingMode } from "../../recording-sources/types";

import { IconRadio } from "./icon-radio";

export function RecordingModePicker({
  isDisabled,
  mode,
  onChange,
}: {
  isDisabled: boolean;
  mode: RecordingMode;
  onChange: (mode: RecordingMode) => void;
}) {
  return (
    <RadioGroup
      aria-label="Recording type"
      className="min-w-0 grow self-stretch"
      isDisabled={isDisabled}
      onChange={(value) => {
        onChange(value as RecordingMode);
      }}
      orientation="horizontal"
      value={mode}
    >
      <IconRadio
        aria-label="Screen"
        icon={<Monitor />}
        subtext="Screen"
        value="screen"
      />
      <IconRadio
        aria-label="Region"
        icon={<SquareDashed />}
        subtext="Region"
        value="region"
      />
      <IconRadio
        aria-label="Window"
        icon={<AppWindowMac />}
        subtext="Window"
        value="window"
      />
      <IconRadio
        aria-label="Camera only"
        icon={<Camera />}
        subtext="Camera"
        value="camera"
      />
      <IconRadio
        aria-label="Audio only"
        icon={<AudioLines />}
        subtext="Audio"
        value="audio"
      />
    </RadioGroup>
  );
}
