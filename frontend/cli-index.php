<?php

declare(strict_types=1);

// "composer serve" routing script. For development only.

$path = $_SERVER['DOCUMENT_ROOT'] . $_SERVER['REQUEST_URI'];

if (is_file($path)) {
    return false;
}

if (is_dir($path) && is_file("$path/index.html")) {
    return false;
}

require_once __DIR__ . '/web/index.php';
