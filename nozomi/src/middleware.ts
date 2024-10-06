import { NextRequest, NextResponse } from "next/server";
import { getRefreshToken } from "@/app/auth/refresh/getRefreshToken";
import { verifyJWE } from "./helpers/verifyJwe";

async function verifyAccessToken(request: NextRequest) {
  const cookies = request.headers.get("cookie");
  const accessCookie = request.cookies.get("authjs.session-token")?.value;

  if (!accessCookie) return null;

  const v = await verifyJWE(accessCookie, "authjs.session-token");
  return v;
}

async function refreshAccessToken(request: NextRequest) {
  const refreshCookie = request.cookies.get("authjs.refresh-token")?.value;
  if (!refreshCookie) return null;

  const res = await getRefreshToken(refreshCookie);

  return res.session_token;
}

export async function middleware(request: NextRequest) {
  try {
    const user = await verifyAccessToken(request);
    if (user) {
      return NextResponse.next();
    } else {
      throw new Error("No access token");
    }
  } catch (e) {
    try {
      const newAccess = await refreshAccessToken(request);
      if (!newAccess)
        throw new Error("No new access token given. Likely no refresh token.");
      const response = new NextResponse();
      response.cookies.set("authjs.session-token", newAccess, {
        httpOnly: true,
        maxAge: 10 * 60 * 10, // 10 minutes
      });
      // redirect to same page, but with new access token
      return response;
    } catch (e) {
      console.log("No refresh token, redirecting to sign in...");
      const response = NextResponse.redirect(
        process.env.NOZOMI_BASE_URL + "/auth/login",
      );
      response.cookies.delete("authjs.session-token");
      response.cookies.delete("authjs.refresh-token");
      return response;
    }
  }
}

export const config = {
  matcher: [
    /*
     * Match all request paths except for the ones starting with:
     * - api (API routes)
     * - auth (authentication routes should verify on their own)
     * - sw (service worker)
     * - icons (icons)
     * - _next/static (static files)
     * - _next/image (image optimization files)
     * - favicon.ico, sitemap.xml, manifest.json, robots.txt (metadata files)
     */
    "/((?!api|auth|sw|icons|_next/static|_next/image|favicon.ico|sitemap.xml|manifest.json|robots.txt).*)",
  ],
};
