// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";

import { Setting } from "./setting";

describe("Setting", () => {
  it("links the control to its title and description", () => {
    const html = renderToStaticMarkup(
      <Setting description="Show the exported file." title="Open after export">
        {(props) => <button {...props}>Configure</button>}
      </Setting>,
    );
    const titleId = html.match(/id="([^"]+-title)"/)?.[1];
    const descriptionId = html.match(/id="([^"]+-description)"/)?.[1];
    if (!titleId || !descriptionId) throw new Error("Missing text IDs");
    expect(html).toContain(`aria-labelledby="${titleId}"`);
    expect(html).toContain(`aria-describedby="${descriptionId}"`);
    expect(html).not.toContain("<label");
  });

  it("omits the description association when there is no description", () => {
    const html = renderToStaticMarkup(
      <Setting title="Launch at login">
        {(props) => <button {...props}>Configure</button>}
      </Setting>,
    );
    expect(html).not.toContain("aria-describedby");
    expect(html).not.toContain("-description");
  });
});
