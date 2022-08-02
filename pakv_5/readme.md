# pakv Project5: Asynchrony

**Task**: Create a multi-threaded, persistent key/value store server and client with *asynchronous* networking over a custom protocol.

**Goals**:

- Understand the patterns used when writing Rust futures
- Understand error handling with futures
- Learn to debug the type system
- Perform asynchronous networking with the tokio runtime
- Use boxed futures to handle difficult type-system problems
- Use `impl Trait` to create anonymous `Future` types

**Topics**: asynchrony, futures, tokio, `impl Trait`.

## Records

这一块主要要引入tokio，tokio对io操作进行了类似于go的调度优化，最大化发挥io性能。

