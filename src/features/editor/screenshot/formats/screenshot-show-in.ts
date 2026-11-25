// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import {
  type FormatContext,
  isShownIn,
  withHere,
  withoutHere,
} from "../../recording/formats/format-scope";
import {
  activeFormat,
  formatInHand,
  formatsOfShape,
} from "../../recording/formats/recording-formats";

import type { FormatScope } from "../../../../bindings/FormatScope";
import type { ScreenshotFormats } from "../../../../bindings/ScreenshotFormats";
import type { ScreenshotShape } from "../../../../bindings/ScreenshotShape";
import type { FormatSync } from "../../recording/formats/format-sync";
import type { ScreenshotWorkspaceOutputSettings } from "../screenshot-output";

// A layer or an annotation of a screenshot can be shown in only some of its
// formats, ticked per format in its right-click menu as a recording's are
// (`format-sync.ts`). What a format leaves out stays in the workspace, so
// every format keeps it and its edits, but is not drawn there
// (`drawnWorkspace`), and a platform lays its layers out without it.

type Workspace = ScreenshotWorkspaceOutputSettings;
type Table = "annotations" | "layers";

/** What a format leaves out: its layers by id, its annotations by id. */
export type LeftOut = {
  annotations: ReadonlySet<string>;
  layers: ReadonlySet<number>;
};

const LAYER = "layer:";

/** The id a layer's menu asks about, told apart from an annotation's. */
export const layerSyncId = (id: number) => `${LAYER}${String(id)}`;

const entryOf = (syncId: string): { key: string; table: Table } =>
  syncId.startsWith(LAYER)
    ? { key: syncId.slice(LAYER.length), table: "layers" }
    : { key: syncId, table: "annotations" };

/** Every format, shape by shape, as the recording's scope helpers read
 * them, with `id` the one asked about. */
const contextOf = (formats: ScreenshotFormats, id: string): FormatContext => ({
  annotationKey: "",
  id,
  shape: "",
  shapes: formats.list.map((shape) => ({
    annotationKey: "",
    formats: formatsOfShape(shape),
    id: shape.id,
  })),
});

/** `scope` without the formats taken away since it was set; every format
 * where none it named is left, as nothing set apart would be lost from
 * them all. */
const liveScope = (
  scope: FormatScope | undefined,
  formats: ScreenshotFormats,
): FormatScope | undefined => {
  if (scope === undefined || scope === "all") return scope;
  const every = formats.list.flatMap(formatsOfShape);
  const left = scope.formats.filter((id) => every.includes(id));
  return left.length > 0 ? { formats: left } : undefined;
};

const scopeOf = (formats: ScreenshotFormats, syncId: string) => {
  const { key, table } = entryOf(syncId);
  return liveScope(formats.showIn?.[table]?.[key], formats);
};

/** What a layer's or an annotation's menu offers to set it apart by
 * format: a tick for each format it shows in; and for an annotation,
 * whether it is custom on the platforms of the proportion in hand, `key`
 * (null on the main canvas), and making its placement every shape's.
 * Null with one format. */
export const screenshotFormatSync = (
  formats: ScreenshotFormats | null,
  { key, syncId }: { key: string | null; syncId: string },
): FormatSync | null => {
  const every = formats?.list.flatMap(formatsOfShape) ?? [];
  if (!formats || every.length < 2) return null;
  const scope = scopeOf(formats, syncId);
  const isLayer = syncId.startsWith(LAYER);
  const own = Object.keys(formats.layouts?.[syncId] ?? {});
  const isCustom = key !== null && own.includes(key);
  return {
    byFraming: false,
    canSpread: !isLayer && (isCustom || (key === null && own.length > 0)),
    // The main canvas's placement is the one every shape starts from.
    custom: isLayer || key === null ? null : isCustom,
    formats: every.map((id) => ({
      id,
      isShown: isShownIn(scope, contextOf(formats, id)),
    })),
  };
};

/** `formats` with `syncId` shown in `format`, or left out of it where it
 * shows there; left out of the only format it shows in, it would show
 * nowhere, which is deleting it, so that is left as it is. */
