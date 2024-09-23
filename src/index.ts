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
// 
// This is mainly a problem because if the server restarts, we'll end up reusing prior IDs,
// so a client might accidentally send a message to a different client. By having random IDs,
// this becomes highly unlikely.
//
// A flush command might also be useful, so that the client can truncate the websocket table
// or something like that.
let idCounter = 0;
const genSym = () => String(++idCounter);

// Figure out where to send the http request for this particular event and
// then do a POST request to the handler.
const dispatchEvent = async (con: WSConnection, event: WSEvent) => {
	try {
		// First we need to change the format from an object to FormData,
		// so that it can be easily parsed by the handler.
		//
		// We could've also used content-type: application/json, but that
		// doesn't seem to be as widely supported as form-data, which at
		// least for PHP is parsed implicitly.
		const formData = new FormData();
		formData.set("id", String(con.id));
		formData.set("uri", con.uri);
		formData.set("verb", event.verb);
		if (event.data) {
			formData.set("data", event.data);
		}

		// ToDo: we need a way to configure the handler, but I'm hoping that
		// there'l be a way to somehow implicitly figure out where to send the
		// request, with a config option only being necessary if one wants to
		// override the default behavior.
		await fetch("http://localhost:8000/ws2http.php", {
			method: "POST",
			headers: {
				cookie: con.cookies,
			},
			body: formData,
		});
	} catch (error) {
		console.error(error);
	}
};

const serverListen = async () => {
	// Without this we couldn't use req.body
	app.use(express.urlencoded());

	// This handler is reposnsible for actually sending data to
	// the connected clients, it listens to POST requests and then
	// figures out which connection matches the arguments and then
	// sends that message.
	app.use(async (req, res) => {
		const uri = req.originalUrl;
		const body = req.body;
		try {
			if(req.method !== "POST"){
				throw new Error("Please use POST");
			}
			if(!body.verb){
				throw new Error("Missing verb");
			}

			// For now we only support sending messages, but the
			// verb field is already implemented so that in the
			// future we can add new verbs, but this also adds
			// a simple way to customize the behaviour of ws2http,
			// to satisfy some special requirements.
			switch(body.verb) {
			case "MESSAGE":
				// We can either send a message directly to a particular connection,
				// ignoreing all other fields including the URL
				if(body.id){
					const con = sockets.get(body.id);
					if (con) {
						con.ws.send(body.data);
					}
				} else {
					// Or we can send a message to all connections that match the URL.
					// With support for excluding particular connection ID's, because
					// this is a pretty common pattern, think of a chat application,
					// there the sender already knows the message and doesn't have to
					// wait for the server to tell it what it sent.
					const notTo = new Set((body.notTo || "").split(","));
					for(const con of sockets.values()){
						if(notTo.has(con.id)){
							continue;
						}
						if(con.uri === uri){
							// ToDo: we need to ensure that the WS is actually
							// ready, right now it works because it's highly unlikely
							// that the response being generated would be faster than
							// the WS connection being established.
							//
							// We probably need to implement a message queue per connection,
							// and as soon as it becomes ready we'll empty the queue.
							con.ws.send(body.data);
						}
					}
				}
				break;
			default:
				console.log(`Received ${body.verb} for ${body.id}`);
				break;
			}
			res.status(200).end("OK");
		} catch (error) {
			// Shouldn't happen, but might, so we'll inform the handler
			// that something went horribly wrong.
			console.error(error);
			res.status(500).end("Something went wrong");
		}
	});
	server = http.createServer(app);

	// This part is responsible for accepting incoming websocket connections,
	// so it is responsible for listening to the clients, and should probably
	// be proxied through nginx for SSL termination.
	const wss = new WebSocketServer({ server: server, path: "/ws" });
	wss.on("connection", async (ws, req) => {
		// First we generate a unique identifier for this particular connection
		// this is quite important because otherwise the handler wouldn't be able
		// to specify to which connection it would like to send a particular message
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
			// If something unexpected happens we'll treat it as an error in the WS connection
			// and inform the handler about it, this shouldn't happen but if it does we can hopefully
			// make sure that there aren't any stale entries in a DB or something like that.
			dispatchEvent(con, { verb: "ERROR" });
			sockets.delete(id);
			con.ws.close();
			console.error(error);
		}
	});

	return new Promise<void>(resolve => {
		server?.listen(8080, "127.0.0.1", resolve);
	});
};
serverListen();
