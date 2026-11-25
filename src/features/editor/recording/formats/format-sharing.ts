// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { laidDocument } from "./format-document";
import {
  isBackgroundChange,
  isScreenChange,
  placedAs,
} from "./format-look-carry";
import { withoutLayouts } from "./format-write-parts";
import { platformClearArea } from "./platform-clear-area";
import {
  activeFormat,
  formatInHand,
  isMainCanvas,
  kept,
  mainDocument,
  newShapeId,
  removedFormat,
} from "./recording-formats";

import type { FormatDocument } from "./format-document";
import type { FormatEdits } from "../../../../bindings/FormatEdits";
import type { FrameFormat } from "../../../../bindings/FrameFormat";
import type { FrameFormats } from "../../../../bindings/FrameFormats";

// Platforms follow the main canvas, each laid out for its own canvas and
// the part of it its app leaves clear, until one is framed its own way:
// its screen changed while it is in hand (`placed`), or a scene laid out
// there. Platforms of one canvas can then share that framing: they sit in
// one shape, which a change made with any of them in hand reaches. One left
// out of it goes back to following the main canvas, and a platform still
// following it can take up another's framing instead.

type Changed = { document: FormatDocument; formats: FrameFormats };

/** Whether `shape` is framed its own way: its screen placed there, or a
 * scene laid out for it. */
export const isFramedOwnWay = (
  shape: FrameFormat,
  edits: FormatEdits | undefined,
) =>
  shape.placed === true ||
  Object.values(edits?.scenes ?? {}).some(
    (scene) => scene.layouts?.[shape.id] !== undefined,
  );

/** The platform shape holding `platform`. */
export const shapeHolding = <Shape extends Pick<FrameFormat, "platforms">>(
  formats: { list: Shape[] },
  platform: string,
) =>
  formats.list
    .slice(1)
    .find((shape) => shape.platforms?.includes(platform) === true);

/** `look`, a shape's, kept clear of what `platforms` cover: laid out from
 * the main canvas's document `main` again where the shape follows it, else
 * only told the new clear area. */
const relaid = (
  main: FormatDocument,
  { look, placed }: { look: FormatDocument; placed: boolean },
  platforms: readonly string[],
): FormatDocument => {
  const clear = platformClearArea(platforms);
  const size = look.recordingOutput.primary;
  if (!placed) return placedAs(look, laidDocument(main, size, clear));
  const { clearArea: _was, ...primary } = size;
  return {
    ...look,
    recordingOutput: {
      ...look.recordingOutput,
      primary: clear ? { ...primary, clearArea: clear } : primary,
    },
  };
};

/** `formats` without the shape `id`, and what was laid out for it. */
const withoutShape = (formats: FrameFormats, id: string): FrameFormats => ({
  ...formats,
  ...(formats.edits ? { edits: withoutLayouts(formats.edits, id) } : {}),
  list: formats.list.filter((shape) => shape.id !== id),
});

/**
 * The platform `platform` made to share the framing of the shape in hand,
 * moved into it from its own shape, which goes where it held no other; or,
 * `isShared` false, taken out of it into a shape of its own that follows the
 * main canvas. The platform in hand stays. Null where nothing changes.
 */
export function withPlatformShared(
  formats: FrameFormats,
  document: FormatDocument,
  { isShared, platform }: { isShared: boolean; platform: string },
): Changed | null {
  const active = activeFormat(formats);
  if (isMainCanvas(formats) || platform === formatInHand(active)) return null;
  const platforms = active.platforms ?? [];
  const isIn = platforms.includes(platform);
  if (isShared === isIn) return null;
  const inHand = { look: document, placed: active.placed === true };
  if (isShared) {
    const from = shapeHolding(formats, platform);
    if (!from) return null;
    const left = (from.platforms ?? []).filter((each) => each !== platform);
    const without =
      left.length === 0
        ? withoutShape(formats, from.id)
        : {
            ...formats,
            list: formats.list.map((shape) =>
              shape === from ? leftBy(shape, platform, left) : shape,
            ),
          };
    const joined = [...platforms, platform];
    return {
      document: relaid(mainDocument(formats, document), inHand, joined),
      formats: {
        ...without,
        list: without.list.map((shape) =>
          shape.id === active.id ? { ...shape, platforms: joined } : shape,
        ),
      },
    };
  }
  const left = platforms.filter((each) => each !== platform);
  const size = document.recordingOutput.primary;
  const own: FrameFormat = {
    id: newShapeId(),
    look: kept(
      laidDocument(
        mainDocument(formats, document),
        size,
        platformClearArea([platform]),
      ),
    ),
    platform,
    platforms: [platform],
  };
  return {
    document: relaid(mainDocument(formats, document), inHand, left),
    formats: {
      ...formats,
      // Next to the shape it leaves, so the format bar keeps its order.
      list: formats.list.flatMap((shape) =>
        shape.id === active.id ? [leftBy(shape, platform, left), own] : [shape],
      ),
    },
  };
}

