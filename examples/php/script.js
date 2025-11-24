const connectionStatus = document.getElementById("status");
const inbox = document.getElementById("in");
const outbox = document.getElementById("out");
const send = document.getElementById("send");
const reconnect = document.getElementById("reconnect");

const connect = () => {
    const socket = new WebSocket("ws://127.0.0.1:4000/chat");

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