<?php

// Parse query string to get username
parse_str($_POST['query'] ?? '', $query);
$username = $query['name'] ?? 'Anonymous';

// Get the method (connect, message, close)
$method = $_POST['method'] ?? '';
$text = $_POST['text'] ?? '';

// Function to send messages to the chat alias
function sendToChat($message) {
    $payload = [
        'aliases' => ['chat'],
        'text_messages' => [$message]
    ];

    $ch = curl_init('http://127.0.0.1:4000/ws2http/send');
    curl_setopt($ch, CURLOPT_POST, true);
    curl_setopt($ch, CURLOPT_POSTFIELDS, json_encode($payload));
    curl_setopt($ch, CURLOPT_HTTPHEADER, ['Content-Type: application/json']);
    curl_setopt($ch, CURLOPT_RETURNTRANSFER, true);
    curl_exec($ch);
    curl_close($ch);
}

$response = [];

switch ($method) {
    case 'connect':
        // Register this connection to the 'chat' alias
        $response['aliases'] = ['chat'];
        $response['meta'] = ['username' => $username];

        // Send join notification to all chat users
        $joinMessage = "[" . date('H:i:s') . "] * $username joined the chat";
        sendToChat($joinMessage);
        break;

    case 'message':
        // Add timestamp and username, broadcast to all
        $timestamp = date('H:i:s');
        $formattedMessage = "[$timestamp] $username: $text";
        sendToChat($formattedMessage);
        break;

    case 'close':
        // Send leave notification
        $leaveMessage = "[" . date('H:i:s') . "] * $username left the chat";
        sendToChat($leaveMessage);
        break;
}

header('Content-Type: application/json; charset=utf-8');
echo json_encode($response);