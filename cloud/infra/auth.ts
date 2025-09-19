import { domain } from "./stage";
import { secret } from "./secrets";
import { database } from "./database";

const authStorage = new sst.cloudflare.Kv("AuthStorage");

export const auth = new sst.cloudflare.Worker("Auth", {
  handler: "cloud/packages/functions/src/auth/index.ts",
  domain: `auth.${domain}`,
  url: true,
  link: [
    database,
    authStorage,
    secret.DISCORD_CLIENT_ID,
    secret.DISCORD_CLIENT_SECRET,
  ],
});
