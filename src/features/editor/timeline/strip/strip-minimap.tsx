// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Minimap, type MinimapProps } from "../minimap/minimap";

import { type StripViewNow, useStripViewNow } from "./strip-view-now";

/** The strip's minimap, its window following the view as the strip pans,
 * drawn again on its own without the strip. */
export function StripMinimap({
  now,
  ...props
}: Omit<Extract<MinimapProps, { isPicture?: false }>, "view"> & {
  now: StripViewNow;
}) {
  const view = useStripViewNow(now);
  return <Minimap {...props} view={view} />;
}
