<?php

function ws2http(string $uri, array $data) {
	$ch = curl_init($uri);
	curl_setopt($ch, CURLOPT_POST, 1);
	curl_setopt($ch, CURLOPT_POSTFIELDS, http_build_query($data));
	$response = curl_exec($ch);
	curl_close($ch);
}

$id   = $_POST['id']   ?? "";
$uri  = $_POST['uri']  ?? "";
$data = $_POST['data'] ?? "";

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
		print_r($uri);
		print_r("\n");
		print_r($data);
		print_r("\n\n");
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