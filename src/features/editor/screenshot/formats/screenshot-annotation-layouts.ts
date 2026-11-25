// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { ScreenshotFormats } from "../../../../bindings/ScreenshotFormats";
import type { ScreenshotWorkspaceOutputSettings } from "../screenshot-output";

// An annotation of a screenshot is placed its own way on the platforms of
// one proportion as a recording's is (`format-sync.ts`): moved, resized or
// restyled with such a platform in hand, it is custom there, kept under the
// proportion's key while the main canvas, and every other proportion,
// keeps its own. What is drawn stays the main canvas's: an annotation
// added, taken away or put in another place in the stack on a platform is
// added, taken away or moved on the main canvas.

type Workspace = ScreenshotWorkspaceOutputSettings;
type Annotation = Workspace["items"][number]["output"]["annotations"][number];
/** Annotations placed their own way, by id, then by proportion. */
export type Layouts = NonNullable<ScreenshotFormats["layouts"]>;

const same = (one: Annotation, other: Annotation) =>
  JSON.stringify(one) === JSON.stringify(other);

/** `workspace` with each annotation placed as `layouts` place it under
 * `key`, the proportion of the canvas it is laid out on. */
export const withLayouts = (
  workspace: Workspace,
  { key, layouts }: { key: string; layouts: Layouts | undefined },
): Workspace => {
  if (!layouts || !Object.values(layouts).some((by) => key in by))
    return workspace;
  // Most annotations have no layout of their own.
  const own: Partial<Record<string, Partial<Record<string, Annotation>>>> =
    layouts;
  return {
    ...workspace,
    items: workspace.items.map((item) => ({
      ...item,
      output: {
        ...item.output,
        annotations: item.output.annotations.map(
          (annotation) => own[annotation.id]?.[key] ?? annotation,
        ),
      },
    })),
  };
};

/** A layer's annotations on the main canvas, `main`, with what changed on
 * a platform from `before` to `after`: added, taken away or put in another
 * place in the stack there, as the main canvas has them; and `own`, the
 * annotations changed there, which are placed their own way on it. */
export const mergedAnnotations = (
  main: readonly Annotation[],
  {
    after,
    before,
  }: { after: readonly Annotation[]; before: readonly Annotation[] },
) => {
  const mains = new Map(main.map((annotation) => [annotation.id, annotation]));
  const befores = new Map(
    before.map((annotation) => [annotation.id, annotation]),
  );
  const own: Annotation[] = [];
  const merged = after.map((annotation) => {
    const was = befores.get(annotation.id);
    const home = mains.get(annotation.id);
    if (!was || !home) return annotation;
    if (!same(was, annotation)) own.push(annotation);
    return home;
  });
  return { merged, own };
};

/** `layouts` with `own`, annotations changed on a platform of the
 * proportion `key`, placed their own way there. */
export const withOwnLayouts = (
  layouts: Layouts | undefined,
  { key, own }: { key: string; own: readonly Annotation[] },
): Layouts | undefined => {
  if (own.length === 0) return layouts;
  const next = { ...layouts };
  for (const annotation of own)
    next[annotation.id] = { ...next[annotation.id], [key]: annotation };
  return next;
};

/** `layouts` without `id`'s own placement under `key`, or under any key
 * where `key` is null. */
export const withoutLayout = (
  layouts: Layouts | undefined,
  { id, key }: { id: string; key: string | null },
): Layouts | undefined => {
  const own = layouts?.[id];
  if (!own) return layouts;
  const left = Object.fromEntries(
    Object.entries(own).filter(([each]) => key !== null && each !== key),
  );
  const rest = Object.fromEntries(
    Object.entries(layouts).filter(([each]) => each !== id),
  );
  return Object.keys(left).length > 0 ? { ...rest, [id]: left } : rest;
};

/** The annotation `id` as `workspace` has it, wherever it is drawn. */
export const annotationIn = (workspace: Workspace, id: string) =>
  workspace.items
    .flatMap((item) => item.output.annotations)
    .find((annotation) => annotation.id === id);

/** `workspace` with the annotation `annotation` in place of its own. */
export const withAnnotation = (
  workspace: Workspace,
  annotation: Annotation,
): Workspace => ({
  ...workspace,
  items: workspace.items.map((item) =>
    item.output.annotations.some((each) => each.id === annotation.id)
      ? {
          ...item,
          output: {
            ...item.output,
            annotations: item.output.annotations.map((each) =>
              each.id === annotation.id ? annotation : each,
            ),
          },
        }
      : item,
  ),
});
