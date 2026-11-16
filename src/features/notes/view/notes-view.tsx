// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ChevronLeft } from "lucide-react";

import { CopyButton } from "../../../composites/copy-button/copy-button";
import { WindowHeader } from "../../../composites/window-header/window-header";
import { WindowShell } from "../../../composites/window-shell/window-shell";
import { t } from "../../../i18n/i18n";
import { Alert } from "../../../primitives/alert/alert";
import { Button } from "../../../primitives/button/button";
import { ScrollArea } from "../../../primitives/scroll-area/scroll-area";

import { NotesPage } from "./notes-page";

import type { NotesDocument } from "../document/notes-document";

// Notes in the projects window, in place of the projects: read as a page,
// edited where they stand, their name edited in the title bar, and their
// file's path copied for an assistant to read. Notes still being taken are
// shown as they grow and not yet edited.

export type NotesViewProps = {
  /** The notes; null while they are read. */
  document: NotesDocument | null;
  isTaking: boolean;
  onBack: () => void;
  onChange: (document: NotesDocument, typing?: string) => void;
  onClose: () => void;
  onCopyPath: () => unknown;
  /** Plays speaker `number`'s voice, or stops it if it is playing. */
  onHearVoice: (number: number) => void;
  onMinimize: () => void;
  onRedo: () => void;
  /** Renames the notes' project. */
  onRename: (title: string) => void;
  onUndo: () => void;
  /** The speaker whose voice is playing, by number; null when none is. */
  playingVoice: number | null;
  title: string;
  /** The numbers of the speakers whose voice can still be heard. */
  voices: readonly number[];
  /** Why the notes could not be saved, while that holds. */
  saveError?: string;
};

export function NotesView({
  document,
  isTaking,
  onBack,
  onChange,
  onClose,
  onCopyPath,
  onHearVoice,
  onMinimize,
  onRedo,
  onRename,
  onUndo,
  playingVoice,
  saveError,
  title,
  voices,
}: NotesViewProps) {
  return (
    <WindowShell
      header={
        <WindowHeader
          actions={
            <>
              <Button onPress={onBack} variant="ghost">
                <ChevronLeft
                  aria-hidden
                  className="size-icon rtl:-scale-x-100"
                />
                {t("notes-view-back")}
              </Button>
              <CopyButton
                aria-label={t("notes-view-copy-path")}
                onCopy={onCopyPath}
              />
            </>
          }
          onClose={onClose}
          onMinimize={onMinimize}
          onTitleChange={isTaking ? undefined : onRename}
          title={title}
        />
      }
    >
      {/* Undo and Redo reach the notes as a whole, a rename or a speaker
          given as much as typing, rather than the one paragraph in focus. */}
      <div
        className="flex min-h-0 grow flex-col gap-separate"
        onKeyDown={(event) => {
          if (!(event.metaKey || event.ctrlKey) || event.key !== "z") return;
          event.preventDefault();
          if (event.shiftKey) onRedo();
          else onUndo();
        }}
      >
        {isTaking || saveError ? (
          <div className="mx-auto flex w-full max-w-2xl flex-col gap-separate px-inset">
            {isTaking ? (
              <Alert role="status">{t("notes-view-still-taking")}</Alert>
            ) : null}
            {saveError ? (
              <Alert color="error" role="alert">
                {t("notes-view-save-failed")}
              </Alert>
            ) : null}
          </div>
        ) : null}
        <ScrollArea className="px-inset pb-group" rootClassName="min-h-0 grow">
          {document ? (
            <NotesPage
              document={document}
              isReadOnly={isTaking}
              onChange={onChange}
              onHearVoice={onHearVoice}
              playingVoice={playingVoice}
              voices={voices}
            />
          ) : null}
        </ScrollArea>
      </div>
    </WindowShell>
  );
}
