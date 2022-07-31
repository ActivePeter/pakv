### [Project 3: Synchronous client-server networking][p3]

**Task**: Create a single-threaded, persistent key/value store server and client
with synchronous networking over a custom protocol.

**Goals**:

- x Create a client-server application
- x Write a custom protocol with `std` networking APIs
- x Introduce logging to the server
- Implement pluggable backends with traits
- Benchmark the hand-written backend against `sled`

**Topics**: `std::net`, logging, traits, benchmarking.

## Introduction

In this project you will create a simple key/value server and client. They will communicate with a custom networking protocol of your design. You will emit logs using standard logging crates, and handle errors correctly across the network boundary. Once you have a working client-server architecture, then you will abstract the storage engine behind traits, and compare the performance of yours to the [`sled`] engine.

[Records](./rec.md)
