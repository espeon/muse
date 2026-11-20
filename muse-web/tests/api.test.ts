import { afterAll, beforeAll, expect, test } from "bun:test";

let api: typeof import("../src/lib/api");
const requests: Request[] = [];
const server = Bun.serve({
  hostname: "127.0.0.1",
  port: 0,
  fetch(request) {
    requests.push(request);
    switch (new URL(request.url).pathname) {
      case "/api/v1/lastfm/session":
        return new Response(null, { status: 204 });
      case "/api/v1/album":
        return Response.json({ albums: [], offset: 0, limit: 20 });
      case "/api/v1/artist":
        return Response.json({ artists: [], cursor: 0, limit: 20 });
      default:
        return new Response("Unknown route", { status: 404 });
    }
  },
});

beforeAll(async () => {
  const original = process.env.VITE_MAKI_URL;
  try {
    process.env.VITE_MAKI_URL = server.url.origin;
    api = await import("../src/lib/api");
  } finally {
    if (original === undefined) delete process.env.VITE_MAKI_URL;
    else process.env.VITE_MAKI_URL = original;
  }
});

afterAll(() => server.stop(true));

test("Last.fm disconnect accepts Maki's empty successful response", async () => {
  const fetch = globalThis.fetch;
  try {
    Object.assign(globalThis, {
      async fetch(...args: Parameters<typeof fetch>) {
        const response = await fetch(...args);
        if (response.status === 204) {
          // Browsers reject JSON parsing on an empty response body.
          response.json = async () => { throw new SyntaxError("Unexpected end of JSON input"); };
        }
        return response;
      },
    });
    await expect(api.disconnectLastfm()).resolves.toBeUndefined();
    expect(requests.at(-1)?.method).toBe("DELETE");
  } finally {
    globalThis.fetch = fetch;
  }
});

test("album and artist searches send the full filter and pagination cursor", async () => {
  await api.getAlbums(20, "海 & Sky");
  await api.getArtists(35, "海 & Sky");

  const urls = requests.slice(-2).map((request) => new URL(request.url));
  expect(urls.map((url) => [url.pathname, url.searchParams.get("cursor"), url.searchParams.get("filter")]))
    .toEqual([["/api/v1/album", "20", "海 & Sky"], ["/api/v1/artist", "35", "海 & Sky"]]);
});

test("a pending session refresh cannot restore the previous account", async () => {
  const auth = await import("../src/lib/auth");
  const browserGlobals = Object.getOwnPropertyDescriptors(globalThis);
  const location = { hash: "", pathname: "/auth/callback" };
  const refreshed = Promise.withResolvers<Response>();
  Object.assign(globalThis, {
    window: { location },
    history: { replaceState() {} },
    fetch: async (url: string) => url.endsWith("/auth/refresh")
      ? refreshed.promise
      : new Response(null, { status: 204 }),
  });
  const future = Math.floor(Date.now() / 1000) + 3600;
  try {
    location.hash = "#session_token=first&refresh_token=first-refresh&session_expiry=0";
    expect(auth.consumeCallbackFragment()).toBe(true);
    const pending = auth.getValidSessionToken();

    location.hash = `#session_token=second&refresh_token=second-refresh&session_expiry=${future}`;
    expect(auth.consumeCallbackFragment()).toBe(true);
    refreshed.resolve(Response.json({ session_token: "first-refreshed", expiry: future }));

    await expect(pending).resolves.toBeNull();
    await expect(auth.getValidSessionToken()).resolves.toBe("second");
  } finally {
    auth.logout();
    for (const name of ["window", "history", "fetch"]) {
      const descriptor = browserGlobals[name];
      if (descriptor) Object.defineProperty(globalThis, name, descriptor);
      else Reflect.deleteProperty(globalThis, name);
    }
  }
});
