// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ComponentProps } from "react";

import { Swatch } from "../../../../../primitives/swatch/swatch";

import { Background } from "./background";
import {
  backgroundTileStyle,
  useBackgroundThumbnail,
} from "./background-preview";

/** The swatch's edge in logical px: a colour swatch's, as the colour grids
 * draw them, so a background is picked from tiles the size of any other
 * colour. */
const SWATCH_SIZE = 20;

// A background offered in the picker, as a swatch of what it paints. The
// swatch is the background rendered by the code that renders it for the
// canvas, so a mesh and a picture are shown rather than approximated; the
// CSS likeness stands in until the render arrives, and in a story, where
// there is no renderer to ask. Without a background it is an empty swatch,
// as the Custom tile is while there is nothing to keep, and stays the same
// control as it fills and empties.
export function BackgroundSwatch({
  background,
  ...props
}: Omit<ComponentProps<typeof Swatch>, "color" | "style"> & {
  background: Background | null;
}) {
  const thumbnail = useBackgroundThumbnail(
    background ?? undefined,
    SWATCH_SIZE,
  );
  const paint = () => {
    if (thumbnail)
      return {
        backgroundImage: `url("${thumbnail}")`,
        backgroundPosition: "center",
        backgroundSize: "cover",
      };
    return background ? backgroundTileStyle(background) : undefined;
  };
  return <Swatch {...props} size="sm" style={paint()} />;
}
