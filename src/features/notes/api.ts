// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { invoke } from "@tauri-apps/api/core";

import type { Notes } from "../../bindings/Notes";
import type { NotesIslandAction } from "../../bindings/NotesIslandAction";
import type { NotesIslandState } from "../../bindings/NotesIslandState";
import type { StartNotesOptions } from "../../bindings/StartNotesOptions";

export const startNotes = (options: StartNotesOptions) =>
  invoke<null>("start_notes", { options });

export const getNotesIslandState = () =>
  invoke<NotesIslandState | null>("notes_island_state");

export const runNotesIslandAction = (action: NotesIslandAction) =>
  invoke<null>("notes_island_action", { action });

/** The notes project the projects window was asked to show before it could
 * hear, by its manifest. */
export const takePendingNotes = () =>
  invoke<string | null>("take_pending_notes");

export const readNotes = (project: string) =>
  invoke<Notes>("read_notes", { project });

/** Saves the notes, answering the numbers of the speakers whose voice can
 * still be heard: a speaker named since is heard no more. */
export const saveNotes = (project: string, markdown: string) =>
  invoke<number[]>("save_notes", { markdown, project });

/** A few seconds of speaker `number`'s voice, as a WAV file. */
export const readVoiceClip = (project: string, number: number) =>
  invoke<ArrayBuffer>("read_voice_clip", { number, project });

export const copyNotesPath = (project: string) =>
  invoke<null>("copy_notes_path", { project });

/** Sent to the projects window with a notes project's manifest to show. */
export const OPEN_NOTES_EVENT = "notes://open";
