// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  BACKGROUND,
  carriedOnto,
  isIn,
  movedIn,
  NEVER,
  SCREEN,
} from "../../recording/formats/format-look-carry";
import { proportionKey } from "../../recording/formats/format-scope";
import { platformClearArea } from "../../recording/formats/platform-clear-area";
import {
  activeFormat,
  formatsOfShape,
  isMainCanvas,
  mainDocument,
  switchedFormat,
  withShapeAdded,
  withShapeRemoved,
} from "../../recording/formats/recording-formats";
import { frameSizePreset } from "../../tools/frame/frame-sizes";

import {
  mergedAnnotations,
  withOwnLayouts,
} from "./screenshot-annotation-layouts";
import {
  type Canvas,
  inverseMapping,
  laidWorkspace,
  moved,
  workspaceMapping,
} from "./screenshot-format-layout";
import { canvasOf, shownWorkspace } from "./screenshot-format-resolve";
import { laidApart } from "./screenshot-show-in";

import type { ScreenshotFormats } from "../../../../bindings/ScreenshotFormats";
import type { ScreenshotShape } from "../../../../bindings/ScreenshotShape";
import type { ScreenshotWorkspaceOutputSettings } from "../screenshot-output";

// A screenshot made for several platforms at once, as a recording is
// (`recording-formats.ts`): the main canvas, then a shape for each platform
// at its template's size. What the screenshot shows is the main canvas's:
// its layers, what is drawn on them, and its background. A platform lays
// the main canvas's layers out on its own canvas (`laidWorkspace`), clear
// of what its app covers, until its layers are set its own way there,
// moved, resized, cropped or restyled with it in hand (`placed`); it then
// keeps where and how its layers show, and takes the rest from the main
// canvas still. Its background, changed with it in hand, is its own too
// (`ownBackground`) until it is reset to follow the main canvas, and so is
// an annotation changed there, for every platform of its proportion
// (`screenshot-annotation-layouts.ts`). Anything else changed with a
// platform in hand is the main canvas's, written back to it as that
// platform is put down (`settledFormats`).

type Workspace = ScreenshotWorkspaceOutputSettings;
type Annotations = Workspace["items"][number]["output"]["annotations"];
/** The layers drawn on rather than pictures, by id. */
type Blank = ReadonlySet<number>;

/** The workspace the window shows, of the format in hand, and the layers
 * in it drawn on rather than pictures. */
export type InHand = { blank: Blank; workspace: Workspace };

/** Where and how each layer shows, and what the canvas sits on, against
 * everything else about it: a platform keeps these its own. */
const isPlatformOwn = (key: string) =>
  isIn(NEVER, key) || isIn(SCREEN, key) || isIn(BACKGROUND, key);

/** Whether a layer of `after` was moved, resized, cropped or restyled
 * since `before`. */
const isLayersMoved = (before: Workspace, after: Workspace) =>
  movedIn(SCREEN, { after, before }) ||
  after.items.some((item) => {
    const was = before.items.find((each) => each.id === item.id);
    return (
      was !== undefined &&
      movedIn(SCREEN, { after: item.output, before: was.output })
    );
  });

/** `main` with what changed from `before` to `after` on `platform` other
 * than where and how its layers show and its background: what is drawn,
 * layers added, taken away or reordered. A layer added there is placed on
 * the main canvas where the platform's layout puts it back; an annotation
 * changed there is placed its own way on it, returned in `own`. */
