import NextAuth from "next-auth";
import authConfig from "./auth.config";

export const { auth: middleware } = NextAuth(authConfig);

export const config = {
  matcher: "/^((?!^$|auth|_next|login|logout).*)$",
};
