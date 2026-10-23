// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { MonitorDetails } from "../../../bindings/MonitorDetails";
import type { Region } from "../../../bindings/Region";

/** The displays `region` lies on, in their arrangement. The region is kept in
 * `owner`'s logical points and may reach past it, so it is laid on the
 * desktop as the overlay lays the displays out: each at its own logical
 * position. One that lies on none is said to lie on its owner. */
export function regionDisplays(
  region: Region,
  owner: MonitorDetails,
  displays: MonitorDetails[],
): MonitorDetails[] {
  const left = owner.position.x + region.position.x;
  const top = owner.position.y + region.position.y;
  const right = left + region.size.width;
  const bottom = top + region.size.height;
  const covered = displays.filter(
    ({ position, size }) =>
      left < position.x + size.width &&
      right > position.x &&
      top < position.y + size.height &&
      bottom > position.y,
  );
  return covered.length > 0 ? covered : [owner];
}
