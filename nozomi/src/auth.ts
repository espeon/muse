"use server";
// Fetch the auth token, verify it, and return the JWT payload
import { getCookiePairServer } from "@/helpers/cookie";
import { verifyJWE } from "@/helpers/verifyJwe";
import { redirect } from "next/navigation";
import { NextRequest, NextResponse } from "next/server";

interface AuthUser {
  id: string | undefined;
  name: string | undefined;
  email: string | undefined;
  image: string | undefined;
}

export async function auth(): Promise<AuthUser | null> {
  try {
    const pair = getCookiePairServer([
      "__Secure-authjs.session-token",
      "authjs.session-token",
    ]);
    let info = await verifyJWE(pair.value, pair.name);
    if (info.error) {
      // we know this'll be a string, if it exists
      throw new Error(info.error as any);
    }

    let user: AuthUser = {
      id: info.sub,
      name: info.name as string | undefined,
      email: info.email as string | undefined,
      image: info.picture as string | undefined,
    };

    return user;
  } catch (e) {
    return null;
  }
}

export async function signIn(id: string) {
  // redirect to login on the server's public URL
  let url = `${process.env.NOZOMI_BASE_URL}/auth/login/redirect`;
  return redirect(url);
}
