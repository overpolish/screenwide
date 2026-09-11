// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Text } from "../../../components/base/text/text";
import { CursorEffectControls } from "../components/cursor-effect-controls";
import { EditorKind } from "../types";

import { useToolPanelSnapshot } from "./use-tool-panel-snapshot";

/** Shows local edits immediately while the editor applies them. */
export function CursorPanel({ workspace }: { workspace: EditorKind }) {
  const { change, snapshot } = useToolPanelSnapshot(workspace);

  if (!snapshot.hasCursorData) {
    return (
      <Text variant="footnote">This recording has no cursor movement.</Text>
    );
  }

  return (
    <CursorEffectControls
      isSaving={snapshot.isSaving}
      onChange={(cursorEffects) => {
        change({ cursorEffects });
      }}
      settings={snapshot.cursorEffects}
    />
  );
}
