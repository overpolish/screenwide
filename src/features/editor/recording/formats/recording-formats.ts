// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// A recording made for several platforms at once. The first shape is the
// main canvas: the one sized freely, whose layout every other follows. The
// platforms added (YouTube, TikTok, Shorts) sit in shapes of their own at
// their templates' sizes, those of one proportion sharing a shape and its
// layout. Every shape shares the edit, and keeps its own canvas and camera;
// what the edit does differently is kept in `FrameFormats.edits` and worked
// out per format (`format-resolve.ts`). The shape in hand's canvas lives
// where the editor keeps it, in the project look, so the preview, the
// islands and the export see one recording as before; the others are kept
// aside in `FrameFormat.look` until one is chosen.

import { type FormatDocument, laidDocument } from "./format-document";
import { withoutLayouts } from "./format-write-parts";
import { platformClearArea } from "./platform-clear-area";

import type { FrameFormat } from "../../../../bindings/FrameFormat";
import type { FrameFormats } from "../../../../bindings/FrameFormats";
import type { PhoneView } from "../../../../bindings/PhoneView";

type Size = { height: number; width: number };

/** The phone a tall format is previewed on until another is chosen: the
 * common tall iPhone. */
const DEFAULT_PHONE: PhoneView = { phone: "iphone-17-pro" };

/** The shape in hand, as `formats` names it. */
export const activeFormat = (formats: FrameFormats) =>
  formats.list.find((format) => format.id === formats.active) ??
  formats.list[0];

/** A shape's formats, by id: a platform shape's templates, or the main
 * canvas itself, which is made for none. */
export const formatsOfShape = (shape: FrameFormat): string[] =>
  shape.platforms?.length ? shape.platforms : [shape.id];

/** Whether `id` is the main canvas of `formats`, or there are none yet,
 * the recording being its own main canvas. */
export const isMainCanvas = (formats: FrameFormats | null, id?: string) =>
  !formats || formats.list[0]?.id === (id ?? formats.active);

/** `formats` as a project opens them: a main canvas saved holding
 * platforms, as one made before it took none could be, holds none, those
 * platforms to be added again in shapes of their own. */
export const openedFormats = (
  formats: FrameFormats | null,
): FrameFormats | null => {
  const main = formats?.list[0];
  if (!formats || !main?.platforms) return formats;
  const { platform: _platform, platforms: _platforms, ...plain } = main;
  return { ...formats, list: [plain, ...formats.list.slice(1)] };
};

/** The format in hand while `shape` is. */
export const formatInHand = (shape: FrameFormat) => {
  const formats = formatsOfShape(shape);
  return shape.platform && formats.includes(shape.platform)
    ? shape.platform
    : formats[0];
};

export const newShapeId = () =>
  `shape-${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 6)}`;

/** `document` as a shape not in hand keeps it. */
export const kept = (document: FormatDocument): FrameFormat["look"] => ({
  cameraOverlay: document.cameraOverlay,
  recordingOutput: document.recordingOutput,
});

/** The format `id` put in hand: within the shape in hand only the platform
 * changes; from another shape, the one in hand's canvas is kept aside and
 * that shape's brought out. */
export const switchedFormat = (
  formats: FrameFormats,
  document: FormatDocument,
  id: string,
): { document: FormatDocument; formats: FrameFormats } | null => {
  const target = formats.list.find(
    (shape) => shape.id === id || formatsOfShape(shape).includes(id),
  );
  if (!target) return null;
  const platform = target.platforms?.includes(id) ? id : undefined;
  const inHandAs = (shape: FrameFormat): FrameFormat => {
    const { platform: _platform, ...rest } = shape;
    return platform ? { ...rest, platform } : rest;
  };
  if (target.id === formats.active)
    return formatInHand(target) === id || !platform
      ? null
      : {
          document,
          formats: {
            ...formats,
            list: formats.list.map((shape) =>
              shape === target ? inHandAs(shape) : shape,
            ),
          },
        };
  if (!target.look) return null;
  return {
    document: target.look,
    formats: {
      ...formats,
      active: target.id,
      list: formats.list.map((shape) => {
        if (shape.id === formats.active)
          return { ...shape, look: kept(document) };
        if (shape !== target) return shape;
        const { look: _look, ...inHand } = inHandAs(shape);
        return inHand;
      }),
    },
  };
};

