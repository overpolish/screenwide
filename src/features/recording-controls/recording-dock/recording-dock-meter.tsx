// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { AudioMeter } from "../../../components/base/audio-meter/audio-meter";

/** An input's level in the dock: a hairline standing beside it. */
export function RecordingDockMeter({
  "aria-label": label,
  decibels,
  disabled,
}: {
  "aria-label": string;
  decibels: number;
  disabled?: boolean;
}) {
  return (
    <AudioMeter
      aria-label={label}
      decibels={decibels}
      disabled={disabled}
      hidePeakTick
      hideTicks
      length={16}
      orientation="vertical"
      thickness={2}
    />
  );
}
