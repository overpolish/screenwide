// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../i18n/i18n";
import { PillGroup } from "../../base/pill-group/pill-group";

const items = (): { id: "clean" | "hand"; label: string }[] => [
  { id: "clean", label: t("annotation-stroke-clean") },
  { id: "hand", label: t("annotation-stroke-hand-drawn") },
];

/** Whether a highlight or a shape is drawn clean or as a pen stroke by hand,
 * labelled for a toolbar that has no room for a row heading. */
export function AnnotationStrokeGroup({
  isDisabled,
  onChange,
  value,
}: {
  onChange: (handDrawn: boolean) => void;
  value: boolean;
  isDisabled?: boolean;
}) {
  return (
    <PillGroup
      aria-label={t("annotation-stroke")}
      display="label"
      isDisabled={isDisabled}
      items={items()}
      onSelectionChange={(id) => {
        onChange(id === "hand");
      }}
      selected={value ? "hand" : "clean"}
    />
  );
}
