// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ReactNode, useState } from "react";

import {
  EditorToolbarContext,
  SetEditorToolbarContext,
} from "./editor-toolbar-context";

/**
 * Holds the tools the visible section is offering, for the title bar above it
 * to draw. `children` is an element the caller already holds, so publishing a
 * new set of tools re-renders the title bar and nothing else.
 */
export function EditorToolbarProvider({ children }: { children: ReactNode }) {
  const [tools, setTools] = useState<ReactNode>(null);

  return (
    <SetEditorToolbarContext value={setTools}>
      <EditorToolbarContext value={tools}>{children}</EditorToolbarContext>
    </SetEditorToolbarContext>
  );
}
