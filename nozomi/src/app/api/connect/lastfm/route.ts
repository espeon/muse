import { getCookiePairServer } from "@/helpers/cookie";
import { cookies } from "next/headers";
import { NextRequest, NextResponse } from "next/server";

export async function GET() {
  const backendUrl =
    process.env.INTERNAL_MAKI_BASE_URL || "http://localhost:3000"; // Adjust as needed

  try {
    const response: any = await fetch(`${backendUrl}/api/v1/lastfm/token`);
    return response;
  } catch (error: any) {
    console.error(
      "Error getting Last.fm token:",
      error.response?.data || error.message,
    );
    return NextResponse.json(
      { error: "Internal Server Error" },
      { status: 500 },
    );
  }
}

export async function POST(request: NextRequest) {
  const backendUrl =
    process.env.INTERNAL_MAKI_BASE_URL || "http://localhost:3000"; // Adjust as needed

  try {
    const { token } = await request.json();

    console.log(token);

    // get jwt from cookies
    const pair = getCookiePairServer([
      "authjs.session-token",
      "__Secure-authjs.session-token",
    ]);

    return await fetch(`${backendUrl}/api/v1/lastfm/session?token=${token}`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Authorization: `Bearer ${pair?.name}:${pair?.value}`,
      },
    });
  } catch (error: any) {
    console.error(
      "Error getting Last.fm session:",
      error.response?.data || error.message,
    );
    return NextResponse.json(
      { error: "Internal Server Error" },
      { status: 500 },
    );
  }
}
