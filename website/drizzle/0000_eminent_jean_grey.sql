CREATE TABLE "bot_state" (
	"id" serial PRIMARY KEY NOT NULL,
	"guild_id" text DEFAULT '' NOT NULL,
	"verify_channel_id" text DEFAULT '' NOT NULL,
	"status" text DEFAULT 'stopped' NOT NULL,
	"updated_at" timestamp with time zone DEFAULT now() NOT NULL
);
--> statement-breakpoint
CREATE TABLE "proxies" (
	"id" serial PRIMARY KEY NOT NULL,
	"url" text NOT NULL,
	"protocol" text DEFAULT 'http' NOT NULL,
	"latency_ema_ms" double precision DEFAULT 9999 NOT NULL,
	"success" integer DEFAULT 0 NOT NULL,
	"fails" integer DEFAULT 0 NOT NULL,
	"healthy" boolean DEFAULT true NOT NULL,
	"updated_at" timestamp with time zone DEFAULT now() NOT NULL,
	CONSTRAINT "proxies_url_unique" UNIQUE("url")
);
--> statement-breakpoint
CREATE TABLE "scans" (
	"id" text PRIMARY KEY NOT NULL,
	"user_id" integer,
	"source" text DEFAULT 'bulk' NOT NULL,
	"usernames" jsonb DEFAULT '[]'::jsonb NOT NULL,
	"platforms" text DEFAULT 'all' NOT NULL,
	"status" text DEFAULT 'running' NOT NULL,
	"total" integer DEFAULT 0 NOT NULL,
	"checked" integer DEFAULT 0 NOT NULL,
	"hits" integer DEFAULT 0 NOT NULL,
	"results" jsonb DEFAULT '[]'::jsonb NOT NULL,
	"created_at" timestamp with time zone DEFAULT now() NOT NULL,
	"finished_at" timestamp with time zone
);
--> statement-breakpoint
CREATE TABLE "search_results" (
	"id" serial PRIMARY KEY NOT NULL,
	"search_id" integer NOT NULL,
	"platform" text NOT NULL,
	"username" text NOT NULL,
	"profile_url" text DEFAULT '' NOT NULL,
	"result" text DEFAULT 'taken' NOT NULL,
	"duration_ms" integer DEFAULT 0 NOT NULL,
	"raw" jsonb
);
--> statement-breakpoint
CREATE TABLE "searches" (
	"id" serial PRIMARY KEY NOT NULL,
	"user_id" integer,
	"query" text NOT NULL,
	"query_type" text DEFAULT 'username' NOT NULL,
	"total" integer DEFAULT 0 NOT NULL,
	"hits" integer DEFAULT 0 NOT NULL,
	"status" text DEFAULT 'completed' NOT NULL,
	"created_at" timestamp with time zone DEFAULT now() NOT NULL
);
--> statement-breakpoint
CREATE TABLE "settings" (
	"key" text PRIMARY KEY NOT NULL,
	"value" text DEFAULT '' NOT NULL
);
--> statement-breakpoint
CREATE TABLE "users" (
	"id" serial PRIMARY KEY NOT NULL,
	"email" text NOT NULL,
	"password_hash" text,
	"provider" text DEFAULT 'email' NOT NULL,
	"plan" text DEFAULT 'free' NOT NULL,
	"tokens_used" integer DEFAULT 0 NOT NULL,
	"tokens_limit" integer DEFAULT 25 NOT NULL,
	"created_at" timestamp with time zone DEFAULT now() NOT NULL,
	CONSTRAINT "users_email_unique" UNIQUE("email")
);
--> statement-breakpoint
CREATE TABLE "watchlist" (
	"id" serial PRIMARY KEY NOT NULL,
	"username" text NOT NULL,
	"platforms" text DEFAULT 'all' NOT NULL,
	"added_by" text,
	"created_at" timestamp with time zone DEFAULT now() NOT NULL,
	CONSTRAINT "watchlist_username_unique" UNIQUE("username")
);
--> statement-breakpoint
CREATE TABLE "webhooks" (
	"id" serial PRIMARY KEY NOT NULL,
	"name" text DEFAULT 'default' NOT NULL,
	"url" text NOT NULL,
	"mode" text DEFAULT 'batch' NOT NULL,
	"batch_size" integer DEFAULT 5 NOT NULL,
	"created_at" timestamp with time zone DEFAULT now() NOT NULL
);
--> statement-breakpoint
ALTER TABLE "scans" ADD CONSTRAINT "scans_user_id_users_id_fk" FOREIGN KEY ("user_id") REFERENCES "public"."users"("id") ON DELETE no action ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "search_results" ADD CONSTRAINT "search_results_search_id_searches_id_fk" FOREIGN KEY ("search_id") REFERENCES "public"."searches"("id") ON DELETE cascade ON UPDATE no action;--> statement-breakpoint
ALTER TABLE "searches" ADD CONSTRAINT "searches_user_id_users_id_fk" FOREIGN KEY ("user_id") REFERENCES "public"."users"("id") ON DELETE no action ON UPDATE no action;