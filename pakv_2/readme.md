## pakv Project 2: Log-structured file I/O

**Task**: Create a persistent key/value store that can be accessed from the
command line.

**Goals**:

- Handle and report errors robustly
- x Use serde for serialization
- x Write data to disk as a log using standard file APIs
- x Read the state of the key/value store from disk
- x Map in-memory key-indexes to on-disk values
- x Periodically compact the log to remove stale data

**Topics**: log-structured file I/O, bitcask, the `failure` crate, `Read` /
`Write` traits, the `serde` crate.

## Introduction

In this project you will create a simple on-disk key/value store that can be
modified and queried from the command line. It will use a simplification of the
storage algorithm used by [bitcask], chosen for its combination of simplicity
and effectiveness. You will start by maintaining a _log_ (sometimes called a
["write-ahead log"][wal] or "WAL") on disk of previous write commands that is
evaluated on startup to re-create the state of the database in memory. Then you
will extend that by storing only the keys in memory, along with offsets into the
on-disk log. Finally, you will introduce log compaction so that it does not grow
indefinitely. At the end of this project you will have built a simple, but
well-architected database using Rust file APIs.

#### [record](./rec.md)

[wal]: 