// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ReactNode } from "react";

import { cn } from "../../../lib/styling";
import { ProgressBar } from "../../base/progress-bar/progress-bar";
import { Text } from "../../base/text/text";

export type ProgressPanelProps = {
  /** What is happening, in one short phrase. */
  label: string;
  /** Percent complete, or `null` when the work cannot report how far it is. */
  progress: number | null;
  /** Offered at the end of the bar row, typically a single button. */
  action?: ReactNode;
  /** The accessible name of the bar, when the label is too terse to serve. */
  progressLabel?: string;
  /** A line under the bar: an estimate, a shortcut hint. */
  secondary?: ReactNode;
};

/**
 * The Finder copy-dialog layout: the label above, the bar beneath with the
 * action at the end of its row, and the estimate below.
 *
 * Presentational: it neither owns the work nor decides how to describe it, so
 * every phrase, estimate and action arrives from the feature that is waiting.
 */
export function ProgressPanel({
  action,
  label,
  progress,
  progressLabel,
  secondary,
}: ProgressPanelProps) {
  return (
    <div className="flex w-full flex-col gap-tight">
      <Text as="span" className="mb-tight">
        {label}
      </Text>
      {/* The action trails the bar, where Finder and Safari put the stop. It
          is taller than the bar, so its overhang is pulled in and the gaps
          above and below measure from the bar rather than the button. */}
      <div
        className={cn(
          "flex items-center gap-control-inset",
          action != null && "-my-2.25",
        )}
      >
        <ProgressBar
          aria-label={progressLabel ?? label}
          className="flex-1"
          isIndeterminate={progress === null}
          value={progress ?? undefined}
        />
        {action}
      </div>
      {secondary === undefined ? null : (
        <Text as="span" className="tabular-nums" variant="subheadline">
          {secondary}
        </Text>
      )}
    </div>
  );
}
