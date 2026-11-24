// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { sceneLayoutOf } from "./format-resolve";
import {
  allFormats,
  CAPTIONS_ID,
  type FormatContext,
  isShownIn,
  withHere,
  withoutHere,
} from "./format-scope";
import {
  defined,
  type Formats,
  tidy,
  withoutItemLayouts,
} from "./format-write-parts";

import type { FormatEdits } from "../../../../bindings/FormatEdits";
import type { FormatScope } from "../../../../bindings/FormatScope";
import type { RecordingTimelineEdit } from "../../timeline/editing/recording-timeline-edit";

// What a scene, annotation, song or sound effect, or the caption track
// offers in its right-click menu to set it apart by format: a tick for
// each format it shows in, one pressed showing or leaving it out there;
// and, for what has a layout, whether it is custom in the shape in hand,
// laid out there its own way whatever changes in the others, with, while
// it is, its layout made the one every shape uses.

/** What an item's menu offers: whether it is custom in the shape in hand,
 * which formats it shows in, and making its layout every shape's. */
export type FormatSync = {
  /** Whether it is laid out per framing, as a scene is, shared by the
   * platforms sharing one; else per proportion, as an annotation is. */
  byFraming: boolean;
  /** Whether its layout here can be made every shape's. */
  canSpread: boolean;
  /** Null where it has no layout to set apart, or there is one shape. */
  custom: boolean | null;
  /** Every format, in the bar's order, ticked where it shows. */
  formats: readonly { id: string; isShown: boolean }[];
};

/** What a menu pick asks: the item made custom here or not, shown in a
 * format or left out of it, or its layout made every shape's. */
export type FormatSyncAction =
  | { kind: "custom" }
  | { kind: "layout-all" }
  | { format: string; kind: "show" };

type ItemKind = "annotation" | "audio" | "captions" | "scene";

const TABLES = {
  annotation: "annotations",
  audio: "audio",
  scene: "scenes",
} as const;

const kindOf = (edit: RecordingTimelineEdit, id: string): ItemKind | null => {
  if (id === CAPTIONS_ID) return edit.captions?.enabled ? "captions" : null;
  if (edit.sceneClips?.some((clip) => clip.id === id)) return "scene";
  if (edit.annotationClips?.some((clip) => clip.annotation.id === id))
    return "annotation";
  if (edit.audioClips?.some((clip) => clip.id === id)) return "audio";
  return null;
};

const formatsOf = (
  edits: FormatEdits,
  kind: ItemKind,
  id: string,
): Formats<unknown> | undefined =>
  kind === "captions" ? edits.captions : edits[TABLES[kind]]?.[id];

/** What `id`, in the shared edit `edit`, offers in `context`'s format;
 * null for an item the edit does not hold, or with one format. */
export function formatSync({
  context,
  edit,
  edits,
  id,
}: {
  context: FormatContext;
  edit: RecordingTimelineEdit;
  edits: FormatEdits;
  id: string;
}): FormatSync | null {
  const kind = kindOf(edit, id);
  const every = allFormats(context);
  if (every.length < 2 || !kind) return null;
  const formats = formatsOf(edits, kind, id);
  // Scenes are laid out per framing; annotations and captions per
  // proportion, on the screen whatever its framing.
  const key = kind === "scene" ? context.shape : context.annotationKey;
  const isCustom = formats?.layouts?.[key] !== undefined;
  return {
    byFraming: kind === "scene",
    canSpread:
      isCustom ||
      (context.shapes[0]?.id === context.shape &&
        Object.keys(formats?.layouts ?? {}).length > 0),
    custom: kind === "audio" || context.shapes.length < 2 ? null : isCustom,
    formats: every.map((format) => ({
      id: format,
      isShown: isShownIn(formats?.showIn, { ...context, id: format }),
    })),
  };
}

/** `edits` with `id` shown in `format`, or left out of it where it shows
 * there; left out of the only format it shows in, it would show nowhere,
 * which is deleting it, so that is left as it is. */
