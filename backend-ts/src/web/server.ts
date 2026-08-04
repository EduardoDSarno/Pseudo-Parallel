import { readFile } from "node:fs/promises";
import { createServer, type IncomingMessage, type ServerResponse } from "node:http";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { publishSubscribe, publishUnsubscribe } from "../redis/publishSubscription.js";
import { priceSubType } from "../redis/subscription_builders.js";
import { Coin, PriceDirection } from "../redis/subscription_types/common.js";
import type { OutgoingAlert } from "../redis/subscription_types/outgoing.js";
import { addAlert, listAllAlerts, removeAlert, type StoredAlert } from "./alertStore.js";

export const ALERT_SERVER_PORT = 3000;

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const INDEX_HTML_PATH = path.join(__dirname, "public", "index.html");

/* Open Server-Sent Events connections — one per connected browser tab. Pushing
fired alerts here is a one-way, server-initiated notification, which is exactly
what SSE is for; no need for a full WebSocket for this. */
const sseClients: ServerResponse[] = [];

/* Called from index.ts's subscribeAlerts callback whenever the engine fires an
alert — pushes it to every currently open browser tab. */
export function broadcastAlertFired(alert: OutgoingAlert): void {
  const payload = `event: alert-fired\ndata: ${JSON.stringify(alert)}\n\n`;
  for (const client of sseClients) {
    client.write(payload);
  }
}

function isValidCoin(value: unknown): value is Coin {
  return typeof value === "string" && (Object.values(Coin) as string[]).includes(value);
}

function isValidDirection(value: unknown): value is PriceDirection {
  return typeof value === "string" && (Object.values(PriceDirection) as string[]).includes(value);
}

function parseStoredAlert(body: Record<string, unknown>): StoredAlert | null {
  if (
    !isValidCoin(body.coin) ||
    !isValidDirection(body.direction) ||
    typeof body.trigger_price !== "number" ||
    !Number.isFinite(body.trigger_price)
  ) {
    return null;
  }
  return { coin: body.coin, direction: body.direction, trigger_price: body.trigger_price };
}

async function readJsonBody(req: IncomingMessage): Promise<Record<string, unknown>> {
  const chunks: Buffer[] = [];
  for await (const chunk of req) {
    chunks.push(chunk as Buffer);
  }
  const raw = Buffer.concat(chunks).toString("utf-8");
  return raw ? (JSON.parse(raw) as Record<string, unknown>) : {};
}

function sendJson(res: ServerResponse, status: number, body: unknown): void {
  res.writeHead(status, { "Content-Type": "application/json" });
  res.end(JSON.stringify(body));
}

export function startAlertServer(port: number = ALERT_SERVER_PORT): void {
  const server = createServer((req, res) => {
    handleRequest(req, res).catch((err: unknown) => {
      console.error("alert server error:", err);
      sendJson(res, 500, { error: "internal error" });
    });
  });

  server.listen(port, () => {
    console.log(`Alert web interface listening on http://localhost:${port}`);
  });
}

async function handleRequest(req: IncomingMessage, res: ServerResponse): Promise<void> {
  if (req.method === "GET" && req.url === "/") {
    const html = await readFile(INDEX_HTML_PATH, "utf-8");
    res.writeHead(200, { "Content-Type": "text/html" });
    res.end(html);
    return;
  }

  if (req.method === "GET" && req.url === "/api/alerts") {
    sendJson(res, 200, listAllAlerts());
    return;
  }

  if (req.method === "GET" && req.url === "/api/events") {
    res.writeHead(200, {
      "Content-Type": "text/event-stream",
      "Cache-Control": "no-cache",
      Connection: "keep-alive",
    });
    res.write("\n");
    sseClients.push(res);

    req.on("close", () => {
      const index = sseClients.indexOf(res);
      if (index !== -1) sseClients.splice(index, 1);
    });
    return;
  }

  if (req.method === "POST" && req.url === "/api/alerts") {
    const alert = parseStoredAlert(await readJsonBody(req));
    if (!alert) {
      sendJson(res, 400, { error: "invalid alert payload" });
      return;
    }

    await publishSubscribe(priceSubType(alert));
    addAlert(alert);
    sendJson(res, 201, listAllAlerts());
    return;
  }

  if (req.method === "DELETE" && req.url === "/api/alerts") {
    const alert = parseStoredAlert(await readJsonBody(req));
    if (!alert) {
      sendJson(res, 400, { error: "invalid alert payload" });
      return;
    }

    await publishUnsubscribe(priceSubType(alert));
    removeAlert(alert);
    sendJson(res, 200, listAllAlerts());
    return;
  }

  sendJson(res, 404, { error: "not found" });
}
