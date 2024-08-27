import NextAuth from "next-auth";
import PostgresAdapter from "@auth/pg-adapter";
import { Pool, PoolConfig } from "pg";
import authConfig from "./auth.config";

const pool = new Pool({
  max: 20,
  idleTimeoutMillis: 30000,
  connectionTimeoutMillis: 2000,
  ...buildPgConfig(process.env.DATABASE_URL as string),
});

export const { handlers, auth, signIn, signOut } = NextAuth({
  session: { strategy: "jwt" },
  adapter: PostgresAdapter(pool),
  ...authConfig,
});

function buildPgConfig(url: string): PoolConfig {
  try {
    const regex =
      /^postgres:\/\/(?:([^:]+)(?::([^@]+))?@)?([^:\/]+)(?::(\d+))?\/(.+?)(?:\?(.+))?$/;
    const match = url.match(regex);
    if (!match) {
      throw new Error("Invalid PostgreSQL connection URL");
    }

    const [, user, password, host, port, database, queryString] = match;

    let config: PoolConfig = {
      host,
      user,
      password,
      database,
      port: port ? parseInt(port, 10) : 5432,
    };

    if (queryString) {
      const params = new URLSearchParams(queryString);
      params.forEach((value, key) => {
        if (key === "ssl") {
          config.ssl = value === "true" ? true : { rejectUnauthorized: false };
        } else {
          (config as any)[key] = value;
        }
      });
    }

    return config;
  } catch (e) {
    console.log(e);
    throw new Error("Error when building pg config");
  }
}
