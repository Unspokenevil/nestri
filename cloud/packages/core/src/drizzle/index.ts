import ws from "ws";
import { Resource } from "sst";
import { memo } from "../utils";
import { Context } from "../context";
import { ExtractTablesWithRelations } from "drizzle-orm";
import { Pool, neonConfig } from "@neondatabase/serverless";
import { PgTransaction, PgTransactionConfig } from "drizzle-orm/pg-core";
import { NeonQueryResultHKT, drizzle } from "drizzle-orm/neon-serverless";

neonConfig.webSocketConstructor = ws;

export namespace Database {
  function addPoolerSuffix(original: string): string {
    const firstDotIndex = original.indexOf(".");
    if (firstDotIndex === -1) return original + "-pooler";
    return (
      original.slice(0, firstDotIndex) +
      "-pooler" +
      original.slice(firstDotIndex)
    );
  }

  const client = memo(() => {
    const dbHost = addPoolerSuffix(Resource.Database.host);
    const pool = new Pool({
      connectionString: `postgres://${Resource.Database.user}:${Resource.Database.password}@${dbHost}/${Resource.Database.name}?sslmode=require`,
    });
    const db = drizzle(pool);
    return db;
  });

  export type Transaction = PgTransaction<
    NeonQueryResultHKT,
    Record<string, never>,
    ExtractTablesWithRelations<Record<string, never>>
  >;

  export type TxOrDb = Transaction | ReturnType<typeof client>;

  const TransactionContext = Context.create<{
    tx: TxOrDb;
    effects: (() => void | Promise<void>)[];
  }>();

  export async function use<T>(callback: (trx: TxOrDb) => Promise<T>) {
    try {
      const { tx } = TransactionContext.use();
      return tx.transaction(callback);
    } catch (err) {
      if (err instanceof Context.NotFound) {
        const effects: (() => void | Promise<void>)[] = [];
        const result = await TransactionContext.provide(
          {
            effects,
            tx: client(),
          },
          () => callback(client()),
        );
        await Promise.all(effects.map((x) => x()));
        return result;
      }
      throw err;
    }
  }

  export async function fn<Input, T>(
    callback: (input: Input, trx: TxOrDb) => Promise<T>,
  ) {
    return (input: Input) => use(async (tx) => callback(input, tx));
  }

  export async function effect(effect: () => any | Promise<any>) {
    try {
      const { effects } = TransactionContext.use();
      effects.push(effect);
    } catch {
      await effect();
    }
  }

  export async function transaction<T>(
    callback: (tx: TxOrDb) => Promise<T>,
    config?: PgTransactionConfig,
  ) {
    try {
      const { tx } = TransactionContext.use();
      return callback(tx);
    } catch (err) {
      if (err instanceof Context.NotFound) {
        const effects: (() => void | Promise<void>)[] = [];
        const result = await client().transaction(async (tx) => {
          return TransactionContext.provide({ tx, effects }, () =>
            callback(tx),
          );
        }, config);
        await Promise.all(effects.map((x) => x()));
        return result;
      }
      throw err;
    }
  }
}
