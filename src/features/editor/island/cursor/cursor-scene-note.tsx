// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { t } from "../../../../i18n/i18n";
import { cn } from "../../../../lib/styling";
import { Alert } from "../../../../primitives/alert/alert";

/** At the Cursor island's foot over a scene, about the eye on its row:
 * whether the scene shows or hides the cursor its own way, or, where it does
 * as the recording does, that the eye acts on this scene alone. Only the eye
 * is the scene's; the options above hold for the whole recording. Under the
 * options it takes their width; alone, with the cursor hidden there, it sets
 * its own, so its words stay on a line. */
export function CursorSceneNote({
  choice,
  isShown,
}: {
  /** The scene's own choice, null where it does as the recording does. */
  choice: boolean | null;
  /** Whether the cursor shows in the scene. */
  isShown: boolean;
}) {
  const state = choice === null ? "other" : choice ? "shown" : "hidden";
  return (
    <Alert
      className={cn("px-inset-sm", isShown && "w-0 min-w-full")}
      role="status"
    >
      {t("editor-island-cursor-in-scene", { state })}
    </Alert>
  );
}
