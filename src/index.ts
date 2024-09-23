import http from "node:http";

import { WebSocketServer, WebSocket } from "ws";
import express from "express";

let idCounter = 0;
let server: http.Server | undefined;
const app = express();
const sockets = new Set<WSConnection>();

interface WSConnection {
	id: number;
	uri: string;
	cookies: string;
	ws: WebSocket;
}

interface WSEvent {
	verb: "OPEN" | "CLOSE" | "MESSAGE";
	data?: unknown;
}

const dispatchEvent = async (con: WSConnection, event: WSEvent) => {
	const formData = new FormData();
	formData.set("id", String(con.id));
	formData.set("uri", con.uri);
	formData.set("verb", event.verb);
	if (event.data) {
		formData.set("data", event.data);
	}

	const res = await fetch("http://localhost:8000/ws.php", {
		method: "POST",
		headers: {
			"cookie": con.cookies,
		},
		body: formData,
	});
	const text = await res.text();
	console.log(text);
};

const receivedMessage = async (con: WSConnection, uri: string, msg: any) => {
	for (const c of sockets.values()) {
		if (c.ws === con.ws) {
			continue;
		}
		c.ws.send(msg.toString('utf8'));
	}
}

const serverListen = async () => {
	app.use(async (req, res) => {
		try {
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
		const con = { id: ++idCounter, uri, cookies, ws };
		sockets.add(con);
		dispatchEvent(con, { verb: "OPEN" });
		ws.on("message", data => {
			dispatchEvent(con, { verb: "MESSAGE", data });
			receivedMessage(con, uri, data);
		});
		ws.on("close", () => {
			sockets.delete(con);
			dispatchEvent(con, { verb: "CLOSE" });
		});
	});

	return new Promise<void>((resolve) => {
		server?.listen(8080, "127.0.0.1", resolve);
	});
};
serverListen();