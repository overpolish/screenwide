// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { create } from "zustand";

import type { AudioClipKind } from "../../../../bindings/AudioClipKind";

/** What the Library shows at a time: songs, sound effects or pictures. */
export type LibraryKind = AudioClipKind | "image";

// The kind shown, kept for the island's window rather than its contents, so
// the row's plus adds what the list below it shows, and the Library comes
// back on the kind it was left on.
const useShown = create<{ kind: LibraryKind }>(() => ({ kind: "music" }));

/** The kind shown among `kinds`, the first where the one last shown is not
 * offered here, and a way to show another. */
export function useLibraryKind(kinds: readonly LibraryKind[]) {
  const shown = useShown((state) => state.kind);
  const kind = kinds.includes(shown) ? shown : kinds[0];
  return {
    kind,
    setKind: (next: LibraryKind) => {
      useShown.setState({ kind: next });
    },
  };
}
