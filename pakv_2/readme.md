## pakv Project 2: Log-structured file I/O

**Task**: Create a persistent key/value store that can be accessed from the
command line.

**Goals**:

- x Handle and report errors robustly
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

#### 知识点学习

bitcask

[Bitcask 存储模型 - 如果的事 - 博客园 (cnblogs.com)](https://www.cnblogs.com/chenny7/p/4572381.html)

wal

通过将每个状态更改作为命令添加到append only 日志中，从而提供持久性保证，而无需将数据结构刷新到磁盘

一次成功的操作对应：写入内存+wal，突发情况，可以根据wal恢复

#### 设计

**执行命令**：

1.append到日志

2.加入内存hashmap

**启动**：

根据日志进行恢复

**定时、启动：**

对log记录进行压缩，只保留，最新的操作，压缩中将数据写入到新的文件，

##### rust线程间通信，

channel response：通过传递一个rxchannel

#### 实现步骤

1. x 先完成了初始化时的指令重做 以及 创建线程以及通道读取来自用户的命令 以及对应的存储操作

   暂时是将k 和v 都存在map中

2. x 用户循环读取指令，并通过通道向存储线程发送命令

3. x 将内存中的kv映射从值改为存储索引

4. x 实现 Periodically compact the log to remove stale data

   1. 大小到阈值后，新建一个文件用于压缩，一个文件用于存储（先获取当前文件列表，选取列表中不存在的数字
   2. 在压缩过程中，依然允许外部访问读取，同时，外部访问写入会写入到新的存储文件中
   3. 压缩过程：遍历以及获取到的文件列表，可能为之前的压缩，也可能为刚满了的文件，越早编辑的文件先访问，将k对应一个位置信息，存入map，这样新的同一个k的操作覆盖旧的，
   4. 在压缩完毕后，更新所有hash中的索引到新文件，
   5. 删除旧文件，

5. x compact模式下，可能处理了一个用户操作后处理compact映射更新

   这样用户操作值可能被compact里的操作覆盖掉，

   **所以compact期间用户操作的k需要记录下来，这样compact就可以跳过这些k**

6. x compact模式下程序停止

   1. 已经压缩至文件，但是原来的文件还没删除 (初始化时按照文件编辑顺序遍历，影响不大

      解决办法：文件留出位置做标记

   2. 未压缩至文件，即等于还没开始压缩时的状态，

   3. 压缩了一半至文件 (初始化时按照文件编辑顺序遍历，影响不大

7. x compact模式下用户操作

   1. 若按照文件编辑时间来确定恢复时顺序，则用户的对k的操作持久化到文件一定要晚于压缩信息持久化到文件

      如果正在持久化压缩信息，且压缩信息钟包含用户操作k，则用户操作的存储应该等持久化结束，并且操作成功后才能返回给用户，此时若程序停止，用户的操作无效

      如果正在压缩未开始持久化，则用户的操作可以执行，之后压缩持久化需要绕开，用户操作过的k，

   2. **标记用户目标文件到一个meta文件中（选取这种方案)**

      持久化完更新hash的时候跳过用户操作过的k

      初始化时最后访问用户目标文件(若存在)这样也保证了顺序，

#### 测试有效性

执行一定量的操作后，触发compact，此时meta中目标写入文件变成新的文件，同时数据都被压缩并存入新文件

#### 改进空间

string 传递过程中 clone深拷贝

文件io操作,不应该每次都重新打开文件

压缩时，压缩用的map中只存文件索引（节约内存，但是会导致写入新文件时，需要去读取，