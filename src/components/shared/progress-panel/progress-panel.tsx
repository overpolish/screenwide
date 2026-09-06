// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ReactNode } from "react";

import { tv } from "../../../lib/variants";
import { CircularProgress } from "../../base/circular-progress/circular-progress";
import { Text } from "../../base/text/text";

const progressPanelVariants = tv({
  defaultVariants: { orientation: "column" },
  slots: {
    base: "gap-section flex",
    secondary: "tabular-nums",
    text: "gap-tight flex flex-col",
  },
  variants: {
    orientation: {
      column: { base: "flex-col items-center", text: "items-center" },
      row: { base: "items-center" },
    },
  },
});

export type ProgressPanelProps = {
  /** What is happening, in one short phrase. */
  label: string;
  /** Percent complete, or `null` when the work cannot report how far it is. */
  progress: number | null;
  /** Offered below the text, typically a single button. */
  action?: ReactNode;
  orientation?: "column" | "row";
  /** The accessible name of the ring, when the label is too terse to serve. */
  progressLabel?: string;
  /** A second line under the label: an estimate, a shortcut hint. */
  secondary?: ReactNode;
  /** The large ring shows its own percentage while determinate. */
  size?: "default" | "large";
};

/**
 * A ring with what it is measuring beside or beneath it.
 *
 * Presentational: it neither owns the work nor decides how to describe it, so
 * every phrase, estimate and action arrives from the feature that is waiting.
 */
export function ProgressPanel({
  action,
  label,
  orientation,
  progress,
  progressLabel,
  secondary,
  size = "default",
}: ProgressPanelProps) {
  const styles = progressPanelVariants({ orientation });

  return (
    <div className={styles.base()}>
      <CircularProgress
        aria-label={progressLabel ?? label}
        isIndeterminate={progress === null}
        size={size}
        value={progress ?? undefined}
      />
      <div className={styles.text()}>
        <Text as="span">{label}</Text>
        {secondary === undefined ? null : (
          <Text as="span" className={styles.secondary()} variant="help">
            {secondary}
          </Text>
        )}
      </div>
      {action}
    </div>
  );
}
