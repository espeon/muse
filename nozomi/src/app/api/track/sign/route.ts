import { cookies } from "next/headers";
import { NextRequest } from "next/server";

export async function GET(request: NextRequest) {
  let url = `${process.env.INTERNAL_MAKI_BASE_URL}/api/v1/track/${request.nextUrl.searchParams.get(
    "id",
  )}/sign`;

  // get jwt from cookies
  const pair = getCookiePairServer([
    "authjs.session-token",
    "__Secure-authjs.session-token",
  ]);

  console.log(pair);

  console.log(
    "Sending request to " +
      url +
      "with auth: " +
      `Bearer ${pair?.name}:${pair?.value}`,
  );

  return await fetch(url, {
    method: "GET",
    headers: {
      "Content-Type": "application/json",
      Authorization: `Bearer ${pair?.name}:${pair?.value}`,
    },
  });
}

function getCookiePairServer(names: string[]): any {
  let ck = cookies();
  for (const name of names) {
    let cookie = ck.get(name);
    if (cookie) {
      return {
        name: cookie.name,
        value: cookie.value,
      };
    }
  }
  throw new Error("No cookie found");
}