export const withScreenshotShownToggled = (
  formats: ScreenshotFormats,
  { format, syncId }: { format: string; syncId: string },
): ScreenshotFormats => {
  const { key, table } = entryOf(syncId);
  const was = scopeOf(formats, syncId);
  const context = contextOf(formats, format);
  const scope = isShownIn(was, context)
    ? withoutHere(was, [format], context)
    : withHere(was, [format], context);
  if (scope === null) return formats;
  const rest = Object.entries(formats.showIn?.[table] ?? {}).filter(
    ([each]) => each !== key,
  );
  return {
    ...formats,
    showIn: {
      ...formats.showIn,
      [table]: Object.fromEntries(
        scope === undefined ? rest : [...rest, [key, scope]],
      ),
    },
  };
};

/** What every one of `ids`, the formats asked about, leaves out. */
export const leftOutOfAll = (
  formats: ScreenshotFormats,
  ids: readonly string[],
): LeftOut => {
  const leaves = (table: Table) =>
    Object.keys(formats.showIn?.[table] ?? {}).filter((key) => {
      const scope = liveScope(formats.showIn?.[table]?.[key], formats);
      return ids.every((id) => !isShownIn(scope, contextOf(formats, id)));
    });
  return {
    annotations: new Set(leaves("annotations")),
    layers: new Set(leaves("layers").map(Number)),
  };
};

/** What the format in hand leaves out; nothing without formats. */
export const leftOutInHand = (formats: ScreenshotFormats | null): LeftOut =>
  formats
    ? leftOutOfAll(formats, [formatInHand(activeFormat(formats))])
    : { annotations: new Set(), layers: new Set() };

/** Whether the format `id` leaves anything out. */
export const isLeavingOut = (formats: ScreenshotFormats, id: string) => {
  const { annotations, layers } = leftOutOfAll(formats, [id]);
  return annotations.size > 0 || layers.size > 0;
};

/** `workspace` as the format leaving out `leftOut` is published: without
 * the layers it leaves out, and the annotations it leaves out on the
 * rest. */
export const publishedWorkspace = (
  workspace: Workspace,
  { annotations, layers }: LeftOut,
): Workspace => ({
  ...workspace,
  items: workspace.items
    .filter((item) => !layers.has(item.id))
    .map((item) => ({
      ...item,
      output: {
        ...item.output,
        annotations: item.output.annotations.filter(
          (annotation) => !annotations.has(annotation.id),
        ),
      },
    })),
});

/** The layers `shape` lays out apart from the rest: `blank`, drawn on
 * rather than pictures, and those every one of its formats leaves out. */
export const laidApart = (
  formats: ScreenshotFormats,
  { blank, shape }: { blank: ReadonlySet<number>; shape: ScreenshotShape },
): ReadonlySet<number> => {
  const { layers } = leftOutOfAll(formats, formatsOfShape(shape));
  return layers.size === 0 ? blank : new Set([...blank, ...layers]);
};

/** `workspace` as the format leaving out `leftOut` draws it: the layers it
 * leaves out kept in their places but drawing nothing, and the annotations
 * it leaves out gone from their layers. */
export const drawnWorkspace = (
  workspace: Workspace,
  { annotations, layers }: LeftOut,
): Workspace =>
  annotations.size === 0 && layers.size === 0
    ? workspace
    : {
        ...workspace,
        items: workspace.items.map((item) => {
          const hidden = layers.has(item.id);
          return {
            ...item,
            hidden,
            output: {
              ...item.output,
              annotations: hidden
                ? []
                : item.output.annotations.filter(
                    (annotation) => !annotations.has(annotation.id),
                  ),
            },
          };
        }),
      };

/** `incoming`, a layer's annotations as the drawn workspace changed them,
 * with those `previous` held that the format leaves out put back where
 * they were. */
export const withLeftOutKept = <Annotation extends { id: string }>(
  incoming: readonly Annotation[],
  {
    leftOut,
    previous,
  }: { leftOut: ReadonlySet<string>; previous: readonly Annotation[] },
): Annotation[] => {
  const kept = [...incoming];
  previous.forEach((annotation, index) => {
    if (leftOut.has(annotation.id))
      kept.splice(Math.min(index, kept.length), 0, annotation);
  });
  return kept;
};
