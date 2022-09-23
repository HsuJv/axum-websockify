# axum-websockify

A simple implement of websockify using axum

## Usage

```
$ ./target/release/axum-websockify  --help
axum_websockify 0.1.0
Jovi Hsu <jv.hsu@outlook.com>
A simple websockify implement using axum

USAGE:
    axum-websockify [FLAGS] [OPTIONS] <src_addr> <dst_addr> --web <web>

FLAGS:
    -d, --debug      Sets log_level to debug
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
        --cert <cert>              SSL certificate file
        --key <key>                SSL key file
    -l, --log_level <log_level>    One of (error, warn[default], info, debug, trace) Note this value will overwrite -d
                                   settings
        --web <web>                Serve files from <web>

ARGS:
    <src_addr>    [source_addr:]source_port
    <dst_addr>    target_addr:target_port
```

* For convenience, the website from noVNC was embeded to generate a simple POC
* Test: 

```bash
$ vncserver
$ tar xf noVnc.tar
$ ./target/release/axum-websockify --web $PWD/noVnc 0.0.0.0:8080 127.0.0.1:5900
```
