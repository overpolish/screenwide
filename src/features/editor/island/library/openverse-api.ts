// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Channel, invoke } from "@tauri-apps/api/core";

import { t } from "../../../../i18n/i18n";

import type { AudioClipKind } from "../../../../bindings/AudioClipKind";
import type { OpenverseFailure } from "../../../../bindings/OpenverseFailure";
import type { OpenverseFound } from "../../../../bindings/OpenverseFound";
import type { OpenverseKind } from "../../../../bindings/OpenverseKind";
import type { OpenversePage } from "../../../../bindings/OpenversePage";
import type { Sound } from "../../sound-track/sound-library";

// Openverse as the Library island asks it, through the app
// (`editor/openverse.rs`), which asks Screenwide's Worker.

export type { OpenverseFailure, OpenverseFound, OpenverseKind, OpenversePage };

/** Whether `kind` is searched online. Pictures are left out while
 * Openverse's image search times out on every query
 * (github.com/WordPress/openverse/issues/5693); let them back in here once
 * it answers again. Typed `boolean`, so the picture paths it turns off are
 * still checked, not narrowed away. */
export const isSearchedOnline = (kind: OpenverseKind): boolean =>
  kind !== "image";

/** `page` of what Openverse holds of `kind` for `query`. Rejects with an
 * `OpenverseFailure`. */
export const searchOpenverse = (
  kind: OpenverseKind,
  query: string,
  page: number,
) => invoke<OpenversePage>("search_openverse", { kind, page, query });

/** A channel handing `onProgress` how much of a file has come, 0 to 1. */
const progressChannel = (onProgress: (fraction: number) => void) => {
  const channel = new Channel<number>();
  channel.onmessage = onProgress;
  return channel;
};

/** Fetches the song or sound effect `found` into the library, listed as
 * `kind`, credited to who made it, and says what it is. */
export const fetchOpenverseSound = (
  found: OpenverseFound,
  kind: AudioClipKind,
  onProgress: (fraction: number) => void,
) =>
  invoke<Sound>("add_openverse_sound", {
    found,
    kind,
    onProgress: progressChannel(onProgress),
  });

/** Fetches the picture `found` into the library, credited to who made it,
 * and says the library's name for it. */
export const fetchOpenversePicture = (
  found: OpenverseFound,
  onProgress: (fraction: number) => void,
) =>
  invoke<string>("add_openverse_picture", {
    found,
    onProgress: progressChannel(onProgress),
  });

/** Why a search failed, from what it rejected with. */
export const failureOf = (cause: unknown): OpenverseFailure =>
  cause === "busy" ? "busy" : "unavailable";

/** A licence's short name, as the library shows it. */
export const licenseName = (license: string) =>
  license === "by"
    ? t("editor-island-openverse-license-by")
    : license === "cc0"
      ? t("editor-island-openverse-license-cc0")
      : t("editor-island-openverse-license-pdm");