export function withShownToggled(
  format: string,
  {
    context,
    edit,
    edits,
    id,
  }: {
    context: FormatContext;
    edit: RecordingTimelineEdit;
    edits: FormatEdits;
    id: string;
  },
): FormatEdits {
  const kind = kindOf(edit, id);
  if (!kind) return edits;
  const was = formatsOf(edits, kind, id)?.showIn;
  const isShown = isShownIn(was, { ...context, id: format });
  const left = isShown
    ? withoutHere(was, [format], context)
    : withHere(was, [format], context);
  if (left === null) return edits;
  return withShowIn(edits, { id, kind }, left);
}

/** `edits` with `id` custom in the shape in hand, laid out there as it is
 * now (`shown`, the timeline the shape in hand plays), or no longer, its
 * layout there given back to the shared one. */
export function withCustomToggled({
  context,
  edits,
  id,
  shown,
}: {
  context: FormatContext;
  edits: FormatEdits;
  id: string;
  shown: RecordingTimelineEdit;
}): FormatEdits {
  const shape = shown.sceneClips?.some((clip) => clip.id === id)
    ? context.shape
    : context.annotationKey;
  const own = (formats: Formats<unknown> | undefined) =>
    formats?.layouts?.[shape] !== undefined;
  if (id === CAPTIONS_ID) {
    if (own(edits.captions)) return withoutItemLayouts(edits, id, [shape]);
    if (!shown.captions) return edits;
    const look = {
      style: shown.captions.style,
      ...(shown.captions.layout ? { layout: shown.captions.layout } : {}),
    };
    return {
      ...edits,
      captions: {
        ...edits.captions,
        layouts: { ...edits.captions?.layouts, [shape]: look },
      },
    };
  }
  const scene = shown.sceneClips?.find((clip) => clip.id === id);
  if (scene) {
    if (own(edits.scenes?.[id])) return withoutItemLayouts(edits, id, [shape]);
    const formats = edits.scenes?.[id];
    return {
      ...edits,
      scenes: {
        ...edits.scenes,
        [id]: {
          ...formats,
          layouts: { ...formats?.layouts, [shape]: sceneLayoutOf(scene) },
        },
      },
    };
  }
  const annotation = shown.annotationClips?.find(
    (clip) => clip.annotation.id === id,
  );
  if (!annotation) return edits;
  if (own(edits.annotations?.[id]))
    return withoutItemLayouts(edits, id, [shape]);
  const formats = edits.annotations?.[id];
  return {
    ...edits,
    annotations: {
      ...edits.annotations,
      [id]: {
        ...formats,
        layouts: { ...formats?.layouts, [shape]: annotation.annotation },
      },
    },
  };
}

/** `edits` with `id`, of `kind`, showing in `showIn`. */
const withShowIn = (
  edits: FormatEdits,
  { id, kind }: { id: string; kind: ItemKind },
  showIn: FormatScope | undefined,
): FormatEdits => {
  switch (kind) {
    case "captions": {
      const { captions, ...rest } = edits;
      const now = reshown(captions, showIn);
      return now ? { ...rest, captions: now } : rest;
    }
    case "scene":
      return { ...edits, scenes: withEntry(edits.scenes, id, showIn) };
    case "annotation":
      return {
        ...edits,
        annotations: withEntry(edits.annotations, id, showIn),
      };
    case "audio":
      return { ...edits, audio: withEntry(edits.audio, id, showIn) };
  }
};

/** `formats` showing in `showIn`, everything else kept; nothing where
 * nothing is left. */
const reshown = <T extends Formats<unknown>>(
  formats: T | undefined,
  showIn: FormatScope | undefined,
): T | undefined => {
  const { showIn: _was, ...rest } = formats ?? ({} as T);
  const now = (showIn === undefined ? rest : { ...rest, showIn }) as T;
  return tidy(now) ? now : undefined;
};

/** `table` with `id`'s entry showing in `showIn`. */
const withEntry = <T extends Formats<unknown>>(
  table: Partial<Record<string, T>> | undefined,
  id: string,
  showIn: FormatScope | undefined,
): Record<string, T> => {
  const now = reshown(table?.[id], showIn);
  const { [id]: _was, ...rest } = table ?? {};
  return defined(now ? { ...rest, [id]: now } : rest);
};
