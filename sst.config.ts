/// <reference path="./.sst/platform/config.d.ts" />
export default $config({
  app(input) {
    return {
      name: "nestri",
      removal: input?.stage === "production" ? "retain" : "remove",
      protect: ["production"].includes(input?.stage),
      home: "aws",
      providers: {
        aws: {
          region: "us-east-1",
          profile:
            input.stage === "production" ? "nestri-production" : "nestri-dev",
        },
        cloudflare: "6.2.0",
        random: "4.17.0",
        command: "1.0.2",
      },
    };
  },
  async run() {
    const fs = await import("fs")

    const outputs = {};
    for (const value of fs.readdirSync("./cloud/infra/")) {
      const result = await import("./cloud/infra/" + value);
      if (result.outputs) Object.assign(outputs, result.outputs);
    }
    return outputs;
  },
});
