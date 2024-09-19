import http from "node:http";

import { WebSocketServer } from "ws";
import express from "express";

let server: http.Server | undefined;
const app = express();
//const webSockets = new Set<Socket>();

const parseCookiesRaw = (req: http.IncomingMessage) => {
	const ret: Record<string, string> = {};
	const cookies = (req.headers.cookie || "").split(";");
	for (const cookie of cookies) {
		const s = cookie.split("=");
		if (s.length === 2) {
			const key = s[0];
			const val = decodeURIComponent(s[1]);
			ret[key] = val;
		}
	}
	return ret;
};

export const serverListen = async () => {
	app.use(async (req, res) => {
		try {
			const webReq = {
				uri: req.originalUrl,
				verb: "GET",
			};
			res.status(200).set({ "Content-Type": "text/html" }).end("Hello");
		} catch (error) {
			console.error(error);
			res.status(500).end("Something went wrong");
		}
	});
	server = http.createServer(app);

	const wss = new WebSocketServer({ server: server, path: "/ws" });
	wss.on("connection", async (ws, req) => {
		const cookies = parseCookiesRaw(req);
		//webSockets.add(new Socket(ws, session));
	});

	return new Promise<void>((resolve) => {
		server?.listen(8080, "127.0.0.1", resolve);
	});
};
