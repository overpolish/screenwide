// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { useEffect, useRef, useState } from "react";

import { renameProject } from "../../project-browser/api";
import { copyNotesPath, NOTES_MOVED_EVENT, readNotes, saveNotes } from "../api";
import { parseNotes, writeNotes } from "../document/notes-document";
import { useVoiceClips } from "../speakers/use-voice-clips";

import { NotesView } from "./notes-view";
import { useNotesHistory } from "./use-notes-history";

/** How long after the last change the notes are saved, in milliseconds. */
const SAVE_AFTER_MS = 500;
/** How often notes still being taken are read again, in milliseconds. */
const GROWING_MS = 2_000;

/** A project's name: its folder's, as the projects list names it. */
const projectTitle = (project: string) =>
  project.split(/[/\\]/).slice(-2, -1).join("");

/** The notes of the project whose manifest is `project`, read from their
 * Markdown file and saved back to it as they are edited. Renamed, the
 * project moves, and `onRenamed` is told where to: at once, or once the
 * notes are finished when they are renamed while being taken. */
export function NotesViewWindow({
  onBack,
  onClose,
  onRenamed,
  project,
}: {
  onBack: () => void;
  onClose: () => void;
  onRenamed: (project: string) => void;
  project: string;
}) {
  const history = useNotesHistory();
  const { document, reset } = history;
  const [isTaking, setIsTaking] = useState(false);
  const [title, setTitle] = useState(() => projectTitle(project));
  const voices = useVoiceClips(project);
  const { setVoices } = voices;
  const [saveError, setSaveError] = useState<string>();
  const projectRef = useRef(project);
  projectRef.current = project;
  const onRenamedRef = useRef(onRenamed);
  onRenamedRef.current = onRenamed;

  // Read once as the notes open, and again every few seconds while still
  // being taken, so they show as they grow. A rename moves the project, not
  // the notes, so it reads nothing again; but notes renamed while being
  // taken move once they are finished, and are read again from there.
  useEffect(() => {
    let disposed = false;
    let timer: number | undefined;
    const read = () => {
      window.clearTimeout(timer);
      readNotes(projectRef.current)
        .then((notes) => {
          if (disposed) return;
          reset(parseNotes(notes.markdown));
          setIsTaking(notes.isTaking);
          setTitle(notes.title);
          setVoices(notes.voices);
          if (notes.isTaking) timer = window.setTimeout(read, GROWING_MS);
        })
        .catch((cause: unknown) => {
          console.error("Could not read the notes", cause);
        });
    };
    read();
    const unlisten = listen<[string, string]>(
      NOTES_MOVED_EVENT,
      ({ payload: [from, to] }) => {
        if (from !== projectRef.current) return;
        projectRef.current = to;
        onRenamedRef.current(to);
        read();
      },
    );
    return () => {
      disposed = true;
      window.clearTimeout(timer);
      void unlisten.then((stop) => {
        stop();
      });
    };
  }, [reset, setVoices]);

  // Changes made here are saved a moment after the last; notes read afresh
  // are not written back. What waits is saved as the view goes.
  const editedRef = useRef(false);
  const pendingRef = useRef<string | null>(null);
  const flush = () => {
    const markdown = pendingRef.current;
    pendingRef.current = null;
    if (markdown === null) return Promise.resolve();
    return saveNotes(projectRef.current, markdown).then(
      (kept) => {
        setSaveError(undefined);
        setVoices(kept);
      },
      (cause: unknown) => {
        setSaveError(String(cause));
      },
    );
  };
  const flushRef = useRef(flush);
  flushRef.current = flush;
  useEffect(() => {
    if (!document || !editedRef.current) return;
    editedRef.current = false;
    pendingRef.current = writeNotes(document);
    const timer = window.setTimeout(() => {
      void flushRef.current();
    }, SAVE_AFTER_MS);
    return () => {
      window.clearTimeout(timer);
    };
  }, [document]);
  useEffect(
    () => () => {
      void flushRef.current();
    },
    [],
  );
  const edited =
    <Args extends unknown[]>(action: (...args: Args) => void) =>
    (...args: Args) => {
      editedRef.current = true;
      action(...args);
    };

  return (
    <NotesView
      canRedo={history.canRedo}
      canUndo={history.canUndo}
      document={document}
      isTaking={isTaking}
      onBack={onBack}
      onChange={edited(history.change)}
      onClose={onClose}
      onCopyPath={() => copyNotesPath(projectRef.current)}
      onHearVoice={voices.hear}
      onMinimize={() => void getCurrentWindow().minimize()}
      onRedo={edited(history.redo)}
      onRename={(next) => {
        // What waits is written where the notes are now, before they move.
        void flush()
          .then(() => renameProject(projectRef.current, next))
          .then((moved) => {
            // Notes being taken keep their folder until they are finished,
            // and their name waits in the manifest until then.
            if (moved === projectRef.current) {
              setTitle(next);
              return;
            }
            projectRef.current = moved;
            onRenamed(moved);
            setTitle(projectTitle(moved));
            edited(history.retitle)(projectTitle(moved));
          })
          .catch((cause: unknown) => {
            console.error("Could not rename the notes", cause);
          });
      }}
      onUndo={edited(history.undo)}
      playingVoice={voices.playing}
      saveError={saveError}
      title={title}
      voices={voices.voices}
    />
  );
}
