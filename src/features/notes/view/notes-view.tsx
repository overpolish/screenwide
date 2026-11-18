// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ChevronLeft, Redo2, Undo2 } from "lucide-react";
import { useState } from "react";

import { CopyButton } from "../../../composites/copy-button/copy-button";
import { Reveal } from "../../../composites/reveal/reveal";
import { WindowHeader } from "../../../composites/window-header/window-header";
import { WindowShell } from "../../../composites/window-shell/window-shell";
import { t } from "../../../i18n/i18n";
import { Alert } from "../../../primitives/alert/alert";
import { Button } from "../../../primitives/button/button";
import { IconButton } from "../../../primitives/button/icon-button";
import { ScrollArea } from "../../../primitives/scroll-area/scroll-area";
import { NotesReplaceLine, NotesSearchRow } from "../find/notes-search";
import { useNotesFind } from "../find/use-notes-find";

import { NotesPage } from "./notes-page";

import type { NotesDocument } from "../document/notes-document";

// Notes in the projects window, in place of the projects: read as a page,
// edited where they stand, their name edited in the title bar, and their
// file's path copied for an assistant to read. Undo and Redo sit in the
// title bar too; the search heads the page, its toggle opening Replace
// under it. Notes still being taken are shown as they grow and not yet
// edited. What comes and goes above the page, Replace and the notices,
// opens and closes rather than cutting in.

export type NotesViewProps = {
  canRedo: boolean;
  canUndo: boolean;
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
  canRedo,
  canUndo,
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
  const find = useNotesFind(document);
  const [isReplacing, setIsReplacing] = useState(false);
  const take = (next: NotesDocument | null) => {
    if (next) onChange(next);
  };
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
              <IconButton
                aria-label={t("notes-view-undo")}
                isDisabled={isTaking || !canUndo}
                onPress={onUndo}
                variant="ghost"
              >
                <Undo2 aria-hidden className="rtl:-scale-x-100" />
              </IconButton>
              <IconButton
                aria-label={t("notes-view-redo")}
                isDisabled={isTaking || !canRedo}
                onPress={onRedo}
                variant="ghost"
              >
                <Redo2 aria-hidden className="rtl:-scale-x-100" />
              </IconButton>
              <CopyButton
                aria-label={t("notes-view-copy-path")}
                onCopy={onCopyPath}
              />
            </>
          }
          onClose={onClose}
          onMinimize={onMinimize}
          onTitleChange={onRename}
          title={title}
        />
      }
    >
      {/* Undo and Redo reach the notes as a whole, a rename or a speaker
          given as much as typing, rather than the one paragraph in focus. */}
      <div
        className="flex min-h-0 grow flex-col"
        onKeyDown={(event) => {
          if (!(event.metaKey || event.ctrlKey) || event.key !== "z") return;
          event.preventDefault();
          if (event.shiftKey) onRedo();
          else onUndo();
        }}
      >
        {/* Inset like the page under it, so the search lines up with the
            notes' left edge. */}
        <div className="px-inset">
          <div className="mx-auto flex w-full max-w-2xl flex-col">
            <NotesSearchRow
              isReplacing={isReplacing}
              onQueryChange={find.setQuery}
              onReplacingChange={setIsReplacing}
              onStep={find.step}
              query={find.query}
            />
            <Reveal className="pt-separate">
              {isReplacing ? (
                <NotesReplaceLine
                  found={find.matches.length}
                  isDisabled={isTaking}
                  onReplace={(replacement) => {
                    take(find.replaceOne(replacement));
                  }}
                  onReplaceAll={(replacement) => {
                    take(find.replaceAll(replacement));
                  }}
                />
              ) : null}
            </Reveal>
            <Reveal className="flex flex-col gap-separate pt-separate">
              {isTaking || saveError ? (
                <>
                  {isTaking ? (
                    <Alert role="status">{t("notes-view-still-taking")}</Alert>
                  ) : null}
                  {saveError ? (
                    <Alert color="error" role="alert">
                      {t("notes-view-save-failed")}
                    </Alert>
                  ) : null}
                </>
              ) : null}
            </Reveal>
          </div>
        </div>
        <ScrollArea className="px-inset pb-group" rootClassName="min-h-0 grow">
          {document ? (
            <NotesPage
              current={find.current}
              document={document}
              isReadOnly={isTaking}
              matches={find.matches}
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
