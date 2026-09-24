// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { AlignCenter, AlignLeft, AlignRight } from "lucide-react";
import { ReactNode } from "react";

import { PillGroup } from "../../base/pill-group/pill-group";

import { AnnotationAlign } from "./types";

const items: { icon: ReactNode; id: AnnotationAlign; label: string }[] = [
  { icon: <AlignLeft aria-hidden="true" />, id: "left", label: "Left" },
  { icon: <AlignCenter aria-hidden="true" />, id: "center", label: "Centre" },
  { icon: <AlignRight aria-hidden="true" />, id: "right", label: "Right" },
];

/** How a text box lines up its lines. */
export function AnnotationAlignGroup({
  isDisabled,
  onChange,
  value,
}: {
  onChange: (align: AnnotationAlign) => void;
  value: AnnotationAlign;
  isDisabled?: boolean;
}) {
  return (
    <PillGroup
      aria-label="Alignment"
      isDisabled={isDisabled}
      items={items}
      onSelectionChange={(id) => {
        onChange(id as AnnotationAlign);
      }}
      selected={value}
    />
  );
}
