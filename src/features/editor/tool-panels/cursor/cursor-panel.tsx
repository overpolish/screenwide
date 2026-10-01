// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { Button } from "../../../../components/base/button/button";
import { Text } from "../../../../components/base/text/text";
import { DEFAULT_CURSOR_EFFECTS } from "../../export/recording-export-settings";
import { EditorKind } from "../../types";
import { useToolPanelSnapshot } from "../use-tool-panel-snapshot";

import { CursorEffectControls } from "./cursor-effect-controls";

/** Shows local edits immediately while the editor applies them. */
export function CursorPanel({ workspace }: { workspace: EditorKind }) {
  const { change, snapshot } = useToolPanelSnapshot(workspace);

  if (!snapshot.hasCursorData) {
    return (
      <Text variant="footnote">This recording has no cursor movement.</Text>
    );
  }

  return (
    <>
      <CursorEffectControls
        isDisabled={snapshot.isLocked}
        onChange={(cursorEffects) => {
          change({ cursorEffects });
        }}
        settings={snapshot.cursorEffects}
      />
      {/* The way back to the defaults lives with the settings it undoes,
          where its meaning is plain, rather than on the toolbar. */}
      <div className="flex justify-end">
        <Button
          isDisabled={snapshot.isLocked}
          onPress={() => {
            change({ cursorEffects: DEFAULT_CURSOR_EFFECTS });
          }}
        >
          Reset
        </Button>
      </div>
    </>
  );
}
