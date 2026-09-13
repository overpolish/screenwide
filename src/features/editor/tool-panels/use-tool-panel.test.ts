// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import {
  anchor,
  cursorPanelOpen,
  mocks,
  setViewportWidth,
  useCursorPanel,
  usePanel,
} from "./tool-panel-test-fixtures";

describe("useToolPanel", () => {
  it("grows a narrow viewport to the complete panel target before fitting", async () => {
    mocks.fitWidth = "960";
    setViewportWidth(400);
    mocks.openToolPanelSpace.mockImplementation(() => {
      setViewportWidth(1280);
      return Promise.resolve();
    });
    await useCursorPanel();
    expect(mocks.openToolPanelSpace).toHaveBeenCalledExactlyOnceWith(880);
    expect(mocks.fitDuringResize).toHaveBeenCalledWith(
      320,
      expect.any(Function),
    );
    expect(mocks.fitPreview).not.toHaveBeenCalled();
  });

  it("fits and grows once when an opted-in panel opens", async () => {
    mocks.openToolPanelSpace.mockImplementation(() => {
      setViewportWidth(1220);
      return Promise.resolve();
    });
    await useCursorPanel();

    expect(mocks.openToolPanelSpace).toHaveBeenCalledWith(320);
    expect(mocks.fitDuringResize).toHaveBeenCalledWith(
      320,
      expect.any(Function),
    );
    expect(mocks.fitPreview).not.toHaveBeenCalled();
    expect(mocks.open).toHaveBeenCalledOnce();
    expect(mocks.showPopupPanel).toHaveBeenCalledOnce();
  });

  it("requests growth again after closing and reopening at the previous fitted width", async () => {
    mocks.openToolPanelSpace.mockImplementation(() => {
      setViewportWidth(1120);
      return Promise.resolve();
    });
    await useCursorPanel();
    mocks.active = cursorPanelOpen;
    await usePanel("recording").toggle("cursor", anchor);
    expect(mocks.openToolPanelSpace).toHaveBeenCalledOnce();
    mocks.active = null;
    await useCursorPanel();
    expect(mocks.openToolPanelSpace).toHaveBeenCalledTimes(2);
    expect(mocks.openToolPanelSpace).toHaveBeenLastCalledWith(320);
  });

  it("opens an overlay panel without growing or fitting", async () => {
    mocks.panelByTool.select = "cursor";
    mocks.resetByTool.cursor = true;
    mocks.resetByTool.select = false;
    const panel = usePanel("recording");

    await panel.toggle("select", anchor);

    expect(mocks.openToolPanelSpace).not.toHaveBeenCalled();
    expect(mocks.fitPreview).not.toHaveBeenCalled();
    expect(mocks.open).toHaveBeenCalledOnce();
  });

  it("closes an open panel without changing the view", async () => {
    mocks.active = cursorPanelOpen;
    const panel = usePanel("recording");

    await panel.toggle("cursor", anchor);

    expect(mocks.close).toHaveBeenCalledOnce();
    expect(mocks.hidePopupPanel).toHaveBeenCalledOnce();
    expect(mocks.fitPreview).not.toHaveBeenCalled();
    expect(mocks.openToolPanelSpace).not.toHaveBeenCalled();
  });

  it("hands the reset basis back to the full viewport on closing", async () => {
    mocks.active = cursorPanelOpen;
    mocks.panelByTool.cursor = "cursor";
    mocks.resetByTool.cursor = true;

    await usePanel("recording").toggle("cursor", anchor);

    expect(mocks.setFitBasis).toHaveBeenCalledOnce();
    expect(mocks.setFitBasis).toHaveBeenCalledWith();
    expect(mocks.fitPreview).not.toHaveBeenCalled();
  });

  it("leaves the panel's own basis alone while it opens", async () => {
    mocks.openToolPanelSpace.mockImplementation(() => {
      setViewportWidth(1220);
      return Promise.resolve();
    });

    await useCursorPanel();

    expect(mocks.fitDuringResize).toHaveBeenCalledWith(
      320,
      expect.any(Function),
    );
    expect(mocks.fitPreview).not.toHaveBeenCalled();
    expect(mocks.setFitBasis).not.toHaveBeenCalled();
  });

  it("switches tools according to the destination policy", async () => {
    mocks.active = {
      content: { kind: "tool", tool: "keyboard", workspace: "recording" },
      id: "tool:keyboard",
    };
    mocks.panelByTool.cursor = "cursor";
    mocks.resetByTool.cursor = true;
    const panel = usePanel("recording");

    await panel.toggle("cursor", anchor);
    expect(mocks.fitDuringResize).toHaveBeenCalledWith(
      320,
      expect.any(Function),
    );
    expect(mocks.fitPreview).not.toHaveBeenCalled();
    expect(mocks.open).toHaveBeenCalledOnce();
    expect(mocks.showPopupPanel).toHaveBeenCalledOnce();
    expect(mocks.hidePopupPanel).not.toHaveBeenCalled();
  });

  it("opens in its own workspace's panel window, leaving the other alone", async () => {
    mocks.active = cursorPanelOpen;
    mocks.panelByTool.cursor = "cursor";
    mocks.resetByTool.cursor = false;

    await usePanel("screenshot").toggle("cursor", anchor);

    expect(mocks.close).not.toHaveBeenCalled();
    expect(mocks.hidePopupPanel).not.toHaveBeenCalled();
    const panel = "tool-panel-screenshot";
    expect(mocks.open).toHaveBeenCalledExactlyOnceWith(panel, {
      content: { kind: "tool", tool: "cursor", workspace: "screenshot" },
      focusContents: false,
      id: "tool:cursor",
    });
    expect(mocks.showPopupPanel).toHaveBeenCalledWith(
      expect.objectContaining({ panel }),
    );
  });

  it("does not refit when an already open panel is toggled closed", async () => {
    mocks.active = cursorPanelOpen;
    mocks.panelByTool.cursor = "cursor";
    mocks.resetByTool.cursor = true;
    const panel = usePanel("recording");

    await panel.toggle("cursor", anchor);

    expect(mocks.fitPreview).not.toHaveBeenCalled();
    expect(mocks.openToolPanelSpace).not.toHaveBeenCalled();
  });
});
