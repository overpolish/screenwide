// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useDateFormatter } from "react-aria";

import { t } from "../../../../i18n/i18n";
import { Alert } from "../../../../primitives/alert/alert";
import { ScrollArea } from "../../../../primitives/scroll-area/scroll-area";
import { TileChoice } from "../../../../primitives/tile-choice/tile-choice";
import {
  type Picture,
  pictureSrc,
  usePictureLibrary,
} from "../../images/picture-library";

// The library's pictures, offered as the canvas's background in a column
// beside the Frame island, each as a tile of itself over its name, as the
// Library island names it. The one the canvas wears is chosen.

export function FrameLibraryPictures({
  onChoose,
  selectedPath,
}: {
  onChoose: (picture: Picture) => void;
  /** The file the canvas's background is drawn from, where it is a
   * picture. */
  selectedPath: string | null;
}) {
  const library = usePictureLibrary();
  const added = useDateFormatter({ dateStyle: "medium", timeStyle: "short" });
  if (library.length === 0)
    return <Alert>{t("editor-island-frame-library-empty")}</Alert>;
  return (
    <ScrollArea>
      <TileChoice
        aria-label={t("editor-island-frame-library")}
        items={library.map((picture) => ({
          id: picture.file,
          label:
            picture.title ??
            t("editor-island-library-picture-unnamed", {
              date: added.format(new Date(picture.addedMs)),
            }),
          picture: (
            <img
              alt=""
              className="size-full object-cover"
              decoding="async"
              draggable={false}
              loading="lazy"
              src={pictureSrc(picture)}
            />
          ),
        }))}
        onChange={(file) => {
          const picture = library.find((each) => each.file === file);
          if (picture) onChoose(picture);
        }}
        value={selectedPath ?? ""}
      />
    </ScrollArea>
  );
}