const withContentOf = (
  main: Workspace,
  { after, before }: { after: Workspace; before: Workspace },
  { blank, platform }: { blank: Blank; platform: ScreenshotShape },
) => {
  const back = inverseMapping(
    workspaceMapping(main, canvasOf(platform, main), blank),
  );
  const home: Canvas = { clear: main.clearArea, size: main };
  const mains = new Map(main.items.map((item) => [item.id, item]));
  const befores = new Map(before.items.map((item) => [item.id, item]));
  const canvas = carriedOnto(
    { after, before },
    (key) => key === "items" || isPlatformOwn(key),
  )(main);
  const own: Annotations = [];
  const items = after.items.map((item) => {
    const was = befores.get(item.id);
    const kept = mains.get(item.id);
    if (!was || !kept)
      return { ...item, output: moved(item.output, back, home) };
    const annotations = mergedAnnotations(kept.output.annotations, {
      after: item.output.annotations,
      before: was.output.annotations,
    });
    own.push(...annotations.own);
    const output = carriedOnto(
      { after: item.output, before: was.output },
      (key) => key === "annotations" || isPlatformOwn(key),
    )(kept.output);
    return { ...kept, output: { ...output, annotations: annotations.merged } };
  });
  return { look: { ...canvas, items }, own };
};

/** `formats` with what changed on the platform in hand written back: to
 * the main canvas, and its layers' places, its background and the
 * annotations changed there to the platform, or to its proportion. Unchanged
 * with the main canvas in hand. */
export const settledFormats = (
  formats: ScreenshotFormats,
  { blank, workspace }: InHand,
): ScreenshotFormats => {
  const [main] = formats.list;
  if (isMainCanvas(formats) || !main.look) return formats;
  const shape = activeFormat(formats);
  const before = shownWorkspace(formats, { blank, main: main.look, shape });
  const { look, own } = withContentOf(
    main.look,
    { after: workspace, before },
    { blank: laidApart(formats, { blank, shape }), platform: shape },
  );
  const placed = shape.placed !== true && isLayersMoved(before, workspace);
  const ownBackground =
    shape.ownBackground !== true &&
    movedIn(BACKGROUND, { after: workspace, before });
  const layouts = withOwnLayouts(formats.layouts, {
    key: proportionKey(workspace),
    own,
  });
  return {
    ...formats,
    ...(layouts ? { layouts } : {}),
    list: formats.list.map((each) => {
      if (each === main) return { ...each, look };
      if (each !== shape || (!placed && !ownBackground)) return each;
      return {
        ...each,
        ...(placed ? { placed: true } : {}),
        ...(ownBackground ? { ownBackground: true } : {}),
      };
    }),
  };
};

/** The format `id` put in hand, the one in hand settled first and the one
 * put in hand showing the main canvas as it is now. */
export const switchedScreenshotFormat = (
  formats: ScreenshotFormats,
  inHand: InHand,
  id: string,
) => {
  const settled = settledFormats(formats, inHand);
  const main = mainDocument(settled, inHand.workspace);
  return switchedFormat(
    {
      ...settled,
      list: settled.list.map((shape) =>
        shape.id === settled.active || !formatsOfShape(shape).includes(id)
          ? shape
          : {
              ...shape,
              look: shownWorkspace(settled, {
                blank: inHand.blank,
                main,
                shape,
              }),
            },
      ),
    },
    inHand.workspace,
    id,
  );
};

/** The platform `preset` added and put in hand, the main canvas laid out
 * on its template's canvas, with its proportion's own annotations. */
export const addedScreenshotFormat = (
  formats: ScreenshotFormats | null,
  inHand: InHand,
  preset: string,
) => {
  const template = frameSizePreset(preset);
  if (!template) return null;
  const canvas = { clear: platformClearArea([preset]), size: template };
  const settled = formats && settledFormats(formats, inHand);
  const shape = { id: preset, platforms: [preset] };
  return withShapeAdded(settled, inHand.workspace, {
    lay: (main) =>
      settled
        ? shownWorkspace(settled, { blank: inHand.blank, main, shape })
        : laidWorkspace(main, canvas, inHand.blank),
    preset,
  });
};

/** The platform `id` taken away, the main canvas put in hand where it was. */
export const removedScreenshotFormat = (
  formats: ScreenshotFormats,
  inHand: InHand,
  id: string,
) =>
  withShapeRemoved(settledFormats(formats, inHand), inHand.workspace, {
    cleared: (current) => current,
    id,
  });
