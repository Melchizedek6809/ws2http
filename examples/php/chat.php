<?php

$vals = [];
$vals["cookie"] = $_COOKIE;
$vals["post"] = $_POST;
$vals["server"] = $_SERVER;

file_put_contents("log.jsonl", json_encode($vals)."\n", FILE_APPEND);