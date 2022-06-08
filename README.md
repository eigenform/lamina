# lamina

Some tools for generating and playing with x64 microbenchmarks in user-space,
and a kernel module for providing direct access to PMCs. This was written 
specifically for use on my Ryzen 9 3950X: only Zen 2 machines are supported.

## Usage

`scripts/config-cpu [enable|disable]` is used to toggle options that might
affect test results, i.e. SMT and CPU frequency scaling. You're also *required*
to run this when using the kernel module to instrument PMCs because:

- It globally sets `CR4.PCE`, allowing you to issue `RDPMC` in user-space
  without causing an exception
- It disables the NMI watchdog (which consumes the first counter)

You'll also need to build and load the kernel module before using the crate.
I'm using this on Linux 5.17 with no [obvious] issues. For example:

```
$ make
...
$ sudo ./scripts/config-cpu enable
...
$ sudo insmod kernel/lamina.ko
...
$ cargo run --release --bin rdpmc_example
```

