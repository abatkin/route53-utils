AWS Route 53 Utilities
======================

This was mostly an experiment writing a CLI/utility project in [Rust](https://www.rust-lang.org/). I needed a tool to
create records in [AWS Route 53](https://aws.amazon.com/route53/) and then wait for the record to be fully synchronized
across the AWS infrastructure before returning.

There are currently two subcommands:
* `update-record` - Update a record (UPSERT) and wait for completion
* `wait-for-change` - Wait for a Route 53 change to be complete

Running
-------

    route53-utils --help

Building
--------
This code happily compiles with Stable Rust (tested with 1.39.0).

After checking out, simply run:

    cargo build # optionally with --release

Dependency Notes
----------------

Note that this program is currently configured to use the Rust-native [rustls](https://github.com/ctz/rustls) instead of
OpenSSL (due to OpenSSL bugs/limitations that surface from the way that [Rusoto](https://www.rusoto.org/) interacts with
OpenSSL). This has the added benefit that there is no dependency/linkage with the system-provided OpenSSL.

As of now, Rusoto uses a compiled-in set of certificates to form its trust anchors ([webpki-roots](https://github.com/ctz/webpki-roots)) but should switch to
[rustls-native-certs](https://github.com/ctz/rustls-native-certs) in the future once the library is updated for
[hyper-rustls](https://github.com/ctz/hyper-rustls) 0.18.0.
