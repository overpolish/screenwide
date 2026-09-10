// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { isTauri } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";

/** Sent to the window a sheet stands over, and again when it leaves. */
const MODAL_EVENT = "confirm-sheet://modal";

/**
 * Whether a confirm sheet is standing over this window. A child window does
 * not take its parent's input on macOS, so the parent goes inert on its own
 * while the question is up, the way it does under the export sheet.
 */
export function useConfirmSheetModal() {
  const [isModal, setIsModal] = useState(false);
  useEffect(() => {
    if (!isTauri()) return;
    let unlisten: UnlistenFn | undefined;
    let disposed = false;
    void listen<boolean>(MODAL_EVENT, ({ payload }) => {
      setIsModal(payload);
    }).then((listener) => {
      if (disposed) listener();
      else unlisten = listener;
    });
    return () => {
      disposed = true;
      unlisten?.();
    };
  }, []);
  return isModal;
}
