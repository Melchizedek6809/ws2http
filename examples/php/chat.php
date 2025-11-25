<?php

$vals = [];
$vals["cookie"] = $_COOKIE;
$vals["post"] = $_POST;
$vals["server"] = $_SERVER;

file_put_contents("log.tmp", json_encode($vals), FILE_APPEND);