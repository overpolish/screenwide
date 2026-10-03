// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Camera, Monitor, Pencil } from "lucide-react";
import { MouseEventHandler, ReactNode } from "react";
import { ToggleButton } from "react-aria-components";

import { cn, elementFocusVisible, focusStyles } from "../../../../lib/styling";

import { SchematicComposition, SchematicRect } from "./scene-schematic";

/** The narrowest a tile is drawn, width over height. Tiles share the panel's
 * width, so a portrait frame is drawn inside a square tile rather than
 * making the row a tower. */
const TILE_MINIMUM_ASPECT = 1;

/**
 * One pane of the drawing, placed as a share of the frame, with the glyph
 * that says which pane it is in its middle. The glyph keeps the small icon
 * size until the pane is too small to hold it with room around it, then
 * shrinks with the pane, so a camera drawn in a corner still reads as a pane.
 * Icons keep one stroke in screen pixels at every size, which would make a
 * shrunken glyph heavy, so its stroke thins in step with it: the usual 1.5px
 * at the small icon size, 14px, held to at least half that. The camera's
 * corner radius is the user's own setting rather than the scene's, so every
 * pane is drawn with the same rounding.
 *
 * The fills are translucent, so a pane laid over another shows where the two
 * overlap without an outline. A chosen tile wears the accent in every pane;
 * it is the tinted accent rather than the opaque one, which would let an
 * overlapping camera melt into the screen under it.
 */
function SchematicPane({
  children,
  isSelected,
  rect,
}: {
  children: ReactNode;
  isSelected: boolean;
  rect: SchematicRect;
}) {
  // Whole pixels for the pane and the glyph, so the glyph's margins come out
  // equal: a fractional box or glyph centres half a pixel off, which shows in
  // panes this small. Half-pixel margins land on whole device pixels on a
  // Retina display.
  const whole = (share: number) => `round(${String(share * 100)}%, 1px)`;
  return (
    <span
      className={cn(
        "absolute flex items-center justify-center rounded-control [container-type:size] [&_svg]:size-[round(down,min(var(--spacing-icon-small),60cqmin),1px)] [&_svg]:[stroke-width:clamp(0.75px,calc(60cqmin*3/28),1.5px)]",
        isSelected
          ? "bg-primary-tint text-primary-fg"
          : "bg-fill-secondary text-content-fg-secondary",
      )}
      style={{
        height: whole(rect.height),
        left: whole(rect.x),
        top: whole(rect.y),
        width: whole(rect.width),
      }}
    >
      {children}
    </span>
  );
}

/**
 * One scene on offer in the Scene panel: a small drawing of where it puts
 * the screen and the camera, at this recording's frame shape, each pane
 * marked with the glyph the recording bar gives that source so the sides read
 * at a glance. The name is the button's accessible name. A chosen tile fills
 * its panes with the accent rather than ringing the tile, which would crowd
 * the drawing. A pencil in the corner marks a tile that draws a layout it
 * would make editable rather than one of its own.
 */
export function ScenePresetTile({
  canvasAspect,
  hasEditBadge = false,
  id,
  isDisabled,
  isSelected,
  label,
  onContextMenu,
  onPress,
  schematic: { camera, cameraBehind = false, screen },
}: {
  /** The finished frame's width over its height. */
  canvasAspect: number;
  id: string;
  isSelected: boolean;
  label: string;
  onPress: () => void;
  /** Where the scene puts its panes, as shares of the canvas. */
  schematic: SchematicComposition;
  hasEditBadge?: boolean;
  isDisabled?: boolean;
  /** A press of the secondary button, which is how a template is removed. */
  onContextMenu?: MouseEventHandler<HTMLDivElement>;
}) {
  const screenPane = screen ? (
    <SchematicPane isSelected={isSelected} key="screen" rect={screen}>
      <Monitor aria-hidden="true" />
    </SchematicPane>
  ) : null;
  const cameraPane = camera ? (
    <SchematicPane isSelected={isSelected} key="camera" rect={camera}>
      <Camera aria-hidden="true" />
    </SchematicPane>
  ) : null;
  // The pencil sits in the corner furthest from the camera, so it never
  // covers the pane that tells the layouts apart.
  const cameraRight = camera ? camera.x + camera.width / 2 >= 0.5 : false;
  const cameraLow = camera ? camera.y + camera.height / 2 >= 0.5 : true;
  return (
    <ToggleButton
      aria-label={label}
      className={cn(
        "relative shrink-0 cursor-default rounded-control bg-fill-tertiary",
        focusStyles,
        elementFocusVisible,
        isDisabled && "opacity-50",
      )}
      id={id}
      isDisabled={isDisabled}
      onContextMenu={onContextMenu}
      onPress={onPress}
      // The grid it sits in gives it a whole-pixel width.
      style={{ aspectRatio: Math.max(canvasAspect, TILE_MINIMUM_ASPECT) }}
    >
      <span
        className="relative mx-auto block h-full"
        style={{ aspectRatio: canvasAspect }}
      >
        {cameraBehind ? [cameraPane, screenPane] : [screenPane, cameraPane]}
      </span>
      {hasEditBadge ? (
        <span
          className={cn(
            "absolute flex text-content-fg-secondary [&_svg]:size-icon-mini",
            cameraRight ? "start-control" : "end-control",
            cameraLow ? "top-control" : "bottom-control",
          )}
        >
          <Pencil aria-hidden="true" />
        </span>
      ) : null}
    </ToggleButton>
  );
}
