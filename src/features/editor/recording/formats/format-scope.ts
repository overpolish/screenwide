// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { FormatEdits } from "../../../../bindings/FormatEdits";
import type { FormatScope } from "../../../../bindings/FormatScope";

/** The id the caption track goes by among the items a format can leave
 * out or lay out its own way. */
export const CAPTIONS_ID = "captions";

/** The format a picture is worked out for, among every shape and format. */
export type FormatContext = {
  /** The key annotation and caption layouts are kept under: shared by
   * every platform of one proportion, as they sit on the screen whatever
   * its framing; the main canvas's own shape id on the main canvas. */
  annotationKey: string;
  /** The format worked out for: a platform's template, or the shape's id
   * for a shape made for none. */
  id: string;
  /** The shape worked out for, which its scene layouts are kept under. */
  shape: string;
  /** Every shape, by id, with its formats by id and its annotation key. */
  shapes: readonly {
    annotationKey: string;
    formats: readonly string[];
    id: string;
  }[];
};

/** The annotation key of a platform canvas `size` big: its proportions. */
export const proportionKey = ({
  height,
  width,
}: {
  height: number;
  width: number;
}) => {
  const divisor = (one: number, other: number): number =>
    other === 0 ? one : divisor(other, one % other);
  const common = divisor(Math.round(width), Math.round(height)) || 1;
  return `ratio-${String(Math.round(width) / common)}x${String(Math.round(height) / common)}`;
};

/** The keys a format's own layouts are kept under: its shape's, for its
 * scenes, and its proportion's, for its annotations and captions. */
export const layoutKeys = (context: FormatContext) =>
  context.annotationKey === context.shape
    ? [context.shape]
    : [context.shape, context.annotationKey];

/** Every key any format keeps layouts under. */
export const everyLayoutKey = (context: FormatContext) => [
  ...new Set(
    context.shapes.flatMap((shape) => [shape.id, shape.annotationKey]),
  ),
];

/** `context` as annotations and captions are written in: their layouts
 * kept under its proportion's key. */
export const annotationContext = (context: FormatContext): FormatContext => ({
  ...context,
  shape: context.annotationKey,
});

/** Where a layout changed in the shape in hand lands: in the shared layout
 * every shape follows, as it does in the main shape, or in the shape's own,
 * as in any other. */
export type EditScope = "all" | "shape";

/** Every format, shape by shape. */
export const allFormats = (context: FormatContext) =>
  context.shapes.flatMap((shape) => shape.formats);

/** The key a scope keeps its own layouts under; none for the shared layout
 * every shape starts from. */
export const layoutKey = (scope: EditScope, context: FormatContext) =>
  scope === "shape" && context.shapes.length > 1 ? context.shape : null;

/** Whether something set to show in `scope` shows in `context`'s format. */
export const isShownIn = (
  scope: FormatScope | undefined,
  context: FormatContext,
) =>
  scope === undefined || scope === "all" || scope.formats.includes(context.id);

/** The formats `scope` shows in, by id. */
export const shownFormats = (
  scope: FormatScope | undefined,
  context: FormatContext,
): string[] =>
  allFormats(context).filter((id) => isShownIn(scope, { ...context, id }));

/** `shown` as a scope: nothing where it is every format. */
const scopeOf = (
  shown: readonly string[],
  context: FormatContext,
): FormatScope | undefined =>
  allFormats(context).every((id) => shown.includes(id))
    ? undefined
    : { formats: [...shown] };

/** `scope` with the formats `leaving` taken out of it, as removing
 * something from them leaves it in the rest; null where it shows nowhere
 * then, and undefined where it still shows everywhere. */
export const withoutHere = (
  scope: FormatScope | undefined,
  leaving: readonly string[],
  context: FormatContext,
): FormatScope | null | undefined => {
  const left = shownFormats(scope, context).filter(
    (id) => !leaving.includes(id),
  );
  return left.length === 0 ? null : scopeOf(left, context);
};

/** `scope` with the formats `joining` put back in it. */
export const withHere = (
  scope: FormatScope | undefined,
  joining: readonly string[],
  context: FormatContext,
): FormatScope | undefined =>
  scopeOf([...new Set([...shownFormats(scope, context), ...joining])], context);

/** `scope` once the format `added` has been made: a scope naming the
 * formats something shows in names it too, as it was set apart only from
 * those taken out of it then. */
export const withFormatAdded = (
  scope: FormatScope | undefined,
  added: string,
): FormatScope | undefined =>
  scope === undefined || scope === "all" || scope.formats.includes(added)
    ? scope
    : { formats: [...scope.formats, added] };

/** `formats` once the format `added` has been made: every scene,
 * annotation, song, sound effect and the captions their edits show in only
 * some formats show in it too (`withFormatAdded`). */
export const withEditsFormatAdded = <Formats extends { edits?: FormatEdits }>(
  formats: Formats | null,
  added: string,
): Formats | null => {
  if (!formats?.edits) return formats;
  const reshown = <T extends { showIn?: FormatScope }>(entry: T): T => {
    const showIn = withFormatAdded(entry.showIn, added);
    return showIn === entry.showIn ? entry : { ...entry, showIn };
  };
  const table = <T extends { showIn?: FormatScope }>(
    entries: Record<string, T>,
  ): Record<string, T> =>
    Object.fromEntries(
      Object.entries(entries).map(([id, entry]) => [id, reshown(entry)]),
    );
  const { annotations, audio, captions, scenes } = formats.edits;
  return {
    ...formats,
    edits: {
      ...formats.edits,
      ...(scenes ? { scenes: table(scenes) } : {}),
      ...(annotations ? { annotations: table(annotations) } : {}),
      ...(audio ? { audio: table(audio) } : {}),
      ...(captions ? { captions: reshown(captions) } : {}),
    },
  };
};
