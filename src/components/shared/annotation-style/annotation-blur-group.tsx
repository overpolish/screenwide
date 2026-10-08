// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../i18n/i18n";
import { PillGroup } from "../../base/pill-group/pill-group";

const items = (): { id: "sharp" | "blur"; label: string }[] => [
  { id: "sharp", label: t("annotation-outside-sharp") },
  { id: "blur", label: t("annotation-outside-blur") },
];

/** Whether what lies outside a spotlight is left sharp or blurred, labelled
 * for a toolbar that has no room for a row heading. */
export function AnnotationBlurGroup({
  isDisabled,
  onChange,
  value,
}: {
  onChange: (blur: boolean) => void;
  value: boolean;
  isDisabled?: boolean;
}) {
  return (
    <PillGroup
      aria-label={t("annotation-outside")}
      display="label"
      isDisabled={isDisabled}
      items={items()}
      onSelectionChange={(id) => {
        onChange(id === "blur");
      }}
      selected={value ? "blur" : "sharp"}
    />
  );
}
