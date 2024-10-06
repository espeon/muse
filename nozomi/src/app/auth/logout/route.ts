import { auth } from "@/auth";
import { cookies } from "next/headers";
import { NextRequest, NextResponse } from "next/server";

export async function GET(request: NextRequest) {
  // Verify we are logged in
  const session = await auth();
  if (!session || !session.id) {
    return NextResponse.redirect(process.env.NOZOMI_BASE_URL ?? "http://localhost:3031" + "/auth/login");
  }

  let refreshToken = cookies().get("authjs.refresh-token")?.value;
  if (!refreshToken) {
    return NextResponse.redirect(process.env.NOZOMI_BASE_URL ?? "http://localhost:3031" + "/auth/login");
  }

  // Logout by deleting the session from cookies.
  cookies().delete("authjs.session-token").delete("authjs.refresh-token");

  // Delete the refresh token from the database
  const data = await fetch(
    `${process.env.INTERNAL_MAKI_BASE_URL ?? "http://localhost:3033"}/auth/refresh`,
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

  return NextResponse.redirect(process.env.NOZOMI_BASE_URL + "/");
}
