import { cookies } from "next/headers";

import { NextRequest, NextResponse } from "next/server";

export interface RefreshResponse {
  success: boolean;
}

export async function GET(request: NextRequest) {
  if (request.nextUrl.searchParams.get("error")) {
    return NextResponse.json({
      success: false,
      error: request.nextUrl.searchParams.get("error"),
    });
  }

  let qparams = new URLSearchParams(request.nextUrl.search);

  const ck = await cookies();
  // basically just forward the request to the backend
  const data = await fetch(
    `${process.env.INTERNAL_MAKI_BASE_URL ?? "http://localhost:3033"}/auth/login/callback?${qparams.toString()}`,
    {
      method: "GET",
      headers: {
        "Content-Type": "application/json",
      },
    },
  ).then((res) => (res.ok ? res.json() : res.text()));

  // set session and refresh token and redirect
  if (data.session_token) {
    const { session_token } = data;
    ck.set("authjs.session-token", session_token.token, {
      httpOnly: true,
      secure: true,
      sameSite: "strict",
      path: "/",
    });

    if (data.refresh_token) {
      const { refresh_token } = data;
      ck.set("authjs.refresh-token", refresh_token.token, {
        httpOnly: true,
        secure: true,
        sameSite: "strict",
        path: "/",
      });
    }
  } else {
    return Response.json({
      success: false,
      error: "Session token invalid",
      text: data,
    });
  }

  // Redirect to home
  return NextResponse.redirect(process.env.NOZOMI_BASE_URL + "/home");
}
