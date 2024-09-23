const wsUrl = 'ws://localhost:8080/ws';

const initChat = chatWrap => {
	const form = chatWrap.querySelector('form');
	const users = chatWrap.querySelector('.chat-users');
	const messages = chatWrap.querySelector('.chat-messages');
	const input = form.querySelector('input[name="msg"]');

	const ws = new WebSocket(wsUrl + `?room=${encodeURIComponent(chatWrap.dataset.room)}`);
	ws.addEventListener('message', msg => {
		const msg = JSON.parse(msg.data || '{}');
		if (msg.type === 'message') {
			showMessage(msg.text);
		}
	});

	const showMessage = (msg, self) => {
		const message = document.createElement('div');
		message.classList.add('chat-message');
		if (self) {
			message.classList.add('chat-message-self');
		}
		message.textContent = msg;
		messages.appendChild(message);
	};

	const sendMessage = msg => {
		showMessage(msg, true);
		ws.send(msg);
	};

	form.addEventListener('submit', e => {
		e.preventDefault();
		const msgText = input.value;
		if (!msgText) return;
		input.value = '';
		const msg = {
			type: 'message',
			text: msgText,
		};
		sendMessage(JSON.stringify(msg));
	});
	input.focus();
};

const initChats = () => {
	document.querySelectorAll('.chat-wrap').forEach(initChat);
};

setTimeout(initChats,0);