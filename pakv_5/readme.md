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

1. 封装服务端客户端，tcp数据处理（粘包半包）

2. 一开始准备吧内核也换成tokio的api，后来发现tokio issue处指出 tokio异步api不如线程阻塞写的性能，所以采用spawn_blocking来维护 文件写入worker。

   ![](./resource/tokio_fs.png)

   关于spawn_blocking的解释，tokio的异步基于await调度，如果执行阻塞函数则会阻塞tokio的调度，所以要将阻塞程序放在spawn_blocking中。

3. 
