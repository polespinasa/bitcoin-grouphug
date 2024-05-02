# Bitcoin GroupHug

[![CI](https://github.com/polespinasa/bitcoin-grouphug/actions/workflows/ci.yml/badge.svg)](https://github.com/polespinasa/bitcoin-grouphug/actions/workflows/ci.yml)

Bitcoin GroupHug is a Bitcoin transactions batching server that does not need coordination between users in order to work.
The transactions are batched in different groups depending on the Bitcoin mining fees they are willing to pay.
The server does not earn any sat for doing the job.

## Backend

### Building

Currently, the backend for Bitcoin GroupHug has to be built from source, and requires the [Rust toolchain](https://rustup.rs/) to do so.

```shell
$ cd server/grouphug-server
$ cargo build --release
```

The compiled binary can be found at `server/grouphug-server/target/release/grouphug-server`.
To install it, move it to a directory in your `$PATH` such as `$HOME/bin` or `/usr/local/bin` (system-wide).

### Usage

#### Start the GroupHug server

Run the GroupHug server:

    $ ./grouphug-server

The server will be running once you see something similar to:

    Server running on 127.0.0.1:8787

This output is the server endpoint. (Can be personalized in the Config.toml file)

A client can connect to the server by using a TCP socket, e.g., Telnet:

    $ telnet 127.0.0.1 8787

The GroupHug server will respond with the network configured on the Config.toml file:

    Trying 127.0.0.1...
    Connected to 127.0.0.1.
    Escape character is '^]'.
    TESTNET


#### Send transactions

A transaction can be sent to the server with the message `add_tx` followed by a raw transaction in hexadecimal.

    add_tx 02000000000101a27959db1b8f057c131465964c3bf5c86cbce8bfa662e62999ae509a7688758b0100000000fdffffff01ddd2f50500000000160014887c4f5e76046e8224113a568b1f7f14945e2d230247304402207945e74b3b9b3bb95fe4440764c7e82dd633ed135bddadc8bed24e4f2f94e65e02202b639a02135fb2cd0f9a5908454caa21b24f02462e7149f805dfe0b9612788af8321024ca581679054b55c9819988af8a990fdf44d5f171ec5bc2203dd90ad33a80da500000000

The server will respond with a `Ok` if the transaction was correctly added to a server. If it was not it will return an error explaining why the transaction could not be added.


### Configuration

The GroupHug provides a number of configurable parameters to modify its behavior. These settings can be modified in the `Config.toml` file.

#### Electrum
`endpoint` -> Specifies the Electrum server endpoint you want to use.

`certificate_validation` -> Set to false if using self-signed certificates, will be necessary if your Electrum endpoint has SSL enabled with a self-signed certificate.

#### Group
`time` -> Time in seconds that a group can be running before it's closed (not implemented yet).

`max_size` -> Minimum group size for the group to be closed. If when adding a new transaction the number of inputs and outputs is greater than or equal to this parameter the group will be closed.


#### Dust & Fee
`limit` -> Minimum value of the outputs to not be considered dust.

`range` -> Range of group fees. e.g., if 3 is specified as the value, the groups will range from 1-3 s/vB, from 3.1 to 5 s/vB, etc.

#### Server
`ip` -> Binding IP.

`port` -> Binding port.

#### Network
`network` -> Mainnet, Testnet or Signet. This value is echoed back to each client when it connects so it can know on which network is the server running.


## Frontend

GroupHug includes an optional web frontend for submitting transactions to the backend in a more user-friendly way than the command line.

### Requirements

* PHP 8.1 or newer
* Web server (nginx or Caddy)
* [Composer](https://getcomposer.org/) package manager
* A GroupHug server

### Setup

Edit `settings.ini` as necessary to reach GroupHug.
If GroupHug is already running locally with the default TCP port, then the default values of `settings.ini.dist` should work out of the box.

You can bring up the frontend locally using `composer serve`, without the need of a webserver.
It runs on port 8080.

```shell
$ cd frontend
$ cp settings.ini.dist settings.ini
$ composer install

$ composer serve
[Thu May  2 10:04:08 2024] PHP 8.3.6 Development Server (http://127.0.0.1:8080) started
```

### Production webserver

Sample Caddyfile.

```
grouphug.example.com {
    root * /var/www/grouphug-manual/web
    encode zstd gzip
    php_fastcgi unix//run/php/php-fpm.sock {
                resolve_root_symlink
        }
}
```
