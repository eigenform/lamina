# lamina

Some tools for generating/playing with x64 microbenchmarks in user-space,
and a kernel module for direct access to PMCs. This was written specifically
for use on my Ryzen 9 3950X - assume that only Zen 2 machines are supported.

## Notes

`scripts/config-cpu [enable|disable]` is used to toggle options that might
affect test results, i.e. SMT and CPU frequency scaling. You're also *required*
to run this when using the kernel module to instrument PMCs because:

- It globally sets `CR4.PCE`, allowing you to issue `RDPMC` in user-space
  without causing an exception
- It disables the NMI watchdog (which consumes the first counter)

