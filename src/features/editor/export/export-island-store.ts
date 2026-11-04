// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { create } from "zustand";

import type { ExportedFile } from "../../../bindings/ExportedFile";

// Where the Export island stands in this editor window: out over the
// picture or not, the file the last save wrote while the island shows it
// done, and why the last save failed. One editor window holds one
// workspace, so the store is the window's own. The title bar's Export
// button, the island window's placing and the editor's locking all read it.

type ExportIslandState = {
  /** The file written, while the finished island shows it. */
  exported: ExportedFile | null;
  /** Why the last save failed, until it is tried again or changed. */
  failure: string | null;
  /** Out with GIF chosen: the strip stays in reach for the range a GIF is
   * made of, while the rest of the editor is locked. */
  isGifRange: boolean;
  /** Out over the picture; false while it is tucked away exporting. */
  isOpen: boolean;
};

const AWAY: ExportIslandState = {
  exported: null,
  failure: null,
  isGifRange: false,
  isOpen: false,
};

export const useExportIslandStore = create<ExportIslandState>()(() => AWAY);

export const setGifRange = (isGifRange: boolean) => {
  if (useExportIslandStore.getState().isGifRange !== isGifRange)
    useExportIslandStore.setState({ isGifRange });
};

/** Brings the island out, or tucks it away while it exports. */
export const setExportIslandOpen = (isOpen: boolean) => {
  useExportIslandStore.setState({ isOpen });
};

/** The save failed: the island comes out to say so, with Try again. */
export const failExport = (failure: string) => {
  useExportIslandStore.setState({ failure, isOpen: true });
};

/** A setting changed or the save tried again: the failure is let go. */
export const clearExportFailure = () => {
  if (useExportIslandStore.getState().failure !== null)
    useExportIslandStore.setState({ failure: null });
};

/** The file is written: the island comes out to show it done. */
export const finishExport = (exported: ExportedFile) => {
  useExportIslandStore.setState({ exported, failure: null, isOpen: true });
};

/** Put away for good, as a new capture arrives or the finish is let go. */
export const resetExportIsland = () => {
  useExportIslandStore.setState(AWAY);
};
