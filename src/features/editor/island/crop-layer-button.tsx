// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Crop } from "lucide-react";

import { t } from "../../../i18n/i18n";
import { IconButton } from "../../../primitives/button/icon-button";

// The crop of the layer in hand, at the end of the Screen, Screenshot and
// Camera islands' row: the crop is the chosen layer's, not a tool of the
// column, as Descript's crop sits on the chosen layer's toolbar. A
// double-click on the layer and K take it up too.

export function CropLayerButton({ onPress }: { onPress: () => void }) {
  return (
    <IconButton
      aria-label={t("editor-island-crop-layer")}
      className="text-fg-secondary enabled:hover:text-fg"
      onPress={onPress}
      variant="ghost"
    >
      <Crop aria-hidden className="size-icon" />
    </IconButton>
  );
}
