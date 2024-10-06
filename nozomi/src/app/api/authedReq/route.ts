import { getCookiePairServer } from "@/helpers/cookie";
import { cookies } from "next/headers";
import { NextRequest, NextResponse } from "next/server";

export async function GET(request: NextRequest) {
  const backendUrl =
    process.env.INTERNAL_MAKI_BASE_URL || "http://localhost:3000";

  const route = request.nextUrl.searchParams.get("route");
  console.log(route);

  // get jwt from cookies
  const pair = getCookiePairServer([
    "__Secure-authjs.session-token",
    "authjs.session-token",
  ]);

  try {
    const response: any = await fetch(`${backendUrl}/${route}`, {
      headers: {
        "Content-Type": "application/json",
        Authorization: `Bearer ${pair?.name}:${pair?.value}`,
      },
    });
    return response;
  } catch (error: any) {
    console.error(
      "Error fetching data from backend route:",
      error.response?.data || error.message,
    );
    return NextResponse.json(
      { error: "Internal Server Error" },
      { status: 500 },
    );
  }
}
