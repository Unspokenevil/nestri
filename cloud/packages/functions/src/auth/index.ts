import { Select } from "./ui";
import { Resource } from "sst";
import { logger } from "hono/logger";
import { subjects } from "../subjects";
import { handleDiscord } from "./utils";
import { DiscordAdapter } from "./adapters";
import { issuer } from "@openauthjs/openauth";
import { User } from "@nestri/core/user/index";
import { patchLogger } from "../utils/patch-logger";
import type { KVNamespace } from "@cloudflare/workers-types";
import { CloudflareStorage } from "@openauthjs/openauth/storage/cloudflare";

interface Env {
  AuthStorage: KVNamespace;
}

patchLogger();
export default {
  async fetch(request: Request, env: Env, ctx: ExecutionContext) {
    return issuer({
      select: Select(),
      theme: {
        title: "Nestri | Auth",
        primary: "#FF4F01",
        //TODO: Change this in prod
        logo: "https://nestri.io/logo.webp",
        favicon: "https://nestri.io/seo/favicon.ico",
        background: {
          light: "#F5F5F5",
          dark: "#171717",
        },
        radius: "lg",
        font: {
          family: "Geist, sans-serif",
        },
        css: `@import url('https://fonts.googleapis.com/css2?family=Geist:wght@100;200;300;400;500;600;700;800;900&display=swap');`,
      },
      subjects,
      storage: CloudflareStorage({
        namespace: env.AuthStorage,
      }),
      providers: {
        discord: DiscordAdapter({
          clientID: Resource.DISCORD_CLIENT_ID.value,
          clientSecret: Resource.DISCORD_CLIENT_SECRET.value,
          scopes: ["email", "identify"],
        }),
      },
      allow: async (input) => {
        const url = new URL(input.redirectURI);
        const hostname = url.hostname;
        if (hostname.endsWith("nestri.io")) return true;
        if (hostname === "localhost") return true;
        return false;
      },
      success: async (ctx, value, req) => {
        let user;

        if (value.provider === "discord") {
          const access = value.tokenset.access;
          user = await handleDiscord(access);
        }

        if (user) {
          try {
            const matching = await User.fromEmail(user.primary.email);

            //Sign Up
            if (!matching) {
              const userID = await User.create({
                email: user.primary.email,
                name: user.username,
                avatarUrl: user.avatar,
              });

              if (!userID) throw new Error("Error creating user");

              return ctx.subject("user", userID, {
                userID,
                email: user.primary.email,
              });
            } else {
              await User.acknowledgeLogin(matching.id);

              //Sign In
              return await ctx.subject("user", matching.id, {
                userID: matching.id,
                email: user.primary.email,
              });
            }
          } catch (error) {
            console.error("error registering the user", error);
          }
        }

        throw new Error("Something went seriously wrong");
      },
    }).use(logger());
  },
};
