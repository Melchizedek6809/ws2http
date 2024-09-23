<?php
$ws2http_url = "http://localhost:8080";

$id   = $_POST['id']   ?? "";
$uri  = $_POST['uri']  ?? "";
$data = $_POST['data'] ?? "";

function ws2http(string $uri, array $data) {
	global $ws2http_url;

	$ch = curl_init($ws2http_url . $uri);
	curl_setopt($ch, CURLOPT_POST, 1);
	curl_setopt($ch, CURLOPT_POSTFIELDS, http_build_query($data));
	$response = curl_exec($ch);
	curl_close($ch);
}

function sendMessageToId(string $id, string $data) {
	ws2http("", [
		"verb" => "MESSAGE",
		"id" => $id,
		"data" => $data,
	]);
}

function sendMessageToUri(string $uri, string $data, string $notTo = "") {
	ws2http($uri, [
		"verb" => "MESSAGE",
		"notTo" => $notTo,
		"data" => $data,
	]);
}

switch($_POST['verb'] ?? ""){
	case "OPEN":
		if (!$id) {
			http_response_code(400);
			die("Missing parameters");
		}
		break;
	case "MESSAGE":
		if (!$id || !$data) {
			http_response_code(400);
			die("Missing parameters");
		}
		sendMessageToUri($uri, $data, $id);
		break;
	case "CLOSE":
	case "ERROR":
		if (!$id) {
			http_response_code(400);
			die("Missing parameters");
		}
		break;
	default:
		break;
}