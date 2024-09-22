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
	formData.append("id", String(con.id));
	formData.append("uri", con.uri);
	formData.append("verb", event.verb);
	if (event.data) {
		formData.append("data", JSON.stringify(event.data));
	}

	const res = await fetch("http://localhost:8000/ws.php", {
		method: "POST",
		body: formData,
	});
	const text = await res.text();
	console.log(text);
};

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
		ws.on("message", msg => {
			dispatchEvent(con, { verb: "MESSAGE", data: msg });
			receivedMessage(con, uri, msg);
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