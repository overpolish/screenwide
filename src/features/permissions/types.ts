// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { PermissionKind } from "../../bindings/PermissionKind";
import type { PermissionSnapshot } from "../../bindings/PermissionSnapshot";
import type { PermissionStatus } from "../../bindings/PermissionStatus";
export type { PermissionKind, PermissionSnapshot, PermissionStatus };

export const initialPermissionSnapshot: PermissionSnapshot = {
  accessibility: { canRequest: true, granted: false },
  camera: { canRequest: true, granted: false },
  microphone: { canRequest: true, granted: false },
  screenRecording: { canRequest: true, granted: false },
};