/**
 * The platform `preset`, of canvas `size`, added and put in hand, in a
 * shape of its own: the screen laid out on it from the main canvas,
 * whichever is in hand, clear of what the platform's app covers
 * (`laidDocument`), as every later change to the main one is
 * (`withLookCarried`). Platforms share a shape only when asked to
 * (`format-sharing.ts`). The main canvas, the first shape, takes no
 * platform: it is the one canvas sized freely.
 */
export const addedFormat = (
  formats: FrameFormats | null,
  document: FormatDocument,
  { preset, size }: { preset: string; size: Size },
): { document: FormatDocument; formats: FrameFormats } => {
  const current = formatsOrNew(formats);
  const id = newShapeId();
  return {
    document: laidDocument(
      mainDocument(current, document),
      size,
      platformClearArea([preset]),
    ),
    formats: {
      ...current,
      active: id,
      list: [
        ...current.list.map((each) =>
          each.id === current.active ? { ...each, look: kept(document) } : each,
        ),
        { id, platform: preset, platforms: [preset] },
      ],
    },
  };
};

/** The main canvas's document: `document` while it is in hand, else the
 * one it keeps aside. */
export const mainDocument = (
  formats: FrameFormats,
  document: FormatDocument,
): FormatDocument => {
  const main = formats.list[0];
  return main.id === formats.active || !main.look ? document : main.look;
};

/** The platform `id` taken away with its shape, putting the main canvas
 * in hand where that shape was; one sharing its shape with others leaves
 * it instead (`removedPlatform` in `format-sharing.ts`). The main canvas
 * itself is never taken away; with no platform left it is the recording's
 * only shape again. */
export const removedFormat = (
  formats: FrameFormats,
  document: FormatDocument,
  id: string,
): { document: FormatDocument; formats: FrameFormats | null } => {
  const [main] = formats.list;
  const shape = formats.list
    .slice(1)
    .find((each) => formatsOfShape(each).includes(id));
  if (!shape) return { document, formats };
  // The shape goes with what it laid out its own way.
  const cleared = formats.edits
    ? { ...formats, edits: withoutLayouts(formats.edits, shape.id) }
    : formats;
  const left = (next: FrameFormats) =>
    next.list.length > 2
      ? {
          ...next,
          list: next.list.filter((each) => each.id !== shape.id),
        }
      : single(next, [next.list[0]]);
  if (shape.id !== cleared.active) return { document, formats: left(cleared) };
  const switched = switchedFormat(cleared, document, main.id);
  return switched
    ? { document: switched.document, formats: left(switched.formats) }
    : { document, formats: left(cleared) };
};

/** One shape left, in hand. */
const single = (
  formats: FrameFormats,
  list: FrameFormat[],
): FrameFormats | null => {
  const [only] = list;
  if (list.length !== 1) return null;
  const { look: _look, ...inHand } = only;
  return keptFormats({ ...formats, active: only.id, list: [inHand] });
};

/** `formats` while it holds anything a recording made in one shape needs
 * to keep: a second shape, a platform, a template the main canvas is made
 * for, the phone or the preview. */
export const keptFormats = (formats: FrameFormats): FrameFormats | null => {
  const [only] = formats.list;
  return formats.list.length === 1 &&
    !only.platforms?.length &&
    !only.madeFor &&
    !formats.phone &&
    !formats.preview
    ? null
    : formats;
};

/** `formats`, or the formats of a recording made in one shape, previewed on
 * the common phone until another is chosen. */
export const formatsOrNew = (formats: FrameFormats | null): FrameFormats => {
  if (formats) return formats;
  const active = `${newShapeId()}-0`;
  return { active, list: [{ id: active }], phone: DEFAULT_PHONE };
};

/** `formats` with every format previewed on `phone`, or on none. */
export const withPhone = (
  formats: FrameFormats | null,
  phone: PhoneView | null,
): FrameFormats | null => {
  const { phone: _phone, ...rest } = formatsOrNew(formats);
  return keptFormats(phone ? { ...rest, phone } : rest);
};

/** `formats` with the phone and the apps shown over the picture, or not. */
export const withPreview = (
  formats: FrameFormats | null,
  isShown: boolean,
): FrameFormats | null => {
  const { preview: _preview, ...rest } = formatsOrNew(formats);
  return keptFormats(isShown ? { ...rest, preview: true } : rest);
};
