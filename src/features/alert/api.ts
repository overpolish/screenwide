// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";

/** Sent when the alert window, already loaded, is to say something else. */
const ALERT_SHOWN_EVENT = "alert://shown";

export type AlertCopy = {
  message: string;
  title: string;
};

/** The alert in front right now, if there is one. */
export const getAlert = async () => invoke<AlertCopy | null>("get_alert");

/** Reports the height the alert's content needs; the window is sized to it,
 * centred and shown. */
export const fitAlert = async (height: number) => {
  await invoke<null>("fit_alert", { height });
};

/** Takes the alert in front away; the next one, if any, takes its place. */
export const dismissAlert = async () => {
  await invoke<null>("dismiss_alert");
};

/** The window stays loaded between alerts, so later ones arrive here. */
export const listenToAlertShown = async (
  onShown: (copy: AlertCopy) => void,
): Promise<UnlistenFn> =>
  listen<AlertCopy>(ALERT_SHOWN_EVENT, ({ payload }) => {
    onShown(payload);
  });
