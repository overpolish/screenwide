// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Plus, SearchX, Trash2 } from "lucide-react";
import { useState } from "react";
import { useDateFormatter, useFilter } from "react-aria";
import { Button } from "react-aria-components";

import { ConfirmActionButton } from "../../../../composites/confirm-action-button/confirm-action-button";
import { t } from "../../../../i18n/i18n";
import { cn, elementFocusVisible, focusStyles } from "../../../../lib/styling";
import { Alert } from "../../../../primitives/alert/alert";
import {
  addLibraryPictures,
  dragLibraryPicture,
  type Picture,
  pictureSrc,
  removeLibraryPicture,
  renameLibraryPicture,
  usePictureLibrary,
} from "../../images/picture-library";

import { LibraryNameField } from "./library-name-field";
import { useDragOut } from "./use-drag-out";

import type { ImageArt } from "../../annotations/annotations";

// The library's pictures, every one given to an image, as tiles of the
// pictures themselves, four to a row, each named under it: by the name it
// was given, else the file it came from, else when it arrived; found by
// that name, and renamed by clicking it (or F2 on the tile). Pressed, one
// is placed as a new image in the middle of the layer in hand; dragged
// out, drawn as itself, it lands where it is let go, on the picture or the
// timeline, as a file from the Finder does, where the system lets a drag
// start. Each tile's bin, out of the way until the tile is pointed at or
// reached, takes it out of the library, asking twice; projects that show it
// keep their own copy. On the image island the grid leads with a card that
// adds pictures, so there is always something to press, and the tiles are
// the pictures alone: names, and renaming, are the Library's.

export function LibraryPictureGrid({
  alsoShown,
  hasAddCard = false,
  hasNames = true,
  onAdd,
  query = "",
}: {
  onAdd: (art: ImageArt) => void;
  /** Pictures shown whatever the search, by name. */
  alsoShown?: ReadonlySet<string>;
  /** Leads with a card adding pictures to the library, which stands in for
   * the empty library's note. */
  hasAddCard?: boolean;
  /** Names each tile under it, where it can be renamed. Without, the tiles
   * are the pictures alone, named for assistive technology only. */
  hasNames?: boolean;
  /** The words searched for in the names of the files pictures came from. */
  query?: string;
}) {
  const library = usePictureLibrary();
  const { contains } = useFilter({ sensitivity: "base" });
  const added = useDateFormatter({ dateStyle: "medium", timeStyle: "short" });
  const named = library.map((picture) => ({
    name:
      picture.title ??
      t("editor-island-library-picture-unnamed", {
        date: added.format(new Date(picture.addedMs)),
      }),
    picture,
  }));
  const shown = named.filter(
    ({ name, picture }) =>
      alsoShown?.has(picture.name) === true || contains(name, query),
  );
  // Standing in for the grid, the note keeps the room a row would, rather
  // than sitting tight under the search. The add card stands in instead.
  if (shown.length === 0 && !hasAddCard)
    return library.length === 0 ? (
      <Alert className="py-inset">
        {t("editor-island-library-empty-images")}
      </Alert>
    ) : (
      <Alert className="py-inset" icon={<SearchX />}>
        {t("editor-island-sound-library-no-match")}
      </Alert>
    );
  return (
    <ul
      aria-label={t("editor-island-library-images")}
      className="grid grid-cols-4 gap-1"
    >
      {hasAddCard ? <AddPicturesCard hasName={hasNames} /> : null}
      {shown.map(({ name, picture }) => (
        <PictureTile
          hasName={hasNames}
          key={picture.name}
          name={name}
          onAdd={onAdd}
          picture={picture}
        />
      ))}
    </ul>
  );
}

