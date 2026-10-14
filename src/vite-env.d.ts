// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/// <reference types="vite/client" />

interface ImportMetaEnv {
  /** When the bundle was built, set by `vite.config.ts`: tells what this
   * build stored from what an earlier one left behind. */
  readonly SCREENWIDE_BUILD?: string;
}
