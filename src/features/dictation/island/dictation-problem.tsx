// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../i18n/i18n";
import { splitAtPlaceable } from "../../../i18n/split-at-placeable";
import { hotkeyKeys } from "../../../lib/hotkey";
import { Keyboard } from "../../../primitives/keyboard/keyboard";

import type { DictationProblem } from "../../../bindings/DictationProblem";

// Why a dictation could not start or type, as the island says it.

/** A message naming the paste shortcut, drawn as keycaps where the
 * language puts it. */
const withPasteKeys = (message: (key: string) => string) => {
  const [before, after] = splitAtPlaceable(message);
  return (
    <>
      {before}
      {hotkeyKeys("CommandOrControl+V").map((key) => (
        <Keyboard className="align-middle" key={key}>
          {key}
        </Keyboard>
      ))}
      {after}
    </>
  );
};

export const problemMessage = (problem: DictationProblem) => {
  switch (problem) {
    case "noModel":
      return t("dictation-island-no-model");
    case "noMicrophone":
      return t("dictation-island-no-microphone");
    case "noAccessibility":
      return t("dictation-island-no-accessibility");
    case "passwordField":
      return t("dictation-island-password-field");
    case "administrator":
      return withPasteKeys((key) =>
        t("dictation-island-administrator", { key }),
      );
    case "onClipboard":
      return withPasteKeys((key) =>
        t("dictation-island-on-clipboard", { key }),
      );
    case "failed":
      return t("dictation-island-failed");
  }
};

/** Whether Settings is where `problem` is put right. */
export const isFixedInSettings = (problem: DictationProblem) =>
  problem === "noModel" ||
  problem === "noMicrophone" ||
  problem === "noAccessibility";

/** Whether the dictation waits on the clipboard: not lost, so a warning. */
export const isOnClipboard = (problem: DictationProblem) =>
  problem === "administrator" || problem === "onClipboard";
