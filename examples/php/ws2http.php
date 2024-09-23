<?php
// First we need to specify where to send the WS messages
$ws2http_url = "http://localhost:8080";

// Then we move the post fields into variables,
// this isn't really necessary but makes the code more
// readable in my opinion
$id   = $_POST['id']   ?? "";
$uri  = $_POST['uri']  ?? "";
$data = $_POST['data'] ?? "";

// This is a low level function doing the actual POST request
// to the WS2HTTP server, it probably shouldn't be called directly
// by your code, instead sendMessageToId or sendMessageToUri should be used.
function ws2http(string $uri, array $data) {
	global $ws2http_url;

	$ch = curl_init($ws2http_url . $uri);
	curl_setopt($ch, CURLOPT_POST, 1);
	curl_setopt($ch, CURLOPT_POSTFIELDS, http_build_query($data));
	$response = curl_exec($ch);
	curl_close($ch);
}

// With this function you can send a message to a particular ID,
// irrespective of the URL it has opened the WebSocket on, think
// private messages for example.
function sendMessageToId(string $id, string $data) {
	ws2http("", [
		"verb" => "MESSAGE",
		"id" => $id,
		"data" => $data,
	]);
}

// Probably the most common function, it sends a message to all WebSockets
// opened for a particular URI, one nice addition is that you can specify
// certain IDs you DON'T want to get that message, most likely the sender.
function sendMessageToUri(string $uri, string $data, string $notTo = "") {
	ws2http($uri, [
		"verb" => "MESSAGE",
		"notTo" => $notTo,
		"data" => $data,
	]);
}

// Depending on you usecase you might want to add code for other events as well,
// right now we only care about messages, but if you to show how many users
// are in a roow, or show a alist of names you might have to add more cases here,
// mainly OPEN/CLOSE/ERROR
switch($_POST['verb'] ?? ""){
	case "MESSAGE":
		if (!$id || !$data) {
			http_response_code(400);
			die("Missing parameters");
		}
		sendMessageToUri($uri, $data, $id);
		break;
	default:
		break;
}