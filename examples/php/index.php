<!DOCTYPE html>
<html lang="en">
<head>
    <title>PHP Chat</title>
    <style>
        label {
            display: block;
        }
    </style>
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

    <script>
        const connectionStatus = document.getElementById("status");
        const inbox = document.getElementById("in");
        const outbox = document.getElementById("out");
        const send = document.getElementById("send");
        const reconnect = document.getElementById("reconnect");
        let socket;

        const connect = () => {
            socket?.close();
            socket = new WebSocket("ws://127.0.0.1:4000/chat?room=Main");

            inbox.value = '';
            connectionStatus.innerText = "Connecting...";
            socket.onerror = () => { connectionStatus.innerText = "Error"; }
            socket.onclose = () => { connectionStatus.innerText = "Closed"; }
            socket.onopen  = () => { connectionStatus.innerText = "Connected"; }

            socket.onmessage = m => {
                inbox.value += `${m.data}\n`;
            }
            send.onclick = () => {
                const data = outbox.value;
                outbox.value = "";
                socket.send(data);
            }
        };
        connect();
        reconnect.onclick = connect;
    </script>
</body>
</html>