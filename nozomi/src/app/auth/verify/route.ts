import { getCookiePairServer } from "@/helpers/cookie";
import { verifyJWE } from "@/helpers/verifyJwe";
import { NextRequest, NextResponse } from "next/server";

export async function GET(request: NextRequest) {
  const pair = await getCookiePairServer([
    "__Secure-authjs.session-token",
    "authjs.session-token",
  ]);

  try {
    let info = await verifyJWE(pair.value, pair.name);
    console.log(info);
    return NextResponse.json({ success: true, info: info });
  } catch (error: any) {
    console.error("Error verifying JWT:", error);
    return NextResponse.json({ success: false, error: error.message });
  }
}
