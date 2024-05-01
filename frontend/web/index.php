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
        return $twig->render(new Response(), 'index.html.twig', ['alert' => ['class' => 'alert-warning', 'message' => 'Service down, try again later.']]);
    }

    $twig->getEnvironment()->addGlobal('chain', stream_get_line($fh, 16, "\n"));

    $response = $handler->handle($request->withAttribute('grouphug_conn', $fh));

    stream_socket_shutdown($fh, \STREAM_SHUT_RDWR);
    fclose($fh);

    return $response;
});

$app->get('/', function (ServerRequestInterface $request, ResponseInterface $response) use ($twig) {
    return $twig->render($response, 'index.html.twig');
});

$app->post('/', function (ServerRequestInterface $request, ResponseInterface $response) use ($twig) {
    return $twig->render($response, 'index.html.twig', ['alert' => processTx($request->getParsedBody(), $request->getAttribute('grouphug_conn'))]);
});

function processTx(mixed $form, $conn): array
{
    if (!is_array($form) || empty($form['tx']) || strlen($form['tx']) > 100 * 1024 || !preg_match('/^([0-9a-fA-F]{2})+$/', $form['tx'])) {
        return ['class' => 'alert-danger', 'message' => 'Invalid transaction received.'];
    }

    $tx = $form['tx'];

    if (false === stream_socket_sendto($conn, "add_tx $tx")) {
        return ['class' => 'alert-warning', 'message' => 'Service down, try again later.'];
    }

    $reply = stream_get_line($conn, 128, "\n");

    if ('Ok' !== $reply) {
        return ['class' => 'alert-warning', 'message' => 'Transaction rejected. '.$reply];
    }

    return ['class' => 'alert-success', 'message' => 'Transaction accepted!'];
}

$app->run();
