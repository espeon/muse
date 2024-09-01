import { NextAuthConfig } from "next-auth";
import zitadel from "next-auth/providers/zitadel";

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
    authorized: async ({ auth }) => {
      // Logged in users are authenticated, otherwise redirect to login page
      return !!auth;
    },
    async signIn({ account, profile }) {
      if (account && profile && account.provider === "zitadel") {
        return true;
      }
      return true; // Do different verification for other providers that don't have `email_verified`
    },
  },
} satisfies NextAuthConfig;
