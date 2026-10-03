// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { LaneBox } from "./timeline-lane-band";

/** The band a press on empty lane draws, over the lane it is choosing in. */
export function TimelineLaneBandBox({ box }: { box: LaneBox }) {
  return (
    <div
      aria-hidden
      className="pointer-events-none absolute bg-primary/15 inset-ring inset-ring-primary"
      style={{
        height: box.bottom - box.top,
        left: box.left,
        top: box.top,
        width: box.right - box.left,
      }}
    />
  );
}
