import { NextAuthConfig } from "next-auth";
import zitadel from "next-auth/providers/zitadel";
import { cookies } from "next/headers";

export default {
  providers: [
    zitadel({
      issuer: process.env.ZITADEL_ISSUER,
      clientId: process.env.ZITADEL_CLIENT_ID,
      clientSecret: process.env.ZITADEL_CLIENT_SECRET,
    }),
  ],
  trustHost: true,
  basePath: "/auth",
  pages: {
    signIn: "/login",
  },
  callbacks: {
    authorized: async ({ auth, request }) => {
      // Logged in users are authenticated, otherwise redirect to login page
      return !!auth;
    },
    async signIn({ user, account, profile, email, credentials }) {
      if (!account) {
        return false;
      }
      // Store the provider in a cookie
      setCookie("lutea_lastUsedProvider", account.provider, {
        maxAge: 30 * 24 * 60 * 60, // 30 days
        path: "/",
      });
      // if (account && profile && account.provider === "zitadel") {
      //   return true;
      // }
      return true;
    },
  },
} satisfies NextAuthConfig;

function setCookie(
  arg1: string,
  provider: any,
  arg3: {
    maxAge: number; // 30 days
    path: string;
  },
) {
  // document.cookie
  cookies().set(arg1, provider, {
    maxAge: arg3.maxAge,
    path: arg3.path,
  });
}
