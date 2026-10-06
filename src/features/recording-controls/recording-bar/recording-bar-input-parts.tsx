// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Lock, TriangleAlert } from "lucide-react";
import { ReactNode } from "react";

import { NativeTooltipTrigger } from "../../../components/shared/native-tooltip/native-tooltip-trigger";
import { AudioMeter } from "../../audio-inputs/audio-meter";

/** An input's corner badge: a warning when it cannot be recorded, which wins
 * over the lock of a missing permission. `warning` is set only while the
 * input is on, since an input that is off is never recorded. */
export const inputBadge = (
  warning: string | undefined,
  isLocked: boolean,
): ReactNode =>
  warning !== undefined ? (
    <TriangleAlert className="text-warning" />
  ) : isLocked ? (
    <Lock className="text-content-fg-secondary" />
  ) : null;

/** Hangs a warning's reason off the input's toggle. The bar's window is only
 * as tall as its controls, so the tooltip is drawn in a window of its own
 * rather than in the page. */
export const warnedToggle = (warning: string | undefined) =>
  warning === undefined
    ? undefined
    : (toggle: ReactNode) => (
        <NativeTooltipTrigger tooltip={warning}>{toggle}</NativeTooltipTrigger>
      );

/**
 * An audio input keeps its glyph and shows its level as a thin bar under the
 * device name, drawn disabled while the input is off so the control keeps one
 * shape in both states. A vertical meter in the glyph's place read as a
 * divider whenever the input was quiet.
 */
export const inputMeter = (isOn: boolean, decibels: number, peak: number) => (
  <AudioMeter
    decibels={decibels}
    disabled={!isOn}
    height={3}
    hidePeakTick
    hideTicks
    peak={peak}
    radius={1.5}
    width="100%"
  />
);
