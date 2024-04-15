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

if (!file_exists(__ROOT__ . '/settings.ini') && flock($lock = fopen(__ROOT__ . '/settings.ini.dist', 'r'), LOCK_EX | LOCK_NB)) {
    copy(__ROOT__ . '/settings.ini.dist', __ROOT__ . '/settings.ini');

    flock($lock, LOCK_UN);
    fclose($lock);
}

$settings = parse_ini_file(__ROOT__ . '/settings.ini', scanner_mode: INI_SCANNER_TYPED);

$app = AppFactory::create();
$app->addErrorMiddleware(
    $settings['debug'],
    $settings['debug'],
    $settings['debug']
);

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

$app->post('/tx', function (ServerRequestInterface $request, ResponseInterface $response) use ($twig, $settings) {
    $form = $request->getParsedBody();

    if (!is_array($form) || empty($form['tx']) || strlen($form['tx']) > 1024 || !preg_match('/^([0-9a-fA-F]{2})+$/', $form['tx'])) {
        return new Response(400, ['Content-Type' => 'text/plain'], 'Fuck off, mate');
    }

    $fh = fsockopen($settings['grouphug_hostname'], $settings['grouphug_port']);
    if ($fh === false) {
        return new Response(500, ['Content-Type' => 'text/plain'], 'Cannot connect to GroupHug server');
    }

    fwrite($fh, "add_tx {$form['tx']}");
    fclose($fh);

    return new Response(302, ['Location' => '/']);
});

$app->run();
