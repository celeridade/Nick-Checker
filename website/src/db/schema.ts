import {
  boolean,
  doublePrecision,
  integer,
  jsonb,
  pgTable,
  serial,
  text,
  timestamp,
} from "drizzle-orm/pg-core";

/** Usuários + plano/tokens (pricing 25/mês free). */
export const users = pgTable("users", {
  id: serial("id").primaryKey(),
  email: text("email").notNull().unique(),
  passwordHash: text("password_hash"),
  provider: text("provider").notNull().default("email"),
  plan: text("plan").notNull().default("free"),
  role: text("role").notNull().default("user"),
  tokensUsed: integer("tokens_used").notNull().default(0),
  tokensLimit: integer("tokens_limit").notNull().default(25),
  tokensResetAt: timestamp("tokens_reset_at", { withTimezone: true }),
  createdAt: timestamp("created_at", { withTimezone: true }).notNull().defaultNow(),
});

/** Uma busca single (q=1 nick) — histórico real (era mock em /api/history). */
export const searches = pgTable("searches", {
  id: serial("id").primaryKey(),
  userId: integer("user_id").references(() => users.id),
  query: text("query").notNull(),
  queryType: text("query_type").notNull().default("username"),
  total: integer("total").notNull().default(0),
  hits: integer("hits").notNull().default(0),
  status: text("status").notNull().default("completed"),
  createdAt: timestamp("created_at", { withTimezone: true }).notNull().defaultNow(),
});

export const searchResults = pgTable("search_results", {
  id: serial("id").primaryKey(),
  searchId: integer("search_id")
    .notNull()
    .references(() => searches.id, { onDelete: "cascade" }),
  platform: text("platform").notNull(),
  username: text("username").notNull(),
  profileUrl: text("profile_url").notNull().default(""),
  result: text("result").notNull().default("taken"),
  durationMs: integer("duration_ms").notNull().default(0),
  raw: jsonb("raw"),
});

/** Jobs bulk/dataset (POST /v1/scans) — polling + SSE. */
export const scans = pgTable("scans", {
  id: text("id").primaryKey(),
  userId: integer("user_id").references(() => users.id),
  source: text("source").notNull().default("bulk"),
  usernames: jsonb("usernames").notNull().default([]),
  platforms: text("platforms").notNull().default("all"),
  status: text("status").notNull().default("running"),
  total: integer("total").notNull().default(0),
  checked: integer("checked").notNull().default(0),
  hits: integer("hits").notNull().default(0),
  results: jsonb("results").notNull().default([]),
  createdAt: timestamp("created_at", { withTimezone: true }).notNull().defaultNow(),
  finishedAt: timestamp("finished_at", { withTimezone: true }),
});

/** Watchlist do sniper (migra watchlist.txt). Bot edita, worker lê. */
export const watchlist = pgTable("watchlist", {
  id: serial("id").primaryKey(),
  username: text("username").notNull().unique(),
  platforms: text("platforms").notNull().default("all"),
  addedBy: text("added_by"),
  createdAt: timestamp("created_at", { withTimezone: true }).notNull().defaultNow(),
});

/** Pool de proxies (espelha proxies.txt + EMA do Rotator). */
export const proxies = pgTable("proxies", {
  id: serial("id").primaryKey(),
  url: text("url").notNull().unique(),
  protocol: text("protocol").notNull().default("http"),
  latencyEmaMs: doublePrecision("latency_ema_ms").notNull().default(9999),
  success: integer("success").notNull().default(0),
  fails: integer("fails").notNull().default(0),
  healthy: boolean("healthy").notNull().default(true),
  updatedAt: timestamp("updated_at", { withTimezone: true }).notNull().defaultNow(),
});

export const webhooks = pgTable("webhooks", {
  id: serial("id").primaryKey(),
  name: text("name").notNull().default("default"),
  url: text("url").notNull(),
  mode: text("mode").notNull().default("batch"),
  batchSize: integer("batch_size").notNull().default(5),
  createdAt: timestamp("created_at", { withTimezone: true }).notNull().defaultNow(),
});

/** Bot state (guild, verify channel, cargos) + settings genéricas. */
export const botState = pgTable("bot_state", {
  id: serial("id").primaryKey(),
  guildId: text("guild_id").notNull().default(""),
  verifyChannelId: text("verify_channel_id").notNull().default(""),
  status: text("status").notNull().default("stopped"),
  updatedAt: timestamp("updated_at", { withTimezone: true }).notNull().defaultNow(),
});

export const settings = pgTable("settings", {
  key: text("key").primaryKey(),
  value: text("value").notNull().default(""),
});
