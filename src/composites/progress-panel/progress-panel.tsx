// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ReactNode } from "react";

import { ProgressBar } from "../../primitives/progress-bar/progress-bar";
import { Text } from "../../primitives/text/text";

// The label and status share a line above the bar; actions sit in a row
// below, at the end. The feature that is waiting supplies every phrase.

type ProgressPanelProps = {
  label: string;
  /** Percent complete; `null` while it cannot tell. */
  progress: number | null;
  actions?: ReactNode;
  /** The bar's accessible name; `label` when left out. */
  progressLabel?: string;
  status?: ReactNode;
};

export function ProgressPanel({
  actions,
  label,
  progress,
  progressLabel,
  status,
}: ProgressPanelProps) {
  return (
    <div className="flex w-full flex-col gap-inline">
      <div className="flex items-center justify-between gap-separate">
        <Text as="span" variant="label">
          {label}
        </Text>
        {status === undefined ? null : (
          <Text as="span" className="tabular-nums" variant="caption">
            {status}
          </Text>
        )}
      </div>
      <ProgressBar
        aria-label={progressLabel ?? label}
        isIndeterminate={progress === null}
        value={progress ?? undefined}
      />
      {actions === undefined ? null : (
        <div className="mt-inline flex items-center justify-end gap-inline">
          {actions}
        </div>
      )}
    </div>
  );
}
