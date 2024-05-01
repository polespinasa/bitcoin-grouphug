# Bitcoin GroupHug

[![CI](https://github.com/polespinasa/bitcoin-grouphug/actions/workflows/ci.yml/badge.svg)](https://github.com/polespinasa/bitcoin-grouphug/actions/workflows/ci.yml)

Bitcoin GroupHug is a Bitcoin transactions batching server that does not need coordination between users in order to work.
The transactions are batched in different groups depending on the Bitcoin mining fees they are willing to pay.
The server does not earn any sat for doing the job.

## Usage

### Start the GroupHug server

Run the GroupHug server:

    $ ./grouphug-server

The server will be runing once you see something similar to:

    Server running on 127.0.0.1:8787

This output is the server endpoint. (Can be personalized in the Config.toml file)

A client can connect to the server by using a TCP socket, e.g., Telnet:

    $ telnet 127.0.0.1 8787

The GroupHug server will respond with the network configured on the Config.toml file:

    Trying 127.0.0.1...
    Connected to 127.0.0.1.
    Escape character is '^]'.
    TESTNET


### Send transactions

A transaction can be sent to the server with the message `add_tx` followed by a raw transaction in hexadecimal.

    add_tx 02000000000101a27959db1b8f057c131465964c3bf5c86cbce8bfa662e62999ae509a7688758b0100000000fdffffff01ddd2f50500000000160014887c4f5e76046e8224113a568b1f7f14945e2d230247304402207945e74b3b9b3bb95fe4440764c7e82dd633ed135bddadc8bed24e4f2f94e65e02202b639a02135fb2cd0f9a5908454caa21b24f02462e7149f805dfe0b9612788af8321024ca581679054b55c9819988af8a990fdf44d5f171ec5bc2203dd90ad33a80da500000000

The server will respond with a `Ok` if the transaction was correctly added to a server. If it was not it will return an error explaining why the transaction could not be added.


### Grouphug front end

TODO



## Configuration

The GroupHug provides a number of configurable parameters to modify its behavior. These settings can be modified in the `Config.toml` file.

### Electrum
`endpoint` -> Specifies the electrum server endpoint you want to use.

`certificate_validation` -> set to false if using self-signed certificates, may be necessary if using own electrum server.

### Group
`time` -> Time in seconds that a group can be runing before get closed (not implemented yet).

`max_size` -> Minimum group size for the group to be closed. If when adding a new transaction the number of inputs and outputs is greater than or equal to this parameter the group will be closed.


### Dust & Fee
`limit` -> Minimum value of the outputs not to be considered dust.

`range` -> Range of group commissions. e.g., if 3 is specified as the value, the groups will range from 1-3sat/vb, from 3.1 to 5sat/vb, etc.

### Server
`ip` -> Binding IP.
`port` -> Binding port.

### Network
`network` -> Mainnet, Testnet, Signet. This value is used to comunicate to the clients wich network is the server running.