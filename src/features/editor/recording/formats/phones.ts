// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../../i18n/i18n";

import type { Phone } from "../../../../bindings/Phone";

/** The phones a tall format can be previewed on, in the order offered. */
export const PHONES: Phone[] = [
  "iphone-17-pro",
  "iphone-17-pro-max",
  "iphone-se",
  "pixel-10",
  "galaxy-s26",
  "galaxy-fold-cover",
];

export const phoneName = (phone: Phone) => {
  switch (phone) {
    case "iphone-17-pro":
      return t("editor-frame-phone-iphone-17-pro");
    case "iphone-17-pro-max":
      return t("editor-frame-phone-iphone-17-pro-max");
    case "iphone-se":
      return t("editor-frame-phone-iphone-se");
    case "pixel-10":
      return t("editor-frame-phone-pixel-10");
    case "galaxy-s26":
      return t("editor-frame-phone-galaxy-s26");
    case "galaxy-fold-cover":
      return t("editor-frame-phone-galaxy-fold-cover");
  }
};
