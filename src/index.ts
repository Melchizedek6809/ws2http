import http from "node:http";

import { WebSocketServer, WebSocket } from "ws";
import express from "express";

let server: http.Server | undefined;
const app = express();
const sockets = new Map<string, WSConnection>();

interface WSConnection {
	id: string;
	uri: string;
	cookies: string;
	ws: WebSocket;
}

interface WSEvent {
	verb: "OPEN" | "MESSAGE" | "CLOSE" | "ERROR";
	data?: unknown;
}

// ToDo: a more sophisticated way to generate unique ids
let idCounter = 0;
const genSym = (): string => {
	return String(++idCounter);
}

const dispatchEvent = async (con: WSConnection, event: WSEvent) => {
	try {
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
				cookie: con.cookies,
			},
			body: formData,
		});

		const text = await res.text();
		console.log(text);
	} catch (error) {
		console.error(error);
	}
};

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
		const id = genSym();
		const cookies = req.headers.cookie || "";
		const uri = req.url || "/ws";
		const con = { id, uri, cookies, ws };
		try {
			sockets.set(id, con);

			dispatchEvent(con, { verb: "OPEN" });

			ws.on("message", data => dispatchEvent(con, { verb: "MESSAGE", data }));

			ws.on("close", () => {
				dispatchEvent(con, { verb: "CLOSE" });
				sockets.delete(id);
			});
			ws.on("error", () => {
				dispatchEvent(con, { verb: "ERROR" });
				sockets.delete(id);
			});
		} catch (error) {
			dispatchEvent(con, { verb: "ERROR" });
			sockets.delete(id);
			console.error(error);
		}
	});

	return new Promise<void>((resolve) => {
		server?.listen(8080, "127.0.0.1", resolve);
	});
};
serverListen();
