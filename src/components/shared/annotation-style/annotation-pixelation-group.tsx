// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../i18n/i18n";
import { PillGroup } from "../../base/pill-group/pill-group";

/** Which pixelation a pixelated redaction draws: secure blocks laid out by
 * its seed, or classic blocks that each show the average under them. */
export function AnnotationPixelationGroup({
  isDisabled,
  onChange,
  value,
}: {
  onChange: (redaction: "pixelate" | "pixelateClassic") => void;
  value: "pixelate" | "pixelateClassic";
  isDisabled?: boolean;
}) {
  return (
    <PillGroup
      aria-label={t("annotation-pixelation")}
      display="label"
      isDisabled={isDisabled}
      items={[
        { id: "pixelate", label: t("annotation-pixelation-secure") },
        { id: "pixelateClassic", label: t("annotation-pixelation-classic") },
      ]}
      onSelectionChange={(id) => {
        onChange(id as "pixelate" | "pixelateClassic");
      }}
      selected={value}
    />
  );
}
