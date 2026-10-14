// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/** Finished capabilities kept out of the interface until they are wanted.
 * Turning one on shows it; the code behind it is built and tested either
 * way, so each is typed as a switch rather than as its current value. */
export const FEATURES: Readonly<{
  /** Importing, exporting and removing caption templates as packages, from
   * the template's menu. */
  captionTemplateFiles: boolean;
}> = {
  captionTemplateFiles: false,
};
