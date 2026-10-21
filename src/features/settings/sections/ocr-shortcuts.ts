// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../i18n/i18n";

import type { OcrAction } from "../../../bindings/OcrAction";

// The keys that work while recognised text is showing.
export const ocrShortcuts = (): {
  action: OcrAction;
  description: string;
  label: string;
}[] => [
  {
    action: "selectAll",
    description: t("settings-ocr-select-all-description"),
    label: t("settings-ocr-select-all"),
  },
  {
    action: "copyText",
    description: t("settings-ocr-copy-description"),
    label: t("settings-ocr-copy"),
  },
];
