// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { type RunningApp } from "../api";

/** Stand-ins for the apps running, without their icons, for previews that
 * have no apps of their own to offer. */
export const SAMPLE_RUNNING_APPS: RunningApp[] = [
  { iconPath: null, id: "com.apple.Safari", label: "Safari" },
  { iconPath: null, id: "com.tinyspeck.slackmacgap", label: "Slack" },
  { iconPath: null, id: "com.apple.Notes", label: "Notes" },
];
