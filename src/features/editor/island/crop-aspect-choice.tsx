// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../i18n/i18n";
import { PillGroup } from "../../../primitives/pill-group/pill-group";

/** The shape a crop keeps: none, the whole capture's, or a ratio of width to
 * height. */
export type CropAspect = "16:9" | "1:1" | "4:3" | "9:16" | "free" | "original";

/** The ratios offered, widest first after the square, as width to height. */
const RATIOS = [
  { height: 1, id: "1:1", width: 1 },
  { height: 3, id: "4:3", width: 4 },
  { height: 9, id: "16:9", width: 16 },
  { height: 16, id: "9:16", width: 9 },
] as const;

/** The shape a layer's crop keeps as its handles are dragged on the picture,
 * the screen's or the camera's alike. How much it keeps is dragged, where a
 * size can be seen rather than read as a number. */
export function CropAspectChoice({
  onChange,
  value,
}: {
  onChange: (aspect: CropAspect) => void;
  value: CropAspect;
}) {
  return (
    <PillGroup
      aria-label={t("editor-island-crop-aspect")}
      items={[
        { id: "free", label: t("editor-island-crop-free") },
        { id: "original", label: t("editor-island-crop-original") },
        ...RATIOS.map(({ height, id, width }) => ({
          ariaLabel: t("editor-island-crop-ratio-name", { height, width }),
          id,
          label: t("editor-island-crop-ratio", { height, width }),
        })),
      ]}
      onSelectionChange={(aspect) => {
        onChange(aspect as CropAspect);
      }}
      selected={value}
    />
  );
}
