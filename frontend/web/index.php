<?php

declare(strict_types=1);

use Psr\Http\Message\ResponseInterface;
use Psr\Http\Message\ServerRequestInterface;
use Slim\Factory\AppFactory;

require __DIR__ . '/../vendor/autoload.php';

$app = AppFactory::create();

$app->get('/', function (ServerRequestInterface $request, ResponseInterface $response) {
    $response->getBody()->write("Hello, ke ase");
    return $response;
});

$app->run();
