import { Resource } from "sst";
import { defineConfig } from "drizzle-kit";

export default defineConfig({
  verbose: true,
  out: "./migrations",
  dialect: "postgresql",
  dbCredentials: {
    url: `postgres://${Resource.Database.user}:${Resource.Database.password}@${Resource.Database.host}/${Resource.Database.name}?sslmode=require`,
  },
  schema: "./src/**/*.sql.ts",
});
