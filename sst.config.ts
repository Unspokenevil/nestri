/// <reference path="./.sst/platform/config.d.ts" />
export default $config({
  app(input) {
    return {
      name: "nestri",
      removal: input?.stage === "production" ? "retain" : "remove",
      protect: ["production"].includes(input?.stage),
      home: "cloudflare",
      providers: {
        cloudflare: "6.6.0",
        random: "4.17.0",
        command: "1.0.2",
        neon: "0.9.0",
      },
    };
  },
  async run() {
    const fs = await import("fs");
    const outputs = {};
    for (const value of fs.readdirSync("./cloud/infra/")) {
      const result = await import("./cloud/infra/" + value);
      if (result.outputs) Object.assign(outputs, result.outputs);
    }
    return outputs;
  },
});
