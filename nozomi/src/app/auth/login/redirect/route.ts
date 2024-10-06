import { cookies } from "next/headers";

import { NextRequest, NextResponse } from "next/server";

export async function GET(request: NextRequest) {
  const data = await fetch(`${process.env.INTERNAL_MAKI_BASE_URL ?? "http://localhost:3033"}/auth/login`, {
    method: "GET",
    headers: {
      "Content-Type": "application/json",
    },
  }).then((res) => res.json());

  if (data.error) {
    return NextResponse.json({ error: data.error }, { status: 400 });
  }

  return NextResponse.redirect(data.url);
}
