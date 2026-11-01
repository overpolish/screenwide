// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { AlignCenter, AlignLeft, AlignRight } from "lucide-react";

import { t } from "../../i18n/i18n";
import { type PillGroupItem } from "../../primitives/pill-group/pill-group";

import { AnnotationAlign, AnnotationHead } from "./types";

// One way an annotation can be dressed, offered as a segmented choice: its
// name, its segments, and how the value the annotation stores maps to a
// segment and back, so the mapping lives with the choice and not at each
// place the choice is offered. The labels are read when the choice is drawn,
// so they follow the app's language.
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
