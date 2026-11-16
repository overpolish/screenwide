// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { NotesStage } from "../features/notes/view/notes-stage";
import { NotesViewWindow } from "../features/notes/view/notes-view-window";
import { useOpenNotes } from "../features/notes/view/use-open-notes";
import { PopupPanelSync } from "../features/popup-panel/popup-panel-sync";
import { hideProjectBrowser } from "../features/project-browser/api";
import { ProjectBrowserWindow } from "../features/project-browser/project-browser-window";

/** The window, with the shared menu panel's state kept in step, so a
 * location's right-click menu can answer back to it. Notes opened show over
 * the projects, which stay as they were behind them. */
export function ProjectBrowserEntry() {
  const [notes, setNotes] = useOpenNotes();
  return (
    <>
      <PopupPanelSync />
      <NotesStage
        notes={notes}
        page={
          notes ? (
            <NotesViewWindow
              onBack={() => {
                setNotes(null);
              }}
              onClose={() => void hideProjectBrowser()}
              onRenamed={setNotes}
              project={notes}
            />
          ) : null
        }
        projects={<ProjectBrowserWindow onOpenNotes={setNotes} />}
      />
    </>
  );
}
