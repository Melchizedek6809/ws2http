// First, where can we open a WebSocket to?
const wsUrl = 'ws://localhost:8080/ws';

// Pretty self explanatory, we initialize a single chat Element, all the
// arguments are specified via data attributes
const initChat = chatWrap => {
	const form = chatWrap.querySelector('form');
	const messages = chatWrap.querySelector('.chat-messages');
	const input = form.querySelector('input[name="msg"]');

	const ws = new WebSocket(wsUrl + `?room=${encodeURIComponent(chatWrap.dataset.room)}`);
	ws.addEventListener('message', msg => {
		const o = JSON.parse(msg.data || '{}');
		if (o.type === 'message') {
			showMessage(o);
		}
	});

	// Build up the DOM nodes to show the actual text of msg, which
	// is the format used for sendMessage
	const showMessage = (msg, self) => {
		const message = document.createElement('div');
		message.classList.add('chat-message');
		if (self) {
			message.classList.add('chat-message-self');
		}
		message.textContent = msg.text;
		messages.appendChild(message);
	};

	const sendMessage = msg => {
		showMessage(msg, true);
		ws.send(JSON.stringify(msg));
	};

	// Very simple event handler, we just ensure
	// that there's actually text to be sent, then
	// we convert it into the proper format and send it
	// off, hopefully to be relayed by the server.
	form.addEventListener('submit', e => {
		e.preventDefault();
		const msgText = input.value;
		if (!msgText) return;
		input.value = '';
		const msg = {
			type: 'message',
			text: msgText,
		};
		sendMessage(msg);
	});
	input.focus();
};

// I kinda like this pattern, implement a function that initializes
// a single element, and then have another very simple function calling
// doing .querySelectorAll().forEach(initSingle)
const initChats = () => {
	document.querySelectorAll('.chat-wrap').forEach(initChat);
};
// Defer the initialization so we don't block the main thread for too long,
// most likely unnecessary here, but I've become used to this structure.
setTimeout(initChats,0);