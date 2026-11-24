// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { createPlayhead } from "../scrub-playhead";
import { recordedTime, type TimeMap } from "../timeline-scale";

import {
  recordingStripLanes,
  type RecordingStripLaneSource,
} from "./recording-strip-lanes";

import type { RecordingAnnotationClip } from "../../../../bindings/RecordingAnnotationClip";
import type { RecordingSceneClip } from "../../../../bindings/RecordingSceneClip";

const RECORDED = 10;

/** The lanes over `clips` with the playhead at `atSeconds`, the moments
 * they asked the playhead to go to, and the scenes they changed to. */
function lanesAt(
  atSeconds: number,
  clips: RecordingSceneClip[] = [],
  {
    annotationClips = [],
    canPlaceCamera = true,
    gaps = [],
    map = recordedTime(RECORDED),
  }: {
    annotationClips?: RecordingAnnotationClip[];
    canPlaceCamera?: boolean;
    gaps?: { endMs: number; startMs: number }[];
    map?: TimeMap;
  } = {},
) {
  const playhead = createPlayhead();
  playhead.publish(atSeconds, atSeconds / RECORDED);
  const seeks: number[] = [];
  const changed: RecordingSceneClip[][] = [];
  const chosen: string[] = [];
  const annotationsChanged: RecordingAnnotationClip[][] = [];
  const source: RecordingStripLaneSource = {
    annotations: {
      clearSelection: () => undefined,
      clips: annotationClips,
      onClipsChange: (next) => {
        annotationsChanged.push(next);
      },
      onSelect: () => undefined,
      selectedIds: new Set(),
    },
    changeCanvasTool: () => undefined,
    gaps,
    map,
    openCaptions: () => undefined,
    openScenes: () => undefined,
    playhead,
    scenes: {
      canPlaceCamera,
      clips,
      drawingOf: () => ({ aspect: 16 / 9 }),
      isAvailable: true,
      onClipsChange: (next) => {
        changed.push(next);
      },
      selection: {
        ids: new Set(),
        onClear: () => undefined,
        onSelect: (id) => {
          chosen.push(id);
        },
      },
    },
    seek: (seconds) => {
      seeks.push(seconds);
    },
    steps: {
      once: () => undefined,
      step: (next) => {
        if (typeof next === "function") next();
      },
    },
  };
  return {
    annotationsChanged,
    changed,
    chosen,
    lanes: recordingStripLanes(source).lanes,
    seeks,
  };
}

describe("a scene drawn on the rail", () => {
  it.each([1, 5, 8])(
    "is chosen and leaves the playhead where it stands, at %d s",
    (at) => {
      const { changed, chosen, lanes, seeks } = lanesAt(at);
      lanes.onSceneDraw(4, 6);
      expect(chosen).toEqual([changed[0]?.[0]?.id]);
      expect(seeks).toEqual([]);
    },
  );

  it("grows a drag too short to the shortest drawn scene, from the press", () => {
    const { changed, lanes } = lanesAt(0);
    lanes.onSceneDraw(4, 4.2);
    expect(changed).toMatchObject([[{ endMs: 5000, startMs: 4000 }]]);
  });

  it("grows a short drag drawn backwards to end at the press", () => {
    const { changed, lanes } = lanesAt(0);
    lanes.onSceneDraw(4.2, 4);
    expect(changed).toMatchObject([[{ endMs: 4200, startMs: 3200 }]]);
  });

  it("slides a grown scene clear of the neighbour it would run into", () => {
    const { changed, lanes } = lanesAt(0, [
      { endMs: 8000, id: "after", preset: "full", startMs: 5000 },
    ]);
    lanes.onSceneDraw(4.5, 4.6);
    expect(changed).toMatchObject([
      [{ endMs: 5000, startMs: 4000 }, { id: "after" }],
    ]);
  });

  it("starts as a picture in picture where the camera can be placed", () => {
    const { changed, lanes } = lanesAt(0);
    lanes.onSceneDraw(4, 6);
    expect(changed).toMatchObject([[{ preset: "picture-in-picture" }]]);
  });

  it("starts as the screen alone where there is no camera to place", () => {
    const { changed, lanes } = lanesAt(0, [], { canPlaceCamera: false });
    lanes.onSceneDraw(4, 6);
    expect(changed).toMatchObject([[{ preset: "screen-only" }]]);
  });
});

describe("a scene beside a gap", () => {
  const GAPS = [{ endMs: 7000, startMs: 5000 }];

  it("is drawn up to the gap's edge, not over it", () => {
    const { changed, lanes } = lanesAt(0, [], { gaps: GAPS });
    lanes.onSceneDraw(3, 8);
    expect(changed).toMatchObject([[{ endMs: 5000, startMs: 3000 }]]);
  });

  it("stops whole at the gap's edge when moved into it", () => {
    const { changed, lanes } = lanesAt(
      0,
      [{ endMs: 3000, id: "moved", preset: "full", startMs: 1000 }],
      { gaps: GAPS },
    );
    lanes.onSceneRetime("moved", 4, 6);
    expect(changed).toMatchObject([[{ endMs: 5000, startMs: 3000 }]]);
  });
});

// Ten recorded seconds played with 4 to 6 moved to the end: 0 to 4, then
// 6 to 10, then 4 to 6.
const MOVED: TimeMap = {
  duration: RECORDED,
  toPlayed: (recorded) =>
    recorded < 4 ? recorded : recorded < 6 ? recorded + 4 : recorded - 2,
  toRecorded: (played) =>
    played < 4 ? played : played < 8 ? played + 2 : played - 4,
};

describe("a scene played across a jump", () => {
  const across = () =>
    lanesAt(0, [{ endMs: 5000, id: "across", preset: "full", startMs: 3000 }], {
      map: MOVED,
    });

  it("is drawn as one piece, where it plays", () => {
    expect(across().lanes.scenes).toMatchObject([
      { end: 5, id: "across", start: 3 },
    ]);
  });

  it("is retimed where it plays, across the jump", () => {
    const { changed, lanes } = across();
    lanes.onSceneRetime("across", 3, 5.5);
    expect(changed).toMatchObject([[{ endMs: 5500, startMs: 3000 }]]);
  });
});

describe("an annotation moved whole on the strip", () => {
  const arrow = {
    annotation: { id: "arrow", shape: { kind: "arrow" } },
    endMs: 3_000,
    startMs: 1_000,
  } as unknown as RecordingAnnotationClip;
  const movedTo = (start: number, end: number) => {
    const lanes = lanesAt(0, [], { annotationClips: [arrow] });
    lanes.lanes.onAnnotationRetime("arrow", start, end);
    const [clip] = lanes.annotationsChanged[0] ?? [];
    return [clip.startMs, clip.endMs];
  };

  it("stops at the timeline's start at its own length", () => {
    expect(movedTo(-1, 1)).toEqual([0, 2_000]);
  });

  it("stops at the timeline's end at its own length", () => {
    expect(movedTo(9, 11)).toEqual([8_000, 10_000]);
  });
});
