import { auth } from "@/auth";
import { cookies } from "next/headers";
import { NextRequest, NextResponse } from "next/server";

export async function GET(request: NextRequest) {
  // Verify we are logged in
  const session = await auth();
  if (!session || !session.id) {
    return NextResponse.redirect(process.env.NOZOMI_BASE_URL + "/auth/login");
  }

  // Logout by deleting the session from cookies.
  // TODO (VERY IMPORTANT): not secure. delete session and refresh token from database.
  cookies().delete("authjs.session-token").delete("authjs.refresh-token");

  return NextResponse.redirect(process.env.NOZOMI_BASE_URL + "/");
}
