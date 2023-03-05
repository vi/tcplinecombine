# tcplinecombine

Privitive logging server that accepts incoming line-based messages and stores them to one compressed file.

```
host1$ tcplinecombine 0.0.0.0:1234 output.zstd -i 5
host1: ...

host2$ date | nc 192.168.0.1 1234
^C
host2$ date | nc 192.168.0.1 1234
^C

host1:
Incoming connection from 127.0.0.1:44750
  finished serving 127.0.0.1:44750
Incoming connection from 127.0.0.1:44762
  finished serving 127.0.0.1:44762
^C (after waiting for 5 seconds)
host1$ zstdcat output.zstd
Sun Mar  5 21:05:05 CET 2023
Sun Mar  5 21:05:07 CET 2023
output.zstd : Read error (39) : premature end
```

## Features

* Handling multiple simultaneous connections and interleaving incoming lines (but not bytes).
* zstd compression of output file with periodical flushes

## Installation

Download a pre-built executable from [Github releases](https://github.com/vi/tcplinecombine/releases) or install from source code with `cargo install --path .`  or `cargo install tcplinecombine`.

## CLI options

<details><summary> tcplinecombine --help output</summary>

```
ARGS:
    <listenaddr>

    <outputfile>

OPTIONS:
    -i, --flush-interval <seconds>

    -l, --max-line-length <bytes>

    -h, --help
      Prints help information.
```
</details>

# See also

* [fdlinecombine](https://github.com/vi/fdlinecombine)
