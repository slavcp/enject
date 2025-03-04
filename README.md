# enject
x64 electron injector that intercepts the WNDPROC to fix a bug in chromium


## the bug
in chromium versions past ~97 holding and dragging the cursor freezes any current websocket connection if running with the flag `--disable-frame-rate-limit`

## build
prerequisites: [Rust + Cargo](https://doc.rust-lang.org/stable/cargo/getting-started/installation.html)

1. git clone https://github.com/slavcp/enject.git
1. cd enject
1. cargo build (or run)

wtf is an enjector
