// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { NotesCardPage } from "../../notes/card/notes-card-page";
import { ScriptCardPage } from "../../teleprompter/script/script-card-page";

import type { ProjectSummary } from "../types";

/** What the card of a project read in the projects window, notes or a
 * script, shows in place of a still: the start of its words. */
export function ProjectCardWords({ project }: { project: ProjectSummary }) {
  const { kind, opening } = project;
  if (!opening) return null;
  if (kind === "notes") return <NotesCardPage markdown={opening} />;
  if (kind === "script") return <ScriptCardPage text={opening} />;
  return null;
}
