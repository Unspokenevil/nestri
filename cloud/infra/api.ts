import { urls } from "./urls";
import { auth } from "./auth";
import { domain } from "./stage";
import { secret } from "./secrets";
import { database } from "./database";

export const api = new sst.cloudflare.Worker("Api", {
  url: true,
  domain: `api.${domain}`,
  handler: "cloud/packages/functions/src/api/index.ts",
  link: [database, secret.POLAR_API_KEY, urls, auth],
});
