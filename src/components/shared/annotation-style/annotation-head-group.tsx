// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../i18n/i18n";
import { PillGroup } from "../../base/pill-group/pill-group";

import { AnnotationHead } from "./types";

const items = (): { id: AnnotationHead; label: string }[] => [
  { id: "none", label: t("annotation-head-none") },
  { id: "end", label: t("annotation-head-end") },
  { id: "both", label: t("annotation-head-both") },
];

/** Which ends of an arrow carry a head. */
export function AnnotationHeadGroup({
  isDisabled,
  onChange,
  value,
}: {
  onChange: (head: AnnotationHead) => void;
  value: AnnotationHead;
  isDisabled?: boolean;
}) {
  return (
    <PillGroup
      aria-label={t("annotation-head")}
      display="label"
      isDisabled={isDisabled}
      items={items()}
      onSelectionChange={(id) => {
        onChange(id as AnnotationHead);
      }}
      selected={value}
    />
  );
}
