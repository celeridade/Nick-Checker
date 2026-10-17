import { drizzle, type NodePgDatabase } from "drizzle-orm/node-postgres";
import { Pool } from "pg";

// Build-safe: não joga throw no import (quebra `next build` sem .env).
// Quem precisa do banco recebe erro só no uso; /api/health retorna {ok:false}.
const databaseUrl = process.env.DATABASE_URL;

const globalForDb = globalThis as typeof globalThis & {
  __arenaNextJsPostgresqlPool?: Pool;
};

function makePool(): Pool | null {
  if (!databaseUrl) return null;
  if (globalForDb.__arenaNextJsPostgresqlPool) return globalForDb.__arenaNextJsPostgresqlPool;
  const p = new Pool({ connectionString: databaseUrl });
  if (process.env.NODE_ENV !== "production") globalForDb.__arenaNextJsPostgresqlPool = p;
  return p;
}

export const pool = makePool();

const dummyDb = new Proxy(
  {},
  {
    get() {
      throw new Error("DATABASE_URL is required");
    },
  },
) as unknown as NodePgDatabase;

export const db: NodePgDatabase = pool ? drizzle(pool) : dummyDb;
