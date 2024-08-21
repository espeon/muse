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
  basePath: "/auth",
  callbacks: {
    async jwt({ token, account, profile }) {
      // Check if account and profile exist before accessing their properties
      if (account && profile) {
        token.email = profile.email;
        token.name = profile.name;
        token.picture = profile.picture;
      }
      return token;
    },
    async signIn({ account, profile }) {
      if (account && profile && account.provider === "zitadel") {
        return true;
      }
      return true; // Do different verification for other providers that don't have `email_verified`
    },
  },
} satisfies NextAuthConfig;
