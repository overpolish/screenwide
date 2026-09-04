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
  const buttonClassName = "size-full";
  const fieldClassName = "size-[70px]";
  const iconClassName = "size-10 [&>svg]:size-full!";

  return (
    <RadioGroup
      aria-label="Recording type"
      className="gap-tight min-w-0 grow items-start justify-center self-stretch"
      isDisabled={isDisabled}
      onChange={(value) => {
        onChange(value as RecordingMode);
      }}
      orientation="horizontal"
      value={mode}
    >
      <IconRadio
        aria-label="Screen"
        buttonClassName={buttonClassName}
        className={fieldClassName}
        icon={<Monitor />}
        iconClassName={iconClassName}
        value="screen"
      />
      <IconRadio
        aria-label="Region"
        buttonClassName={buttonClassName}
        className={fieldClassName}
        icon={<SquareDashed />}
        iconClassName={iconClassName}
        value="region"
      />
      <IconRadio
        aria-label="Window"
        buttonClassName={buttonClassName}
        className={fieldClassName}
        icon={<AppWindowMac />}
        iconClassName={iconClassName}
        value="window"
      />
      <IconRadio
        aria-label="Camera only"
        buttonClassName={buttonClassName}
        className={fieldClassName}
        icon={<Camera />}
        iconClassName={iconClassName}
        value="camera"
      />
      <IconRadio
        aria-label="Audio only"
        buttonClassName={buttonClassName}
        className={fieldClassName}
        icon={<AudioLines />}
        iconClassName={iconClassName}
        value="audio"
      />
    </RadioGroup>
  );
}
