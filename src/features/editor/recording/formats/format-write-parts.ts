// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { CAPTIONS_ID, layoutKey } from "./format-scope";

import type { EditScope, FormatContext } from "./format-scope";
import type { FormatEdits } from "../../../../bindings/FormatEdits";
import type { FormatScope } from "../../../../bindings/FormatScope";
import type { RecordingTimelineEdit } from "../../timeline/editing/recording-timeline-edit";

/** What one scene, annotation or caption track does differently: where it
 * shows, and its layout by the key of the formats it is laid out for. */
export type Formats<Layout> = {
  layouts?: Record<string, Layout>;
  showIn?: FormatScope;
};

/** A change to the format in hand's timeline to write back: `previous` to
 * `next`, into the one shared edit `stored` and what the formats do
 * differently, `edits`, where `scope` says. */
export type Write = {
  context: FormatContext;
  edits: FormatEdits;
  next: RecordingTimelineEdit;
  previous: RecordingTimelineEdit;
  scope: EditScope;
  stored: RecordingTimelineEdit;
};

/** `table` without the entries left empty. */
export const defined = <T>(table: Record<string, T | undefined>) =>
  Object.fromEntries(
    Object.entries(table).filter(
      (entry): entry is [string, T] => entry[1] !== undefined,
    ),
  );

export const same = (left: unknown, right: unknown) =>
  JSON.stringify(left) === JSON.stringify(right);

/** `formats` cleared of empty parts, or nothing where nothing is left. */
export const tidy = <Layout>(
  formats: Formats<Layout>,
): Formats<Layout> | undefined => {
  const layouts = Object.keys(formats.layouts ?? {}).length
    ? formats.layouts
    : undefined;
  if (!layouts && formats.showIn === undefined) return undefined;
  return {
    ...(layouts ? { layouts } : {}),
    ...(formats.showIn === undefined ? {} : { showIn: formats.showIn }),
  };
};

/** Where a layout changed for an item with `formats` lands: in the shape
 * in hand's own, where the item is custom there, whichever shape that is;
 * else where `scope` says. */
export const itemLayoutKey = (
  formats: Formats<unknown> | undefined,
  scope: EditScope,
  context: FormatContext,
) =>
  formats?.layouts?.[context.shape] !== undefined
    ? context.shape
    : layoutKey(scope, context);

/** `formats` with `key`'s layout set to `layout`; for the shared layout
 * (`key` null), changed in the main shape `own`, with that shape's own
 * taken away so the shared one shows there, and every other shape's own
 * kept, as those shapes were laid out apart. */
export const withLayout = <Layout>(
  formats: Formats<Layout> | undefined,
  { key, layout, own }: { key: string | null; layout: Layout; own: string },
): Formats<Layout> => {
  if (key !== null)
    return { ...formats, layouts: { ...formats?.layouts, [key]: layout } };
  const { [own]: _own, ...others } = formats?.layouts ?? {};
  return { ...formats, layouts: others };
};

/** The fields of `after` that differ from `before`, outside `skipped`. */
export const changed = <T extends object>(
  before: T,
  after: T,
  skipped: readonly string[],
): Partial<T> => {
  const keys = new Set([...Object.keys(before), ...Object.keys(after)]);
  const out: Partial<T> = {};
  for (const key of keys) {
    if (skipped.includes(key)) continue;
    const value = (after as Record<string, unknown>)[key];
    if (!same((before as Record<string, unknown>)[key], value))
      Object.assign(out, { [key]: value });
  }
  return out;
};

/** `stored` with `fields` applied, an undefined field taken away. */
export const applied = <T extends object>(stored: T, fields: Partial<T>): T => {
  const next = { ...stored, ...fields };
  for (const [key, value] of Object.entries(fields))
    if (value === undefined) Reflect.deleteProperty(next, key);
  return next;
};

const withoutKey = <Layout>(
  formats: Formats<Layout>,
  key: string,
): Formats<Layout> | undefined => {
  const { [key]: _gone, ...layouts } = formats.layouts ?? {};
  return tidy({ ...formats, layouts });
};

/** `edits` with every layout made for `key` taken away, as when its shape
 * goes; where things show is kept. */
export const withoutLayouts = (
  edits: FormatEdits,
  key: string,
): FormatEdits => {
  const each = <Layout>(table: Record<string, Formats<Layout>> = {}) =>
    defined(
      Object.fromEntries(
        Object.entries(table).map(([id, formats]) => [
          id,
          withoutKey(formats, key),
        ]),
      ),
    );
  const { captions: _captions, ...rest } = edits;
  const captions = edits.captions && withoutKey(edits.captions, key);
  return {
    ...rest,
    annotations: each(edits.annotations),
    scenes: each(edits.scenes),
    ...(captions ? { captions } : {}),
  };
};

/** `edits` with the scene, annotation or caption track `id`'s layouts for
 * `keys` taken away, so the layout it shares shows for it again. */
export const withoutItemLayouts = (
  edits: FormatEdits,
  id: string,
  keys: readonly string[],
): FormatEdits => {
  const kept = <Layout>(formats: Formats<Layout>) =>
    keys.reduce<Formats<Layout> | undefined>(
      (left, key) => left && withoutKey(left, key),
      formats,
    );
  if (id === CAPTIONS_ID) {
    const { captions, ...rest } = edits;
    const left = captions && kept(captions);
    return left ? { ...rest, captions: left } : rest;
  }
  const each = <Layout>(table: Record<string, Formats<Layout>> = {}) => {
    const formats = table[id] as Formats<Layout> | undefined;
    return formats ? defined({ ...table, [id]: kept(formats) }) : table;
  };
  return {
    ...edits,
    annotations: each(edits.annotations),
    scenes: each(edits.scenes),
  };
};
