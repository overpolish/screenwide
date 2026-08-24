// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { describe, expect, it } from "vitest";

import { classifyQrPayload } from "./qr-code-payload";

describe("classifyQrPayload", () => {
  it.each([
    ["https://screenwide.app", "link"],
    ["http://localhost:1420", "link"],
    ["tel:+442071234567", "phone"],
    ["mailto:hello@example.com?subject=Hello", "email"],
    ["sms:+442071234567?body=Hello", "sms"],
  ] as const)("classifies the supported action %s", (content, action) => {
    expect(classifyQrPayload(content)).toMatchObject({
      action,
      kind: "action",
    });
  });

  it.each([
    ["Some plain text", "Text"],
    ["WIFI:T:WPA;S:Studio;P:secret;;", "Wi-Fi network"],
    ["BEGIN:VCARD\nVERSION:3.0\nFN:Dom\nEND:VCARD", "Contact card"],
    ["BEGIN:VEVENT\nSUMMARY:Demo\nEND:VEVENT", "Calendar event"],
  ])("classifies informational content", (content, label) => {
    expect(classifyQrPayload(content)).toEqual({
      kind: "information",
      label,
    });
  });

  it.each([
    ["tel:", "Phone QR is missing a number."],
    ["WIFI:T:WPA;P:secret;;", "Wi-Fi QR is missing a network name."],
    ["javascript:alert(1)", "The javascript action is not supported."],
    ["", "QR code has no content."],
  ])("rejects malformed or unsupported content", (content, reason) => {
    expect(classifyQrPayload(content)).toEqual({
      kind: "unsupported",
      label: "Unsupported QR",
      reason,
    });
  });

  it("reports a detected QR-like code that could not be decoded", () => {
    expect(classifyQrPayload("", "QR-like code could not be decoded.")).toEqual(
      {
        kind: "unsupported",
        label: "Unsupported QR",
        reason: "QR-like code could not be decoded.",
      },
    );
  });
});
