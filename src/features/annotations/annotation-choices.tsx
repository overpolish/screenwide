// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { AlignCenter, AlignLeft, AlignRight } from "lucide-react";

import { t } from "../../i18n/i18n";
import { type PillGroupItem } from "../../primitives/pill-group/pill-group";

import { AnnotationAlign, AnnotationHead } from "./types";

// One way an annotation can be dressed, offered as a segmented choice: its
// name, its segments, and how the value the annotation stores maps to a
// segment and back. Several store a boolean or a number rather than the
// segment's id, so the mapping lives with the choice and not at each place
// the choice is offered. The labels are read when the choice is drawn, so
// they follow the app's language.
export type AnnotationChoiceDefinition<Value> = {
  items: () => PillGroupItem[];
  label: () => string;
  pick: (id: string) => Value;
  select: (value: Value) => string;
};

// A choice whose stored value is the segment's own id.
const named = <Value extends string>(
  label: () => string,
  items: () => PillGroupItem[],
): AnnotationChoiceDefinition<Value> => ({
  items,
  label,
  pick: (id) => id as Value,
  select: (value) => value,
});

// A choice between two segments stored as a boolean: `on` is the segment
// for true.
const either = (
  label: () => string,
  off: { id: string; label: () => string },
  on: { id: string; label: () => string },
): AnnotationChoiceDefinition<boolean> => ({
  items: () => [
    { id: off.id, label: off.label() },
    { id: on.id, label: on.label() },
  ],
  label,
  pick: (id) => id === on.id,
  select: (value) => (value ? on.id : off.id),
});

/** Which ends of an arrow carry a head. */
export const headChoice = named<AnnotationHead>(
  () => t("annotation-head"),
  () => [
    { id: "none", label: t("annotation-head-none") },
    { id: "end", label: t("annotation-head-end") },
    { id: "both", label: t("annotation-head-both") },
  ],
);

/** How a text box lines up its lines. */
export const alignChoice = named<AnnotationAlign>(
  () => t("annotation-alignment"),
  () => [
    {
      ariaLabel: t("annotation-align-left"),
      icon: <AlignLeft aria-hidden="true" />,
      id: "left",
    },
    {
      ariaLabel: t("annotation-align-center"),
      icon: <AlignCenter aria-hidden="true" />,
      id: "center",
    },
    {
      ariaLabel: t("annotation-align-right"),
      icon: <AlignRight aria-hidden="true" />,
      id: "right",
    },
  ],
);

/** Which pixelation a pixelated redaction draws: secure blocks laid out by
 * its seed, or classic blocks that each show the average under them. */
export const pixelationChoice = named<"pixelate" | "pixelateClassic">(
  () => t("annotation-pixelation"),
  () => [
    { id: "pixelate", label: t("annotation-pixelation-secure") },
    { id: "pixelateClassic", label: t("annotation-pixelation-classic") },
  ],
);

/** Whether a highlight recolours the page it reads, turning the page into
 * its colour and the text into ink that reads on it, or tints what it covers
 * the way a felt marker does, for content the recolouring misreads. True is
 * tint. */
export const inkChoice = either(
  () => t("annotation-ink"),
  { id: "recolor", label: () => t("annotation-ink-recolor") },
  { id: "tint", label: () => t("annotation-ink-tint") },
);

/** Whether a highlight fits itself to the text under it or is laid by hand
 * over the box its drag spans. True is by hand. */
export const fitChoice = either(
  () => t("annotation-fit"),
  { id: "auto", label: () => t("annotation-fit-auto") },
  { id: "manual", label: () => t("annotation-fit-manual") },
);
