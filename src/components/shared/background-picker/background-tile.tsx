// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { isTauri } from "@tauri-apps/api/core";
import { MouseEventHandler, ReactNode } from "react";
import { ToggleButton } from "react-aria-components";

import { cn, elementFocusVisible, focusStyles } from "../../../lib/styling";

import { Background } from "./background";
import {
  backgroundTileStyle,
  useBackgroundThumbnail,
} from "./background-preview";

/** The swatch's edge in logical px, matching `size-icon-large`. */
const TILE_SIZE = 28;

/**
 * One background offered in the grid: a swatch of what it paints, or a glyph
 * for the two that stand for an action rather than a colour.
 *
 * A swatch is the background rendered by the code that renders it for the
 * canvas, so a mesh and a picture are shown rather than approximated. That
 * holds for a wallpaper too: the small copy the system keeps is HEIC, which
 * the webview has no decoder for, so it is handed to the renderer as the file
 * to draw from rather than to the tile as a URL. The CSS likeness stands in
 * until the picture arrives, and in a story, where there is no renderer to
 * ask.
 *
 * The chosen tile carries the accent ring the app draws selection with, held
 * off the tile by an offset so a dark background keeps its own edge.
 */
export function BackgroundTile({
  ariaLabel,
  background,
  children,
  id,
  isDisabled,
  isSelected,
  onContextMenu,
  onPress,
  thumbnailPath,
}: {
  ariaLabel: string;
  id: string;
  onPress: () => void;
  /** What this tile paints. Absent for the two tiles that stand for an action
   * rather than a colour. */
  background?: Background;
  children?: ReactNode;
  isDisabled?: boolean;
  isSelected?: boolean;
  onContextMenu?: MouseEventHandler<HTMLDivElement>;
  /** A small copy of the picture, where whoever offered it already has one.
   * Given, it is what the swatch is drawn from: the system's own desktop
   * pictures are tens of megabytes, and this is a file a tile's size. */
  thumbnailPath?: string;
}) {
  const rendered = useBackgroundThumbnail(background, TILE_SIZE, thumbnailPath);
  // Outside the app there is no renderer, so a story shows the small copy as
  // the picture it is.
  const thumbnail = rendered ?? (isTauri() ? null : thumbnailPath);
  const style = background
    ? thumbnail
      ? {
          backgroundImage: `url("${thumbnail}")`,
          backgroundPosition: "center",
          backgroundSize: "cover",
        }
      : backgroundTileStyle(background)
    : undefined;

  return (
    <ToggleButton
      aria-label={ariaLabel}
      className={cn(
        "size-icon-large shrink-0 cursor-default overflow-hidden rounded-control",
        "flex items-center justify-center bg-fill-tertiary text-content-fg-secondary",
        "[&_svg]:size-icon",
        focusStyles,
        elementFocusVisible,
        isSelected && "ring-2 ring-primary ring-offset-2 ring-offset-content",
        isDisabled && "opacity-50",
      )}
      id={id}
      isDisabled={isDisabled}
      onContextMenu={onContextMenu}
      onPress={onPress}
      style={style}
    >
      {children}
    </ToggleButton>
  );
}
