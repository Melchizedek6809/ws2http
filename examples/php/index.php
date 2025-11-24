<!DOCTYPE html>
<html lang="en">
<head>
    <title>PHP Chat</title>
    <style>
        label {
            display: block;
        }
    </style>
    <script src="./script.js" type="module" defer></script>
</head>
<body>
    <h1>WS2HTTP - PHP example Chat</h1>
    <div id="status"></div>
    <label>
        <h4>Inbox</h4>
        <textarea id="in" readonly></textarea>
    </label>
    <label>
        <h4>Outbox</h4>
        <textarea id="out"></textarea>
    </label>
    <button id="send">Send</button>
    <button id="reconnect">Reconnect</button>
</body>
</html>