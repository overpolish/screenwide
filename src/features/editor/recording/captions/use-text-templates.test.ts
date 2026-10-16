// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { storyTextStyle } from "../../tool-panels/text-style/text-template-fixtures";

import { DEFAULT_CAPTION_SETTINGS } from "./recording-captions";
import { captionDefaultsSettings } from "./use-text-templates";

const [box, pop] = [storyTextStyle("box"), storyTextStyle("pop")];

describe("captionDefaultsSettings", () => {
  it("sets a template, looking as it does now, as the default", () => {
    const savedPop = { ...pop, sizePercent: 120 };
    const restyled = { ...pop, sizePercent: 160 };
    expect(
      captionDefaultsSettings([box, savedPop], { style: restyled }),
    ).toEqual({ captionStyles: [box, restyled], captionTemplate: "pop" });
  });

  it("puts a template back to how it ships and keeps the default", () => {
    expect(captionDefaultsSettings([box, pop], { original: "pop" })).toEqual({
      captionStyles: [box],
    });
  });

  it("keeps the rules new captions start with", () => {
    const settings = { ...DEFAULT_CAPTION_SETTINGS, hideFillers: true };
    expect(captionDefaultsSettings([box], { settings })).toEqual({
      captionSettings: settings,
    });
  });

  it("keeps the layout set by hand for new captions", () => {
    expect(
      captionDefaultsSettings([box], { layout: { lines: "one" } }),
    ).toEqual({ captionLayout: { lines: "one" } });
  });
});
