// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { laidCamera, laidPlacement } from "./format-document";

import type { FormatDocument } from "./format-document";
import type { EditScope } from "./format-scope";
import type { FrameFormats } from "../../../../bindings/FrameFormats";

// A change to the screen or the camera as the main canvas shows them
// (inset, corners, turn, shadow, background, crop, place) reaches every
// platform, as a change to a scene does. Places are in output pixels, so
// the screen is laid out on each platform's canvas from where it is on the
// main one (`laidPlacement`), and the camera moves with it: a move, a
// resize or a reshaped canvas on the main one shows on every platform. A
// platform whose screen was set its own way (moved, resized, turned,
// cropped or restyled with it in hand) keeps all of the screen's own
// settings; only the background still reaches it. The camera's own canvas,
// when it is saved apart, is its own, and its places only carry to one of
// the same size.

/** Output pixels, meaningful on a canvas of one size alone. */
const PLACED_OUTPUT: Record<string, true> = {
  cropHeight: true,
  cropWidth: true,
  cropX: true,
  cropY: true,
  imageWidth: true,
  imageX: true,
  imageY: true,
};
const PLACED_CAMERA: Record<string, true> = {
  cameraWidth: true,
  cameraX: true,
  cameraY: true,
  frameHeight: true,
  frameWidth: true,
  frameX: true,
  frameY: true,
};
/** The screen's own settings: where it is and how it is shown, as against
 * the canvas's background around it. A platform set its own way keeps
 * these. */
const SCREEN: Record<string, true> = {
  ...PLACED_OUTPUT,
  dropShadow: true,
  inset: true,
  radius: true,
  recenterInsetColor: true,
  rotation: true,
  sourceCrop: true,
};
/** The canvas itself, what only the preview draws, and the part of the
 * canvas each platform leaves clear, which is its own. */
const NEVER: Record<string, true> = {
  clearArea: true,
  cropPreview: true,
  dropShadowStrength: true,
  height: true,
  width: true,
};

const isIn = (table: Record<string, true>, key: string) =>
  Object.prototype.hasOwnProperty.call(table, key);

const same = (left: unknown, right: unknown) =>
  JSON.stringify(left) === JSON.stringify(right);

/** What changed from `before` to `after`, outside `skipped`, written over
 * a target. */
const carriedOnto =
  <T extends object>(
    { after, before }: { after: T; before: T },
    skipped: (key: string) => boolean,
  ) =>
  (target: T): T => {
    const next = { ...target } as Record<string, unknown>;
    const keys = new Set([...Object.keys(before), ...Object.keys(after)]);
    for (const key of keys) {
      if (skipped(key)) continue;
      const value = (after as Record<string, unknown>)[key];
      if (same((before as Record<string, unknown>)[key], value)) continue;
      if (value === undefined) Reflect.deleteProperty(next, key);
      else next[key] = value;
    }
    return next as T;
  };

/** Whether anything `table` names changed from `before` to `after`. */
const movedIn = <T extends object>(
  table: Record<string, true>,
  { after, before }: { after: T; before: T },
) =>
  Object.keys(table).some(
    (key) =>
      !same(
        (after as Record<string, unknown>)[key],
        (before as Record<string, unknown>)[key],
      ),
  );

/** The keys of `table` taken from `from`, written over `target`. */
const placedFrom = <T extends object>(
  table: Record<string, true>,
  from: T,
  target: T,
): T => {
  const next = { ...target } as Record<string, unknown>;
  for (const key of Object.keys(table))
    next[key] = (from as Record<string, unknown>)[key];
  return next as T;
};

/** Whether the screen or the camera was changed from `before` to `after`:
 * moved, resized, turned, cropped or restyled, rather than the canvas's
 * background changed. */
export const isScreenChange = ({
  after,
  before,
}: {
  after: FormatDocument;
  before: FormatDocument;
}) =>
  movedIn(SCREEN, {
    after: after.recordingOutput.primary,
    before: before.recordingOutput.primary,
  }) ||
  movedIn(PLACED_CAMERA, {
    after: after.cameraOverlay,
    before: before.cameraOverlay,
  });

/** `look` with the screen and the camera set as `laid` sets them, and the
 * clear area `laid`'s; the rest of it, such as the background, its own. */
export const placedAs = (
  look: FormatDocument,
  laid: FormatDocument,
): FormatDocument => {
  const { clearArea: _was, ...primary } = placedFrom(
    SCREEN,
    laid.recordingOutput.primary,
    look.recordingOutput.primary,
  );
  const clearArea = laid.recordingOutput.primary.clearArea;
  return {
    cameraOverlay: placedFrom(
      PLACED_CAMERA,
      laid.cameraOverlay,
      look.cameraOverlay,
    ),
    recordingOutput: {
      ...look.recordingOutput,
      primary: clearArea ? { ...primary, clearArea } : primary,
    },
  };
};

/**
 * `formats` with the change to the shape in hand's look, `before` to
 * `after`, carried to the shapes `scope` reaches: none for this shape,
 * every one for all shapes. Null where nothing else changes.
 */
export function withLookCarried(
  formats: FrameFormats | null,
  { after, before }: { after: FormatDocument; before: FormatDocument },
  scope: EditScope,
): FrameFormats | null {
  if (!formats || scope === "shape") return null;
  const canvas = after.recordingOutput.primary;
  const change = <K extends "camera" | "primary">(track: K) => ({
    after: after.recordingOutput[track],
    before: before.recordingOutput[track],
  });
  const reshaped =
    canvas.width !== before.recordingOutput.primary.width ||
    canvas.height !== before.recordingOutput.primary.height;
  const screenMoved = reshaped || movedIn(PLACED_OUTPUT, change("primary"));
  // The camera moves with the screen, so a move of either places it again.
  const cameraMoved =
    screenMoved ||
    movedIn(PLACED_CAMERA, {
      after: after.cameraOverlay,
      before: before.cameraOverlay,
    });
  const list = formats.list.map((format) => {
    const look = format.look;
    if (format.id === formats.active || !look) return format;
    const size = look.recordingOutput.primary;
    const clear = size.clearArea;
    const sameSize =
      size.width === canvas.width && size.height === canvas.height;
    // A platform whose screen is set its own way keeps all of the screen's
    // own settings; only the background still comes from the main canvas.
    const follows = format.placed !== true;
    const skipped = (key: string) =>
      isIn(NEVER, key) ||
      isIn(PLACED_OUTPUT, key) ||
      (!follows && isIn(SCREEN, key));
    const camera = carriedOnto(
      { after: after.cameraOverlay, before: before.cameraOverlay },
      (key) => isIn(PLACED_CAMERA, key),
    )(look.cameraOverlay);
    const primary = carriedOnto(
      change("primary"),
      skipped,
    )(look.recordingOutput.primary);
    const next = {
      cameraOverlay:
        cameraMoved && follows
          ? placedFrom(
              PLACED_CAMERA,
              laidCamera(after.cameraOverlay, canvas, { clear, size }),
              camera,
            )
          : camera,
      recordingOutput: {
        camera: carriedOnto(
          change("camera"),
          (key) => isIn(NEVER, key) || (!sameSize && isIn(PLACED_OUTPUT, key)),
        )(look.recordingOutput.camera),
        primary:
          screenMoved && follows
            ? placedFrom(
                PLACED_OUTPUT,
                laidPlacement(canvas, size, clear),
                primary,
              )
            : primary,
      },
    };
    return same(next, look) ? format : { ...format, look: next };
  });
  return list.some((format, index) => format !== formats.list[index])
    ? { ...formats, list }
    : null;
}
