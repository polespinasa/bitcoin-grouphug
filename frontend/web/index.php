<?php

declare(strict_types=1);

use Nyholm\Psr7\Response;
use Psr\Http\Message\ResponseInterface;
use Psr\Http\Message\ServerRequestInterface;
use Psr\Http\Server\RequestHandlerInterface;
use Slim\Factory\AppFactory;
use Slim\Views\Twig;

define('__ROOT__', dirname(__DIR__));

require __ROOT__.'/vendor/autoload.php';

if (!file_exists(__ROOT__.'/settings.ini') && flock($lock = fopen(__ROOT__.'/settings.ini.dist', 'r'), \LOCK_EX | \LOCK_NB)) {
    copy(__ROOT__.'/settings.ini.dist', __ROOT__.'/settings.ini');

    flock($lock, \LOCK_UN);
    fclose($lock);
}

$settings = parse_ini_file(__ROOT__.'/settings.ini', scanner_mode: \INI_SCANNER_TYPED);

$twig = Twig::create(__ROOT__.'/views', ['debug' => $settings['debug'], 'strict_variables' => true]);

$app = AppFactory::create();
$app->addErrorMiddleware($settings['debug'], $settings['debug'], $settings['debug']);

$app->add(function (ServerRequestInterface $request, RequestHandlerInterface $handler) use ($twig, $settings): ResponseInterface {
    if (false === $fh = stream_socket_client($settings['grouphug_server'])) {
        return $twig->render(new Response(), 'index.html.twig', ['role' => 'warning', 'message' => 'Service down, try again later.']);
    }

    $response = $handler->handle(
        $request
            ->withAttribute('grouphug_conn', $fh)
            ->withAttribute('grouphug_chain', stream_get_line($fh, 16, "\n"))
    );

    stream_socket_shutdown($fh, \STREAM_SHUT_RDWR);
    fclose($fh);

    return $response;
});

$app->get('/', function (ServerRequestInterface $request, ResponseInterface $response) use ($twig) {
    return $twig->render($response, 'index.html.twig', ['role' => null, 'message' => null]);
});

$app->post('/', function (ServerRequestInterface $request, ResponseInterface $response) use ($twig) {
    $form = $request->getParsedBody();

    if (!is_array($form) || empty($form['tx']) || strlen($form['tx']) > 1024 || !preg_match('/^([0-9a-fA-F]{2})+$/', $form['tx'])) {
        return $twig->render($response, 'index.html.twig', ['role' => 'danger', 'message' => 'Transaction rejected!']);
    }

    if (false === fwrite($request->getAttribute('grouphug_conn'), "add_tx {$form['tx']}")) {
        return $twig->render($response, 'index.html.twig', ['role' => 'warning', 'message' => 'Service down, try again later.']);
    }

    return $twig->render($response, 'index.html.twig', ['role' => 'success', 'message' => 'Transaction accepted!']);
});

$app->run();
