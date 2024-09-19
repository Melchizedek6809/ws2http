<?php
$roomname = $_GET['room'] ?? "";

?><!DOCTYPE html>
<html lang="en">
<head>
	<meta charset="utf-8" />
	<title>TinyChat <?= htmlspecialchars($roomname) ?></title>
	<link rel="stylesheet" href="style.css" />
	<script type="text/javascript" src="script.js" defer></script>
</head>
<body>
	<main><?php
if($roomname)  {
	?>
	<h1>TinyChat - <?= htmlspecialchars($roomname) ?></h1>
	<div class="chat-wrap" data-room="<?= htmlspecialchars($roomname) ?>">
		<div class="chat-users"></div>
		<div class="chat-messages"></div>
		<form>
			<input type="text" name="msg" placeholder="Message" autocomplete="off"/>
			<input type="submit" value="Send" />
		</form>
	</div>
	<?php
} else {
	?>
	<h1>TinyChat</h1>
	<p>This is an example chat, built with WS2HTTP and PHP.</p>
	<p>Please enter the name of the room you wish to join:</p>
	<form action="/" method="GET">
		<input type="text" name="room" placeholder="Room" required />
		<input type="submit" value="Join" />
	</form>
	<?php
}
?>
	</main>
</body>
</html>