/** `shape` with `platform` gone from it, `left` the platforms it keeps. */
export const leftBy = <Shape extends Pick<FrameFormat, "platform">>(
  shape: Shape,
  platform: string,
  left: string[],
): Shape => {
  // The platform in hand stays, unless it is the one gone.
  const inHand = shape.platform === platform ? undefined : shape.platform;
  return { ...shape, platform: inHand, platforms: left };
};

/**
 * The shape in hand, following the main canvas, made to share the framing
 * of the shape `id` instead: its platforms move there and it goes, and that
 * shape comes into hand with the same platform in hand. Null where nothing
 * changes.
 */
export function withShapeFollowed(
  formats: FrameFormats,
  document: FormatDocument,
  id: string,
): Changed | null {
  const active = activeFormat(formats);
  const target = formats.list.find((shape) => shape.id === id);
  if (isMainCanvas(formats) || !target?.look || target === active) return null;
  const platforms = [...(target.platforms ?? []), ...(active.platforms ?? [])];
  const without = withoutShape(formats, active.id);
  const { look, ...rest } = target;
  return {
    document: relaid(
      mainDocument(formats, document),
      { look, placed: target.placed === true },
      platforms,
    ),
    formats: {
      ...without,
      active: target.id,
      list: without.list.map((shape) =>
        shape === target
          ? { ...rest, platform: formatInHand(active), platforms }
          : shape,
      ),
    },
  };
}

/** The shape in hand's screen and camera set as the main canvas sets them
 * again, no longer its own. A framing shared only for its screen has
 * nothing left to share then: each platform it held follows the main
 * canvas on its own again, laid out for its own app. Null on the main
 * canvas. */
export function withMainPlacement(
  formats: FrameFormats,
  document: FormatDocument,
): Changed | null {
  const active = activeFormat(formats);
  if (isMainCanvas(formats)) return null;
  const { placed: _placed, ...following } = active;
  let changed: Changed = {
    document: relaid(
      mainDocument(formats, document),
      { look: document, placed: false },
      active.platforms ?? [],
    ),
    formats: {
      ...formats,
      list: formats.list.map((shape) => (shape === active ? following : shape)),
    },
  };
  // A scene laid out for the shape still frames it its own way, and keeps
  // the platforms sharing it.
  if (isFramedOwnWay(following, formats.edits)) return changed;
  const inHand = formatInHand(following);
  for (const platform of following.platforms ?? []) {
    if (platform === inHand) continue;
    changed =
      withPlatformShared(changed.formats, changed.document, {
        isShared: false,
        platform,
      }) ?? changed;
  }
  return changed;
}

/** `formats` with the shape in hand's screen, or its background, set its
 * own way where the change `before` to `after` changed it; null where it
 * is the main canvas, or nothing new is its own. */
export function withOwnHere(
  formats: FrameFormats,
  change: { after: FormatDocument; before: FormatDocument },
): FrameFormats | null {
  const active = activeFormat(formats);
  if (isMainCanvas(formats)) return null;
  const placed = active.placed !== true && isScreenChange(change);
  const ownBackground =
    active.ownBackground !== true && isBackgroundChange(change);
  if (!placed && !ownBackground) return null;
  return {
    ...formats,
    list: formats.list.map((shape) =>
      shape === active
        ? {
            ...shape,
            ...(placed ? { placed: true } : {}),
            ...(ownBackground ? { ownBackground: true } : {}),
          }
        : shape,
    ),
  };
}

/** The platform `id` taken away: out of a shape it shares with others,
 * which is kept clear of what those left cover; else with its shape
 * (`removedFormat`). */
export function removedPlatform(
  formats: FrameFormats,
  document: FormatDocument,
  id: string,
): { document: FormatDocument; formats: FrameFormats | null } {
  const shape = shapeHolding(formats, id);
  const platforms = shape?.platforms ?? [];
  if (!shape || platforms.length < 2)
    return removedFormat(formats, document, id);
  const left = platforms.filter((each) => each !== id);
  const isActive = shape.id === formats.active;
  const look = isActive ? document : shape.look;
  if (!look) return removedFormat(formats, document, id);
  const next = relaid(
    mainDocument(formats, document),
    { look, placed: shape.placed === true },
    left,
  );
  return {
    document: isActive ? next : document,
    formats: {
      ...formats,
      list: formats.list.map((each) =>
        each === shape
          ? {
              ...leftBy(each, id, left),
              ...(isActive ? {} : { look: kept(next) }),
            }
          : each,
      ),
    },
  };
}
