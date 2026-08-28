// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { PermissionSnapshot } from "./types";

export const permissionsPreviewSnapshot: PermissionSnapshot = {
  accessibility: { canRequest: false, granted: true },
  camera: { canRequest: true, granted: false },
  microphone: { canRequest: false, granted: false },
  screenRecording: { canRequest: false, granted: true },
};
