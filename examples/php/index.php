<!DOCTYPE html>
<html lang="en">
<head>
    <title>PHP Chat</title>
    <style>
        body {
            font-family: Arial, sans-serif;
            max-width: 800px;
            margin: 20px auto;
            padding: 0 20px;
        }
        label {
            display: block;
            margin: 10px 0;
        }
        input, textarea {
            width: 100%;
            padding: 8px;
            box-sizing: border-box;
            font-family: monospace;
        }
        textarea {
            height: 300px;
            resize: vertical;
        }
        #out {
            height: 60px;
        }
        button {
            padding: 10px 20px;
            margin: 5px;
            cursor: pointer;
        }
        #status {
            padding: 10px;
            margin: 10px 0;
            background: #f0f0f0;
            border-radius: 4px;
            font-weight: bold;
        }
    </style>
</head>
<body>
    <h1>WS2HTTP - PHP Chat Example</h1>

    <label>
        <strong>Username:</strong>
        <input type="text" id="username" placeholder="Enter your name" value="User">
    </label>

    <div id="status">Disconnected</div>

    <label>
        <strong>Chat Messages:</strong>
        <textarea id="in" readonly></textarea>
    </label>

    <label>
        <strong>Type your message:</strong>
        <textarea id="out" placeholder="Type a message and press Send or Enter"></textarea>
    </label>

    <div>
        <button id="send">Send</button>
        <button id="reconnect">Connect</button>
    </div>

    <script>
        const connectionStatus = document.getElementById("status");
        const inbox = document.getElementById("in");
        const outbox = document.getElementById("out");
        const send = document.getElementById("send");
        const reconnect = document.getElementById("reconnect");
        const usernameInput = document.getElementById("username");
        let socket;

        const connect = () => {
            if(socket){
                socket.onclose = undefined;
                socket.close();
            }

            const username = usernameInput.value.trim() || 'Anonymous';
            socket = new WebSocket(`ws://127.0.0.1:4000/chat?name=${encodeURIComponent(username)}`);

            connectionStatus.innerText = "Connecting...";
            connectionStatus.style.background = "#ffffcc";

            socket.onerror = () => {
                connectionStatus.innerText = "Error";
                connectionStatus.style.background = "#ffcccc";
            }
            socket.onclose = () => {
                connectionStatus.innerText = "Disconnected";
                connectionStatus.style.background = "#ffcccc";
            }
            socket.onopen  = () => {
                connectionStatus.innerText = "Connected";
                connectionStatus.style.background = "#ccffcc";
            }

            socket.onmessage = m => {
                inbox.value += `${m.data}\n`;
                inbox.scrollTop = inbox.scrollHeight;
            }
        };

        send.onclick = () => {
            const data = outbox.value.trim();
            if (data && socket && socket.readyState === WebSocket.OPEN) {
                socket.send(data);
                outbox.value = "";
            }
        };

        // Send on Enter key (without Shift)
        outbox.onkeydown = (e) => {
            if (e.key === 'Enter' && !e.shiftKey) {
                e.preventDefault();
                send.click();
            }
        };
        reconnect.onclick = connect;
    </script>
</body>
</html>