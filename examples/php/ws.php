<?php

$id = $_POST['id'] ?? "";
$verb = $_POST['verb'] ?? "";
$uri = $_POST['uri'] ?? "";
$data = $_POST['uri'] ?? "";

switch($verb){
	case "OPEN":
		if (!$id || !$uri) {
			http_response_code(400);
			die("Missing parameters");
		}
		break;
	case "MESSAGE":
		if (!$id || !$uri || !$data) {
			http_response_code(400);
			die("Missing parameters");
		}
		break;
	case "CLOSE":
	case "ERROR":
		if (!$id || !$uri) {
			http_response_code(400);
			die("Missing parameters");
		}
		break;
	default:
		break;
}