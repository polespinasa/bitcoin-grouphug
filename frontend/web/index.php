<?php

declare(strict_types=1);

use Nyholm\Psr7\Response;
use Psr\Http\Message\ResponseInterface;
use Psr\Http\Message\ServerRequestInterface;
use Slim\Factory\AppFactory;
use Twig\Environment;
use Twig\Loader\FilesystemLoader;

define('__ROOT__', dirname(__DIR__));

require __ROOT__ . '/vendor/autoload.php';

$app = AppFactory::create();

$twig = new Environment(
    new FilesystemLoader(__ROOT__.'/views'),
    [
        'debug' => true,
        'strict_variables' => true,
    ]
);

$app->get('/', function (ServerRequestInterface $request, ResponseInterface $response) use ($twig) {
    $response->getBody()->write($twig->render('index.html.twig'));

    return $response;
});

$app->post('/tx', function (ServerRequestInterface $request, ResponseInterface $response) use ($twig) {
    $form = $request->getParsedBody();

    if (!is_array($form) || empty($form['tx']) || strlen($form['tx']) > 1024 || !preg_match('/^([0-9a-fA-F]{2})+$/', $form['tx'])) {
        return new Response(400, ['Content-Type' => 'text/plain'], 'Fuck off, mate');
    }

    $fh = fsockopen('localhost', 8787);
    if ($fh === false) {
        return new Response(500, ['Content-Type' => 'text/plain'], 'Cannot connect to GroupHug server');
    }

    fwrite($fh, "add_tx {$form['tx']}");
    fclose($fh);

    return new Response(302, ['Location' => '/']);
});

$app->run();
