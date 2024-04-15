<?php

declare(strict_types=1);

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
    $response->getBody()->write($twig->render('index.html.twig', ['role' => null, 'message' => null]));

    return $response;
});

$app->post('/', function (ServerRequestInterface $request, ResponseInterface $response) use ($twig, $settings) {
    $form = $request->getParsedBody();

    if (!is_array($form) || empty($form['tx']) || strlen($form['tx']) > 1024 || !preg_match('/^([0-9a-fA-F]{2})+$/', $form['tx'])) {
        $response->getBody()->write($twig->render('index.html.twig', ['role' => 'danger', 'message' => 'Transaction rejected!']));

        return $response;
    }

    $fh = fsockopen($settings['grouphug_hostname'], $settings['grouphug_port']);
    if ($fh === false) {
        $response->getBody()->write($twig->render('index.html.twig', ['role' => 'warning', 'message' => 'Service down, try again later.']));

        return $response;
    }

    fwrite($fh, "add_tx {$form['tx']}");
    fclose($fh);

    $response->getBody()->write($twig->render('index.html.twig', ['role' => 'success', 'message' => 'Transaction accepted!']));

    return $response;
});

$app->run();
