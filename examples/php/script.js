const wsUrl = 'ws://localhost:8080/ws';

const initChat = chatWrap => {
	const form = chatWrap.querySelector('form');
	const users = chatWrap.querySelector('.chat-users');
	const messages = chatWrap.querySelector('.chat-messages');
	const input = form.querySelector('input[name="msg"]');

	const ws = new WebSocket(wsUrl + `?room=${encodeURIComponent(chatWrap.dataset.room)}`);
	ws.addEventListener('message', msg => {
		const o = JSON.parse(msg.data || '{}');
		console.log(o);
		if (o.type === 'message') {
			showMessage(o);
		}
	});

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

const initChats = () => {
	document.querySelectorAll('.chat-wrap').forEach(initChat);
};

setTimeout(initChats,0);