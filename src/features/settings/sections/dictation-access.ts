// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { usePermissionStore } from "../../permissions/store";

/** What macOS asks of dictation before its model is worth downloading:
 * hearing the microphone, and pressing the paste keys in other apps. Windows
 * asks neither, and reports both granted. */
const DICTATION_ACCESS = ["microphone", "accessibility"] as const;

/** The first access dictation still needs, if any. */
export const useDictationMissingAccess = () =>
  usePermissionStore((state) =>
    DICTATION_ACCESS.find((kind) => !state.permissions[kind].granted),
  );
