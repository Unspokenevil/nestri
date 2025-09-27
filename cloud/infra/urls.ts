import { domain } from "./stage";

export const urls = new sst.Linkable("Urls", {
  properties: {
    api: "https://api." + domain,
    auth: "https://auth." + domain,
    site: $dev ? "http://localhost:4321" : "https://" + domain,
    openapi: "https://api." + domain + "/doc",
  },
});
