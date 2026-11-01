// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/**
 * What one drag holds on to. The targets and the threshold are in whichever
 * time the drag itself works in: output for a clip on the timeline, source
 * for a trim of the footage. Whether to snap is asked as the drag moves
 * rather than when it begins, so an edge can be pulled off a target part way
 * through the gesture.
 */
export type TimelineSnapGesture = {
  isSnapping: () => boolean;
  /** Where the held drag is snapped to, in output time; null once it is not. */
  showGuide: (outputPosition: number | null) => void;
  targets: number[];
  threshold: number;
};

/** The target within reach of `position`, the closest one when several are. */
export function nearestTimelineSnapTarget(
  { isSnapping, targets, threshold }: TimelineSnapGesture,
  position: number,
): number | null {
  if (!isSnapping()) return null;
  let nearest: number | null = null;
  let distance = threshold;
  for (const target of targets) {
    const candidate = Math.abs(target - position);
    if (candidate <= distance) {
      nearest = target;
      distance = candidate;
    }
  }
  return nearest;
}
