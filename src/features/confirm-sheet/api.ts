// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";

/** Sent when a new question arrives at an already-loaded sheet. */
const CONFIRM_SHEET_ASKED_EVENT = "confirm-sheet://asked";

/** Everything the sheet says, named by role: it never learns what it is
 * confirming. */
export type ConfirmSheetCopy = {
  cancelLabel: string;
  confirmLabel: string;
  message: string;
  title: string;
};

/** The question standing right now, if there is one. */
export const getConfirmSheet = async () =>
  invoke<ConfirmSheetCopy | null>("get_confirm_sheet");

/** Reports the height the sheet's content needs; the window is sized to
 * it and, when a question is waiting unseen, shown. */
export const fitConfirmSheet = async (height: number) => {
  await invoke<null>("fit_confirm_sheet", { height });
};

/** Answers the standing question and closes the sheet. */
export const resolveConfirmSheet = async (confirmed: boolean) => {
  await invoke<null>("resolve_confirm_sheet", { confirmed });
};

/** The window stays loaded between questions, so later ones arrive here. */
export const listenToConfirmSheetAsked = async (
  onAsked: (copy: ConfirmSheetCopy) => void,
): Promise<UnlistenFn> =>
  listen<ConfirmSheetCopy>(CONFIRM_SHEET_ASKED_EVENT, ({ payload }) => {
    onAsked(payload);
  });
