import { NextAuthConfig } from "next-auth";
import zitadel from "next-auth/providers/zitadel";
import { cookies } from "next/headers";

function getDomain() {
  // use env variable if available
  if (process.env.COOKIE_DOMAIN) {
    return process.env.COOKIE_DOMAIN;
  }
  // otherwise use the hostname
  if (process.env.NOZOMI_BASE_URL) {
    let hostname = new URL(process.env.NOZOMI_BASE_URL).hostname;
    if (!hostname.includes("localhost")) {
      return "." + hostname;
    } else {
      return "localhost";
    }
  }
}

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
  cookies: {
    sessionToken: {
      name: `authjs.session-token`,
      options: {
        domain: getDomain(), // Makes the cookie accessible to subdomains
        path: "/", // Root path for cookie
        sameSite: "lax", // Or 'strict'/'none' depending on your needs
        secure: process.env.NODE_ENV === "production", // Secure only in production
      },
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
