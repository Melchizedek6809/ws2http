<?php

$vals = [];
$vals["cookie"] = $_COOKIE;
$vals["post"] = $_POST;
$vals["server"] = $_SERVER;

//file_put_contents("log.jsonl", json_encode($vals)."\n", FILE_APPEND);

$data = [
    "aliases" => ["Chat"],
    "meta" => [
        "Name" => "Ben",
    ],
    "text_messages" => [
        "Welcome!!!"
    ]
];
header('Content-Type: application/json; charset=utf-8');
echo json_encode($data);