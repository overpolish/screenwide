// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

import { beforeAll, describe, expect, it } from "vitest";

import { activate } from "./activate";
import { keyHash, type SignedRecord } from "./signing";

const KEY = "WTKP4-66NL5-HMKQW-GFSCZ";
const INSTALLATION = "0f1e2d3c4b5a69788796a5b4c3d2e1f0";
const SECRETS = {
  ruler: "secret-ruler",
  references: "secret-references",
  glide: "secret-glide",
  annotate: "secret-annotate",
};

let env: Env;
let publicKey: CryptoKey;

beforeAll(async () => {
  const pair = await crypto.subtle.generateKey(
    { name: "ECDSA", namedCurve: "P-256" },
    true,
    ["sign", "verify"],
  );
  if (!("publicKey" in pair)) throw new Error("ECDSA makes a key pair");
  publicKey = pair.publicKey;
  const pkcs8 = await crypto.subtle.exportKey("pkcs8", pair.privateKey);
  if (!(pkcs8 instanceof ArrayBuffer)) throw new Error("PKCS#8 is bytes");
  const bytes = new Uint8Array(pkcs8);
  env = {
    LICENSE_SIGNING_KEY: btoa(String.fromCharCode(...bytes)),
    PAYHIP_SECRET_RULER: SECRETS.ruler,
    PAYHIP_SECRET_REFERENCES: SECRETS.references,
    PAYHIP_SECRET_GLIDE: SECRETS.glide,
    PAYHIP_SECRET_ANNOTATE: SECRETS.annotate,
  };
});

type Reply = Response | "network error";

/** Payhip as each product would answer, by product secret; an unlisted
 * product does not know the key, and answers as Payhip does live. */
function payhip(
  replies: Partial<Record<keyof typeof SECRETS, () => Reply>>,
): typeof fetch {
  return (_input, init) => {
    const secret = new Headers(init?.headers).get("product-secret-key");
    const product = (Object.keys(SECRETS) as (keyof typeof SECRETS)[]).find(
      (name) => SECRETS[name] === secret,
    );
    const reply =
      product && replies[product] ? replies[product]() : unknownKey();
    return reply === "network error"
      ? Promise.reject(new TypeError("network error"))
      : Promise.resolve(reply);
  };
}

// Payhip's live answer for a key a product does not have.
const unknownKey = () =>
  Response.json({ data: [], error: true }, { status: 400 });

const licence = (enabled: unknown) =>
  Response.json({
    data: {
      enabled,
      license_key: KEY,
      buyer_email: "buyer@example.com",
      uses: 0,
    },
  });

function request(body: unknown): Request {
  return new Request("https://license.getscreenwide.com/activate", {
    method: "POST",
    body: typeof body === "string" ? body : JSON.stringify(body),
  });
}

const ask = (
  fetcher: typeof fetch,
  body: unknown = { licenseKey: KEY, installationId: INSTALLATION },
) => activate(request(body), env, fetcher);

describe("activate", () => {
  it("signs a record of the tool whose product knows the key", async () => {
    const response = await ask(payhip({ glide: () => licence(true) }));
    expect(response.status).toBe(200);
    const { payload, signature } = await response.json<SignedRecord>();
    const bytes = Uint8Array.from(atob(payload), (char) => char.charCodeAt(0));
    const signed = Uint8Array.from(atob(signature), (char) =>
      char.charCodeAt(0),
    );
    expect(signed.length).toBe(64);
    expect(
      await crypto.subtle.verify(
        { name: "ECDSA", hash: "SHA-256" },
        publicKey,
        signed,
        bytes,
      ),
    ).toBe(true);
    const claims: unknown = JSON.parse(new TextDecoder().decode(bytes));
    expect(claims).toEqual({
      entitlements: ["glide"],
      keyHash: await keyHash(KEY),
      installationId: INSTALLATION,
      issuedAt: expect.any(Number) as number,
      email: "buyer@example.com",
    });
  });

  it("refuses a key Payhip has disabled, as on a refund", async () => {
    const response = await ask(payhip({ ruler: () => licence(false) }));
    expect(response.status).toBe(403);
    expect(await response.json()).toEqual({ error: "disabled" });
  });

  it("calls a key invalid only when every product clearly does not know it", async () => {
    const empty = await ask(payhip({}));
    expect(empty.status).toBe(400);
    expect(await empty.json()).toEqual({ error: "invalid_key" });

    const notFound = await ask(
      payhip({
        ruler: () => new Response("", { status: 404 }),
        glide: () => Response.json({ data: [] }),
        references: () => new Response(""),
      }),
    );
    expect(await notFound.json()).toEqual({ error: "invalid_key" });
  });

  it("never calls a key invalid when Payhip could not answer for a product", async () => {
    const rejectedSecret = () =>
      Response.json(
        { code: "authentication_fail", success: false },
        { status: 401 },
      );
    for (const failure of [
      rejectedSecret,
      () => new Response("slow down", { status: 429 }),
      () => new Response("down", { status: 503 }),
      () => new Response("<html>", { status: 200 }),
      () => Response.json({ code: "something_else" }, { status: 400 }),
      (): Reply => "network error",
    ]) {
      const response = await ask(payhip({ references: failure }));
      expect(response.status).toBe(502);
      expect(await response.json()).toEqual({ error: "upstream" });
    }
  });

  it("grants the key even when another product failed to answer", async () => {
    const response = await ask(
      payhip({
        ruler: () => licence("1"),
        glide: () => new Response("down", { status: 503 }),
      }),
    );
    expect(response.status).toBe(200);
  });

  it("turns away a body without a key and an installation", async () => {
    for (const body of [
      "not json",
      {},
      { licenseKey: KEY },
      { licenseKey: " ", installationId: INSTALLATION },
      { licenseKey: "K".repeat(200), installationId: INSTALLATION },
    ]) {
      const response = await ask(payhip({ ruler: () => licence(true) }), body);
      expect(response.status).toBe(400);
      expect(await response.json()).toEqual({ error: "bad_request" });
    }
  });

  it("does not answer for a product whose secret is missing", async () => {
    const response = await activate(
      request({ licenseKey: KEY, installationId: INSTALLATION }),
      { ...env, PAYHIP_SECRET_GLIDE: "" },
      payhip({ ruler: () => licence(true) }),
    );
    expect(response.status).toBe(500);
  });
});
