// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";

/** Brings the island out to offer an update, if it is free; resolves to
 * whether it came out. */
export const presentUpdateIsland = () =>
  invoke<boolean>("present_update_island");

/** Tucks the island away after an update was put off or skipped. */
export const withdrawUpdateIsland = () =>
  invoke<null>("withdraw_update_island");

export const updateIslandShown = () => invoke<boolean>("update_island_shown");
