// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { storyCaptionStyle } from "../../tool-panels/captions/caption-template-fixtures";

import { DEFAULT_CAPTION_SETTINGS } from "./recording-captions";
import { captionDefaultsSettings } from "./use-caption-templates";

const [box, pop] = [storyCaptionStyle("box"), storyCaptionStyle("pop")];

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
