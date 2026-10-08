// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { AlignCenter, AlignLeft, AlignRight } from "lucide-react";
import { ReactNode } from "react";

import { t } from "../../../i18n/i18n";
import { PillGroup } from "../../base/pill-group/pill-group";

import { AnnotationAlign } from "./types";

const items = (): {
  icon: ReactNode;
  id: AnnotationAlign;
  label: string;
}[] => [
  {
    icon: <AlignLeft aria-hidden="true" />,
    id: "left",
    label: t("annotation-align-left"),
  },
  {
    icon: <AlignCenter aria-hidden="true" />,
    id: "center",
    label: t("annotation-align-center"),
  },
  {
    icon: <AlignRight aria-hidden="true" />,
    id: "right",
    label: t("annotation-align-right"),
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
