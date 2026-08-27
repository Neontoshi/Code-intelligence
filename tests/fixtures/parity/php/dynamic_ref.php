<?php
$handlers = [];

function register($name, $fn) {
    global $handlers;
    $handlers[$name] = $fn;
}

register("test", function() { echo "Handler"; });
$handlers["test"]();
