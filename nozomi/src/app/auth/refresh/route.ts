import { getCookiePairServer } from "@/helpers/cookie";
import { cookies } from "next/headers";

import { NextRequest, NextResponse } from "next/server";
import { getRefreshToken } from "./getRefreshToken";

export async function GET(request: NextRequest) {
  let ck = await cookies();
  const pair = await getCookiePairServer([
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
