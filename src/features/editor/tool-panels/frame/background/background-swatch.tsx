// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ComponentProps } from "react";

import { Swatch } from "../../../../../primitives/swatch/swatch";

import { Background } from "./background";
import {
  backgroundTileStyle,
  useBackgroundThumbnail,
} from "./background-preview";

/** The swatch's edge in logical px: a control's height. */
const SWATCH_SIZE = 28;

// A background offered in the picker, as a swatch of what it paints. The
// swatch is the background rendered by the code that renders it for the
// canvas, so a mesh and a picture are shown rather than approximated; the
// CSS likeness stands in until the render arrives, and in a story, where
// there is no renderer to ask.
export function BackgroundSwatch({
  background,
  ...props
}: Omit<ComponentProps<typeof Swatch>, "color" | "style"> & {
  background: Background;
}) {
  const thumbnail = useBackgroundThumbnail(background, SWATCH_SIZE);
  return (
    <Swatch
      {...props}
      style={
        thumbnail
          ? {
              backgroundImage: `url("${thumbnail}")`,
              backgroundPosition: "center",
              backgroundSize: "cover",
            }
          : backgroundTileStyle(background)
      }
    />
  );
}
