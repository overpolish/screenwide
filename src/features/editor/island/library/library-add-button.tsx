// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Plus } from "lucide-react";

import { t } from "../../../../i18n/i18n";
import { IconButton } from "../../../../primitives/button/icon-button";
import { addLibraryPictures } from "../../images/picture-library";
import { browseSounds, importSounds } from "../../sound-track/sound-library";

import { type LibraryKind, useLibraryKind } from "./library-kind";

/** Adds files of the `kind` shown to the library, without placing any. */
const addFiles = (kind: LibraryKind) => {
  const adding =
    kind === "image"
      ? addLibraryPictures()
      : browseSounds().then((paths) =>
          paths.length > 0 ? importSounds(paths) : null,
        );
  adding.catch((cause: unknown) => {
    console.error("Could not add to the library", cause);
  });
};

/** The Library's plus, at the end of its row: files of the kind the list
 * shows, added to the library only. */
export function LibraryAddButton({ kinds }: { kinds: readonly LibraryKind[] }) {
  const { kind } = useLibraryKind(kinds);
  return (
    <IconButton
      aria-label={t("editor-island-library-add")}
      className="text-fg-secondary enabled:hover:text-fg"
      onPress={() => {
        addFiles(kind);
      }}
      variant="ghost"
    >
      <Plus aria-hidden className="size-icon" />
    </IconButton>
  );
}
