```t
A simple CLI stopwatch that saves your progress.

Usage: better-rust-stopwatch [COMMAND]

Commands:
  start
  resume
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

```text
better-rust-stopwatch start --help
Usage: better-rust-stopwatch start [OPTIONS]

Options:
  -t, --time <TIME>  Starting duration in human-readable format (e.g., "1h 30m 10s", "10m") Uses rust humantime to parse
  -h, --help         Print help
```

It also logs time. See better-rust-stopwatch resume --help for more info. To resume from last time, just run better-rust-stopwatch resume with no arguments.
