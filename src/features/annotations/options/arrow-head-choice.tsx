// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { PillGroup } from "../../../primitives/pill-group/pill-group";
import { headChoice } from "../annotation-choices";

import { ArrowHeadGlyph } from "./arrow-head-glyph";

import type { AnnotationHead } from "../../../bindings/AnnotationHead";

/** An arrow's heads, drawn in the text's colour as the tool's icon is: the
 * colour in hand shows elsewhere already, and a dark one would hide them. */
export function ArrowHeadChoice({
  onChange,
  value,
}: {
  onChange: (head: AnnotationHead) => void;
  value: AnnotationHead;
}) {
  return (
    <PillGroup
      aria-label={headChoice.label()}
      items={headChoice.items().map((item) => ({
        ariaLabel: item.label ?? item.ariaLabel,
        icon: (
          <ArrowHeadGlyph
            className="size-icon-lg"
            head={item.id as AnnotationHead}
          />
        ),
        id: item.id,
      }))}
      onSelectionChange={(id) => {
        onChange(headChoice.pick(id));
      }}
      selected={headChoice.select(value)}
    />
  );
}
