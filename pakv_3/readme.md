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

## 程序运行流程

- client
  - 等待输入
  - 输入一行，创建tcp短链接，发送给服务端
  - 读取返回信息，解析输出到命令行
- server
  - 初始化kv内核
  - 启动服务端，监听新的链接，解析接受数据，转换为消息枚举发给kv内核
  - 等待kv内核结果，返回给客户端

## Record

1. 单线程短链接tcp

   [Building a Single-Threaded Web Server - The Rust Programming Language (rust-lang.org)](https://doc.rust-lang.org/book/ch20-01-single-threaded.html)

   协议就先简单点好了，直接吧命令发过去，返回也直接返回消息，开头用f: s:标注是否成功

   基本很快就换完了

2. logging

   主要作用是将一些错误调试信息更加结构化输出，便于开发时寻找漏洞，同时可以配置log级别来选择log内容

3. benchmark 对比sled

   benchmark是比较重要但我未曾实践过的环节，只有测验性能才能知道自己代码真正的效率

   对比来看，写操作特别慢，因为这里每次写都要重新打开文件![](F:\prj\talent-plan\pakv_talentplan\pakv_3\bench2.png)

   破案了，sled 默认500ms fsync,也就是一次set后不代表真正的操作成功了，只是进入了内存，后来测试sled一次set大约12us

   ![image-20220818221527635](F:\prj\talent-plan\pakv_talentplan\pakv_3\sledfsync.jpg)

4. ~~可插拔（可替换引擎）~~

   这里考虑到后面要引入async，但trait不支持零开销async，所以暂时不考虑用trait
