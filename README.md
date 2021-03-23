Library for interacting with Ethereum
=====================================

This code was distilled out of code in the `comit-network`'s [comit-rs repository](https://github.com/comit-network/comit-rs/]).

It retains the GPLv3 license. I intend on using this inside an Intel SGX enclave
so any 'improvements' will unlikely be of interest to the upstream authors.


TODO
----

Add in the prop test and quickcheck stuff from the original code.

- https://docs.rs/proptest/1.0.0/proptest/
- https://altsysrq.github.io/proptest-book/intro.html
- https://github.com/BurntSushi/quickcheck

Write/import integration tests that run against a geth instance to test the RPC
client.
