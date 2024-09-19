import http from "node:http";

import { WebSocketServer, WebSocket } from "ws";
import express from "express";

let server: http.Server | undefined;
const app = express();
const sockets = new Set<WSConnection>();

interface WSConnection {
	uri: string;
	cookies: string;
	ws: WebSocket;
}

const receivedMessage = async (con: WSConnection, uri: string, msg: any) => {
	console.log("Message: ", uri, msg);
	for (const c of sockets.values()) {
		if (c.ws === con.ws) {
			continue;
		}
		c.ws.send(msg.toString());
	}
}

const serverListen = async () => {
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
		const cookies = req.headers.cookie || "";
		const uri = req.url || "/ws";
		const con = { uri, cookies, ws };
		sockets.add(con);
		ws.on("message", msg => {
			receivedMessage(con, uri, msg);
		});
	});

	return new Promise<void>((resolve) => {
		server?.listen(8080, "127.0.0.1", resolve);
	});
};
serverListen();