// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useSyncExternalStore } from "react";

import type { Picture } from "../../../bindings/Picture";
import type { ImageArt } from "../annotations/annotations";

// The library of pictures as this window knows it: every picture given to
// an image, read once and again whenever one arrives or is taken out
// (`editor/images/library.rs`). The pictures themselves live in the app's
// data folder (`screenwide_images::store`).

export type { Picture };

let library: Picture[] = [];
let isRead = false;
const listeners = new Set<() => void>();

const subscribe = (listener: () => void) => {
  listeners.add(listener);
  return () => {
    listeners.delete(listener);
  };
};

const read = () => {
  invoke<Picture[]>("get_picture_library")
    .then((pictures) => {
      library = pictures;
      for (const listener of listeners) listener();
    })
    .catch((cause: unknown) => {
      console.error("Could not read the picture library", cause);
    });
};

/** Every picture in the library, the most recently added first, read once
 * per window and again whenever it changes (`editor://picture-library`). */
export function usePictureLibrary() {
  useEffect(() => {
    if (isRead) return;
    isRead = true;
    read();
    void listen("editor://picture-library", read);
  }, []);
  return useSyncExternalStore(subscribe, () => library);
}

/** Asks for pictures and adds them to the library, without placing any. */
export const addLibraryPictures = () => invoke<null>("add_library_pictures");

/** Takes the picture `name` out of the library; projects that show it keep
 * their own copy. */
export const removeLibraryPicture = (name: string) =>
  invoke<null>("remove_library_picture", { name });

/** Starts dragging the picture `name`'s file out of this window, from the
 * press under way, onto the picture or the timeline, or into another app. */
export const dragLibraryPicture = (name: string) =>
  invoke<null>("drag_library_picture", { name });

/** Keeps the picture at `path`, dragged in, and says what an image showing
 * it carries. */
export const importLibraryPicture = (path: string) =>
  invoke<ImageArt>("import_library_picture", { path });

/** Gives the picture `name` the name `title` in the library. */
export const renameLibraryPicture = (name: string, title: string) =>
  invoke<null>("rename_library_picture", { name, title });

/** The kinds of file a picture can be made from, as
 * `editor::images::import::PICTURE_EXTENSIONS` lists them. */
const EXTENSIONS: readonly string[] = [
  "bmp",
  "gif",
  "jpeg",
  "jpg",
  "png",
  "tif",
  "tiff",
  "webp",
];

/** Whether `path` names a kind of file a picture can be made from. */
export const isPicturePath = (path: string) =>
  EXTENSIONS.includes(path.split(".").pop()?.toLowerCase() ?? "");

/** Where the page reads `picture` to show it. */
export const pictureSrc = (picture: Picture) => convertFileSrc(picture.file);
