// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { AlignCenter, AlignLeft, AlignRight } from "lucide-react";
import { ReactNode } from "react";

import { t } from "../../i18n/i18n";
import { PillGroup } from "../../primitives/pill-group/pill-group";

import { AnnotationAlign } from "./types";

const items = (): {
  ariaLabel: string;
  icon: ReactNode;
  id: AnnotationAlign;
}[] => [
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
      aria-label={t("annotation-alignment")}
      isDisabled={isDisabled}
      items={items()}
      onSelectionChange={(id) => {
        onChange(id as AnnotationAlign);
      }}
      selected={value}
    />
  );
}
