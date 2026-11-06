// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { storyRecording } from "../features/editor/timeline/strip/strip-story-fixtures";
import { LiveStrip } from "../features/editor/timeline/strip/strip-story-live";

// The strip's own stories' recording, so the page shows what the editor
// shows. It is built once: the strip keeps its edits in its own state.
const recording = storyRecording({ duration: 90 });

// The strip reports each edit for the editor to keep; the page keeps none,
// so the edits last until the page reloads.
const ignore = () => {};

/** The editor's real timeline, over a sample recording, to try. */
export function SiteTimeline() {
  return (
    // As wide as the editor's window needs, wider than a phone.
    <section className="hidden pb-16 md:block" dir="ltr">
      <LiveStrip
        isAltHeld={false}
        isAnnotationsHidden={false}
        isCaptionsHidden={false}
        isCutting={false}
        isPlaying={false}
        isRanging={false}
        isScreenShown
        isShortcutsHidden={false}
        moves={[]}
        onAnnotationRestack={ignore}
        onAnnotationRetime={ignore}
        onBeatTrim={ignore}
        onCaptionPress={ignore}
        onDeleteShortcuts={ignore}
        onLullPlayChange={ignore}
        onLullTrim={ignore}
        onMove={ignore}
        onPutBack={ignore}
        onRemoveSplit={ignore}
        onRestore={ignore}
        onSceneDraw={ignore}
        onSceneRetime={ignore}
        onSeek={ignore}
        onSelect={ignore}
        onSplit={ignore}
        onSqueezeChange={ignore}
        recording={recording}
        selection={null}
        squeeze={0.5}
      />
    </section>
  );
}
