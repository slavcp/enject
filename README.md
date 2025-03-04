# krinject

x64 electron injector that intercepts the WNDPROC to fix a bug in chromium


## the bug
in chromium versions past ~100 holding and dragging the cursor freezes any current websocket connection  if running with the flag `--disable-frame-rate-limit`

## build

1. prerequisites: [cargo](https://doc.rust-lang.org/stable/cargo/getting-started/installation.html)
1. git clone https://github.com/slavcp/enject.git
1. cd enject
1. cargo build (or run)


what is an enjector
