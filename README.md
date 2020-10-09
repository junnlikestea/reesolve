# Reesolve (ree)
![release](https://github.com/junnlikestea/reesolve/workflows/release/badge.svg)
[![Build status](https://github.com/junnlikestea/reesolve/workflows/Continuous%20Integration/badge.svg)](https://github.com/junnlikestea/reesolve/actions)

Reesolve tool to do dual stack IPv4 & IPv6 A & AAAA lookups, since these seem to 
be the records I'm most frequently after. Why not write a tool for it?


### Installation
Precompiled binaries for Reesolve are available in the [releases](https://github.com/junnlikestea/reesolve/releases) 
tab. Just pick your platform and extract the archive that contains the binary.

## Building it yourself 
If you want to build it yourself you will need to install Rust, you can get the
[official installation](https://www.rust-lang.org/tools/install) from the Rust website.

To build Reesolve:
```
$ git clone https://github.com/junnlikestea/reesolve
$ cd reesolve
$ cargo build --release
$ ./target/release/ree --version
```

### Usage

**Providing input**

You can pass in a file containing hostnames using the `-i` flag.
```
ree -i hosts.txt

```
or from `stdin`
```
cat hosts.txt | ree
```
or with output from other tools
```
vita -d hackerone.com | ree
```

**Using a custom list of resolvers**

By default Reesolve will use CloudFlare and Google public nameservers, but if you 
would like to change that, use the `-r` flag. These can be Ipv4 or Ipv6 Addresses
but Reesolve assumes they're using port `53`. 
```
ree -i hosts.txt -r resolvers.txt
```

**Changing the timeout**

The default timeout is `5` seconds, if you would like to change that use the `-t`
flag.
```
ree -i hosts.txt -t 15
```

**Changing the output path or filename**

By default Reesolve will write all results as json, to an output file named `records.json`
in the current directory. However, if you would like to change the path and filename
you could do so with the `-o` flag.
```
ree -i hosts.txt -o some/path/filename
```

**The output format**

Reesolve will write the results to an output file by default. If you would like 
more output formats supported feel free to raise an issue.
```
[
  {
    "name": "hackerone.com.",
    "ip": "104.16.99.52",
    "type": "A",
    "ttl": 300,
    "is_wildcard": false
  },
  {
    "name": "hackerone.com.",
    "ip": "2606:4700::6810:6334",
    "type": "AAAA",
    "ttl": 300,
    "is_wildcard": false
  }
]
```
**How can I tell what's going on?**

If you would like some more verbose output for debugging purposes, you can use the `-v` flag. 
There are different levels of verbosity ranging from noisy to informational, most of the
time I just use `info`. This is all printing to stderr, so it won't be captured
in the results.
* `info`: General information like how many results each source returned.
* `debug`: Lots and lots of information about what's going on under the hood.
```
ree -i hosts.txt -r resolvers.txt -v info
```

### Panic messages!
Reesolve uses [trust-dns](https://github.com/bluejekyll/trust-dns) under the hood
and there is an [open issue](https://github.com/bluejekyll/trust-dns/issues/1232) referring 
to this very error. These panics don't actually affect the output of Reesolve, until
the issue has been fixed in trust-dns they will still occur when running the tool.
```
junn@void:~/reesolve/target/release$ time ./ree -i hosts.txt -t 15 -c 100
thread 'tokio-runtime-worker' panicked at 'internal error: entered unreachable code: non NoError responses should have been converted to an error above', /rustc/04488afe34512aa4c33566eb16d8c912a3ae04f9/src/libstd/macros.rs:16:9
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
thread 'tokio-runtime-worker' panicked at 'internal error: entered unreachable code: non NoError responses should have been converted to an error above', /rustc/04488afe34512aa4c33566eb16d8c912a3ae04f9/src/libstd/macros.rs:16:9
thread 'tokio-runtime-worker' panicked at 'internal error: entered unreachable code: non NoError responses should have been converted to an error above', /rustc/04488afe34512aa4c33566eb16d8c912a3ae04f9/src/libstd/macros.rs:16:9
```

### Common error - Too many open files
Reesolve uses tokio under the hood. If you encounter an error 
similar to *"Too many open files"* it means that there isn't enough available file descriptors on 
your system for the amount of tokio tasks being spawned. You can fix this by increasing the hard and soft limits. 
There are lots of different guides available to increase the limits 
[but here is one for linux](https://www.tecmint.com/increase-set-open-file-limits-in-linux/). 


### A note on tuning the concurrency
Reesolve will limit itself to `200` concurrent and parallel tasks, you can change this using
the `-c` flag. 

```
ree -i hosts.txt -c 500
``` 

### Thanks
[0xatul](https://twitter.com/0xatul) For feedback and improvement ideas.


### Disclaimer
Developers have/has no responsibility or authority over any kind of:
* Legal or Law infringement by third parties and users.
* Malicious use capable of causing damage to third parties.
* Illegal or unlawful use of Reesolve.

