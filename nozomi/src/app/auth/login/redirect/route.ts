import { NextRequest, NextResponse } from "next/server";
// Force dynamic. we do NOT want to cache this page
export const dynamic = "force-dynamic";
export async function GET(request: NextRequest) {
  const data = await fetch(
    `${process.env.INTERNAL_MAKI_BASE_URL ?? "http://localhost:3033"}/auth/login`,
    {
      method: "GET",
      headers: {
        "Content-Type": "application/json",
      },
      cache: "no-store",
    },
  ).then((res) => res.json());

  if (data.error) {
    return NextResponse.json({ error: data.error }, { status: 400 });
  }

  return NextResponse.redirect(data.url);
}
