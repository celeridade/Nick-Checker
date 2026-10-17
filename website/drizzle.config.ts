import { defineConfig } from "drizzle-kit";

// Lê DATABASE_URL do ambiente (.env.local no dev) em vez de
// credencial fixa — `npx drizzle-kit push` usa a senha real.
export default defineConfig({
  dialect: "postgresql",
  schema: "./src/db/schema.ts",
  out: "./drizzle",
  dbCredentials: {
    url: process.env.DATABASE_URL ?? "postgresql://postgres:postgres@127.0.0.1:5432/app_db",
  },
  verbose: true,
});
