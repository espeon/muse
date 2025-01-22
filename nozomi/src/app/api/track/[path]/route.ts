import { getCookiePairServer } from "@/helpers/cookie";
import { cookies } from "next/headers";
import { NextRequest } from "next/server";

export async function POST(
  request: NextRequest,
  {
    params,
  }: {
    params: Promise<{ path: string }>;
  },
) {
  const { path } = await params;
  let url = `${process.env.INTERNAL_MAKI_BASE_URL}/api/v1/track/${request.nextUrl.searchParams.get(
    "id",
  )}/${path}`;

  // get jwt from cookies
  const pair = await getCookiePairServer([
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
