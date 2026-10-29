// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { storyTextStyle } from "../../tool-panels/text-style/text-template-fixtures";

import { handLayout, withLayout } from "./caption-layout";
import {
  CaptionDefaults,
  DEFAULT_CAPTION_SETTINGS,
  hideCaptions,
  recordingCaptions,
  withCaptions,
} from "./recording-captions";

import type { RecordingTimelineEdit } from "../../timeline/editing/recording-timeline-edit";

const [box, pop] = [storyTextStyle("box"), storyTextStyle("pop")];
const keptForNew = { maxWords: 5 };
const defaults: CaptionDefaults = {
  layout: keptForNew,
  settings: DEFAULT_CAPTION_SETTINGS,
  style: withLayout(keptForNew, box),
};
const edit = {} as RecordingTimelineEdit;

describe("recordingCaptions", () => {
  it("starts captions not yet made with the layout kept for new ones", () => {
    const captions = recordingCaptions(edit, defaults);
    expect(captions.layout).toEqual(keptForNew);
    expect(captions.style.maxWords).toBe(5);
  });

  it("keeps captions already made free of the layout kept since", () => {
    const made = withCaptions(
      edit,
      { enabled: true },
      { ...defaults, layout: {} },
    );
    expect(recordingCaptions(made, defaults).layout).toBeUndefined();
  });
});

describe("withCaptions", () => {
  it("drops a layout with nothing set, so the edit keeps none", () => {
    const set = withCaptions(edit, { layout: { allCaps: false } }, defaults);
    const cleared = withCaptions(set, { layout: {}, style: pop }, defaults);
    expect(cleared.captions?.layout).toBeUndefined();
  });
});

describe("hideCaptions", () => {
  it("keeps captions deleted through later changes until Restore all", () => {
    const shown = withCaptions(edit, { enabled: true }, defaults);
    const hidden = hideCaptions(hideCaptions(shown, [3_000, 1_000]), [1_000]);
    expect(hidden.captions?.hidden).toEqual([1_000, 3_000]);
    const restyled = withCaptions(hidden, { style: pop }, defaults);
    expect(restyled.captions?.hidden).toEqual([1_000, 3_000]);
    const restored = withCaptions(restyled, { hidden: [] }, defaults);
    expect(restored.captions?.hidden).toBeUndefined();
  });
});

describe("withLayout", () => {
  it("keeps the options set by hand over a template and lets the rest follow it", () => {
    const chosen = withLayout({ maxWords: 8 }, pop);
    expect(chosen).toMatchObject({
      allCaps: pop.allCaps,
      lines: pop.lines,
      maxWords: 8,
      sizePercent: pop.sizePercent,
      template: "pop",
    });
  });
});

describe("handLayout", () => {
  // The next template chosen, holding more words than the one left.
  const next = { ...pop, maxWords: box.maxWords + 2 };

  it("lets an option set back to the template's own follow the next template", () => {
    const layout = handLayout({ maxWords: box.maxWords }, box);
    expect(withLayout(layout, next).maxWords).toBe(next.maxWords);
  });

  it("keeps an option that differs from the template's own", () => {
    const maxWords = box.maxWords + 5;
    expect(withLayout(handLayout({ maxWords }, box), next).maxWords).toBe(
      maxWords,
    );
  });
});