// Opens the file picker and adds what is chosen to the library, where it
// arrives as the first tile after this card; it places nothing.
function AddPicturesCard({ hasName }: { hasName: boolean }) {
  const label = t("editor-island-image-add-pictures");
  return (
    <li className="flex min-w-0 flex-col">
      {/* Drawn as a tile is, its dashed blank inset as a picture is. */}
      <Button
        aria-label={label}
        className={cn(
          "relative block aspect-square w-full cursor-default rounded-control text-fg-secondary transition-colors data-[hovered]:bg-fill data-[hovered]:text-fg",
          focusStyles,
          elementFocusVisible,
        )}
        onPress={() => {
          addLibraryPictures().catch((cause: unknown) => {
            console.error("Could not add to the library", cause);
          });
        }}
      >
        <span className="absolute inset-1 flex items-center justify-center rounded-part border border-dashed border-fg-tertiary">
          <Plus aria-hidden className="size-icon-lg" />
        </span>
      </Button>
      {hasName ? (
        <span
          aria-hidden
          className="truncate px-1 text-center text-caption text-fg-secondary"
        >
          {label}
        </span>
      ) : null}
    </li>
  );
}

function PictureTile({
  hasName,
  name,
  onAdd,
  picture,
}: {
  /** Whether its name shows under it, where it can be renamed. */
  hasName: boolean;
  /** What the tile is called, under it and for assistive technology. */
  name: string;
  onAdd: (art: ImageArt) => void;
  picture: Picture;
}) {
  const dragOut = useDragOut(() => {
    dragLibraryPicture(picture.name).catch((cause: unknown) => {
      console.error("Could not drag the picture", cause);
    });
  });
  const [isEditing, setIsEditing] = useState(false);
  return (
    <li className="group relative flex min-w-0 flex-col" {...dragOut}>
      {/* Drawn as the island's tiles are: the picture inset, the tile
          filled under the pointer. */}
      <Button
        aria-label={t("editor-island-library-picture-add", { name })}
        className={cn(
          "relative block aspect-square w-full cursor-default rounded-control transition-colors data-[hovered]:bg-fill",
          focusStyles,
          elementFocusVisible,
        )}
        onKeyDown={(event) => {
          if (event.key === "F2" && hasName) setIsEditing(true);
          else event.continuePropagation();
        }}
        onPress={() => {
          onAdd(picture.art);
        }}
      >
        {/* Held in a box of its own, so a tall picture fits the square
            rather than stretching it. */}
        <span className="absolute inset-1">
          <img
            alt=""
            className="size-full object-contain"
            decoding="async"
            draggable={false}
            loading="lazy"
            src={pictureSrc(picture)}
          />
        </span>
      </Button>
      {!hasName ? null : isEditing ? (
        <LibraryNameField
          className="px-1 text-center text-caption text-fg"
          label={t("editor-island-library-name")}
          onDone={() => {
            setIsEditing(false);
          }}
          onRename={(title) => {
            renameLibraryPicture(picture.name, title).catch(
              (cause: unknown) => {
                console.error("Could not rename the picture", cause);
              },
            );
          }}
          title={picture.title ?? ""}
        />
      ) : (
        <span
          className="cursor-text truncate px-1 text-center text-caption text-fg-secondary"
          dir="auto"
          // The name is apart from the tile that adds it, so one click is
          // enough, where a sound's title is the press that adds it.
          onClick={() => {
            setIsEditing(true);
          }}
        >
          {name}
        </span>
      )}
      {/* On the island's black, so it reads over any picture. */}
      <span className="absolute end-0 top-0 rounded-full bg-island opacity-0 transition-opacity group-focus-within:opacity-100 group-hover:opacity-100">
        <ConfirmActionButton
          confirmLabel={t("editor-island-library-confirm-remove")}
          icon={<Trash2 aria-hidden />}
          label={t("editor-island-library-picture-remove", { name })}
          onConfirm={() => {
            removeLibraryPicture(picture.name).catch((cause: unknown) => {
              console.error("Could not remove the picture", cause);
            });
          }}
          variant="ghost"
        />
      </span>
    </li>
  );
}
