// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type HTMLMotionProps, motion } from "motion/react";

import { type IslandPlace } from "./island-place";

// What a host carries an island in, to wherever it stands it in its window.
// The parts inside an island come, go and change with Motion's layout
// animations (`IslandParts`), which measure where each part is on the page
// at every render. Moved by a transform Motion knows nothing of, the whole
// island moving reads to each part as a move of its own, and it glides
// after the island from where it was, out of the black and clipped (#40,
// #89). Moved by `place` (`useIslandPlace`), which Motion takes back out of
// what it measures, the island's own move is never seen, and every part
// goes with it on the same frame.

/** The only way a host moves an island: by `place`, never by a transform
 * of its own. */
export function IslandHolder({
  place,
  style,
  ...props
}: HTMLMotionProps<"div"> & { place: IslandPlace }) {
  return <motion.div {...props} style={{ ...style, x: place.x, y: place.y }} />;
}
