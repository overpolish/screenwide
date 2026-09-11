// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { createContext, ReactNode, use, useEffect } from "react";

/**
 * The editor's tools live in the title bar, the way a unified toolbar sets
 * them, while the section that owns their behaviour lives under it. The
 * section hands its tools up through here and the title bar draws them.
 *
 * A context rather than a module global, so each editor window and each story
 * carries its own workspace's tools and nothing else. Split in two so the
 * setter stays stable: a section re-rendering at pointer rate never re-runs
 * the effect that publishes.
 */
export const EditorToolbarContext = createContext<ReactNode>(null);

export const SetEditorToolbarContext = createContext<
  (tools: ReactNode) => void
>(() => undefined);

/** The title bar's side: whatever the visible section is offering. */
export const useEditorToolbarTools = () => use(EditorToolbarContext);

/**
 * The section's side. Pass a memoized element: it is published as-is, so an
 * element rebuilt every render would re-render the title bar every render.
 */
export function useProvideEditorToolbarTools(tools: ReactNode) {
  const setTools = use(SetEditorToolbarContext);
  useEffect(() => {
    setTools(tools);
  }, [setTools, tools]);
  // Leaving the workspace empties the bar, whatever it was last showing.
  useEffect(
    () => () => {
      setTools(null);
    },
    [setTools],
  );
}
