# rcore-console

[![Build Status](https://travis-ci.org/rcore-os/rcore-console.svg?branch=master)](https://travis-ci.org/rcore-os/rcore-console)

The virtual console embedded in rCore kernel.

## Run example

1. rterm

Read and show stdin:

```
htop | cargo run --example rterm
```

2. pty

Spawn a process and show:

```
cargo run --example pty htop
```

TODO: documents and tests
