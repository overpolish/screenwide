// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

import { invoke } from "@tauri-apps/api/core";
import { useRef } from "react";

import { t } from "../../../../i18n/i18n";
import { usePlayheadMoment } from "../../../editor/recording/use-copy-recording-frame";
import {
  type PopupMenuAnchor,
  usePopupMenu,
} from "../../../popup-panel/use-popup-menu";
import { type CueRequest, requestCue } from "../cue-api";
import { afterMenuCloses, openCuePanel } from "../cue-panel";
import { PICKER_SIZE } from "../picker/use-picker-size";

import type { EditorKind } from "../../../../bindings/EditorKind";

const ID_PREFIX = "editor-cue:";
const MENU_WIDTH = 200;
/** How far below the window's top the script picker opens when asked for
 * from the keyboard, with no pointer to open under, in points. */
const KEYBOARD_PICKER_TOP = 48;

const report = (what: string) => (error: unknown) => {
  console.error(what, error);
};

/** The script picker's place when asked for from the keyboard: centred at
 * the top of the window. */
const keyboardAnchor = (): PopupMenuAnchor => ({
  height: 1,
  width: PICKER_SIZE.width,
  x: (window.innerWidth - PICKER_SIZE.width) / 2,
  y: KEYBOARD_PICKER_TOP,
});

/**
 * Adds what the `editor` editor has open as `artifactId` to a script as a
 * cue: a recording's frame under the playhead, as its edit shows it, or the
 * screenshot as edited. The open teleprompter waits for a word to be
 * clicked for it; with none open, the editor first asks which script, under
 * `anchor` or at the top of the window, and that script opens waiting.
 */
export const addEditorCue = async (
  editor: EditorKind,
  artifactId: number,
  anchor: PopupMenuAnchor | null = null,
) => {
  const moment = usePlayheadMoment.getState().moment;
  if (editor === "recording" && !moment) return;
  const source = await invoke<string>("editor_cue_source", {
    artifactId,
    editor,
  });
  const timing = editor === "recording" && moment ? moment() : null;
  const request: CueRequest = {
    source,
    timing: timing && {
      playedMs: Math.round(timing.playedMs),
      sourceMs: Math.round(timing.sourceMs),
    },
  };
  if (await requestCue(request)) return;
  await openCuePanel({
    anchor: anchor ?? keyboardAnchor(),
    content: { kind: "scriptPicker", ...request },
    id: `${ID_PREFIX}scripts`,
    label: t("teleprompter-cue-script-picker"),
  });
};

/** The item another menu offers to add what its editor has open to a script
 * (`addEditorCueFromMenu`). */
export const addCueMenuItem = () => ({
  id: "add-cue",
  label: t("teleprompter-cue-add-to-script"),
  shortcut: "Alt+KeyC",
});

/** `addEditorCue` for an item picked in a menu opened at `anchor`, once that
 * menu has closed: the script picker opens in the panel it leaves. */
export const addEditorCueFromMenu = (
  editor: EditorKind,
  artifactId: number,
  anchor: PopupMenuAnchor,
) => {
  afterMenuCloses()
    .then(() => addEditorCue(editor, artifactId, anchor))
    .catch(report("Could not add the cue"));
};

/** A menu, opened under the anchor it is given, offering to add the frame
 * under the playhead of the recording open as `artifactId` to a script. */
export function useRecordingCueMenu(artifactId: number) {
  const anchorRef = useRef<PopupMenuAnchor | null>(null);
  const openMenu = usePopupMenu({
    idPrefix: ID_PREFIX,
    label: t("teleprompter-cue-menu"),
    mode: "menu",
    onSelect: () => {
      if (anchorRef.current)
        addEditorCueFromMenu("recording", artifactId, anchorRef.current);
    },
    width: MENU_WIDTH,
  });
  return (anchor: PopupMenuAnchor) => {
    anchorRef.current = anchor;
    openMenu({ anchor, items: [addCueMenuItem()] }).catch(
      report("Could not open the cue menu"),
    );
  };
}
