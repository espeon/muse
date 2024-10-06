import { getCookiePairServer } from "@/helpers/cookie";
import { cookies } from "next/headers";

import { NextRequest, NextResponse } from "next/server";

export interface RefreshResponse {
  success: boolean;
}

export async function getRefreshToken(refreshToken: string) {
  // Get the access token
  const data = await fetch(
    `${process.env.INTERNAL_MAKI_BASE_URL}/auth/refresh`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Authorization: `Bearer ${refreshToken}`,
      },
    },
  )
    .then((res) => res.json())
    .catch((err) => {
      console.error("Error refreshing token:", err.message);
      return { success: false };
    });

  console.log("Refresh token response:", data);
  return data;
}

export async function GET(request: NextRequest) {
  let ck = cookies();
  const pair = getCookiePairServer([
    "__Secure-authjs.refresh-token",
    "authjs.refresh-token",
  ]);
  let data = await getRefreshToken(pair.value);
  if (data.session_token) {
    const { session_token } = data;
    ck.set("authjs.session-token", session_token.token, {
      httpOnly: true,
      secure: true,
      sameSite: "strict",
      path: "/",
    });
  } else {
    return data;
  }

  return NextResponse.json({ success: true });
}
