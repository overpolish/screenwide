// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { PopupPanelSync } from "../features/popup-panel/popup-panel-sync";
import { ProjectBrowserWindow } from "../features/project-browser/project-browser-window";

/** The window, with the shared menu panel's state kept in step, so a
 * location's right-click menu can answer back to it. */
export function ProjectBrowserEntry() {
  return (
    <>
      <PopupPanelSync />
      <ProjectBrowserWindow />
    </>
  );
}
