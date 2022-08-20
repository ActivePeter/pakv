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

通过将每个状态更改作为命令添加到append only 日志中，从而提供持久性保证，一次成功的操作对应：wal+写入内存。发生突发情况，可以根据wal恢复.由于连续追加，所以写入的io性能比较好。

#### 运行流程

- 先初始化pakv，返回sender用于从用户操作获取输入
  - 在loop中，等待用户指令传入，通过match匹配操作，进行对应的读写
  - set,del先往tarfile(当前目标写入文件)追加记录
    - 写入完成后将k，以及写入文件位置（文件id，xi）记录到hash中，
    - 追加记录后若超过单个文件大小，则设置新的目标文件id，并开启线程进行对旧文件的压缩，主循环上下文添加compacting标志位
    - 通过读取旧文件，将所有操作数据读取到一个hashmap中
    - 开始从刚刚建立的map中读取kv，写入新文件（写入时记录新的文件位置），若单个文件装不下则再创建文件
    - 完成一个文件写入就往主循环发送一个SysKvOpeBatchUpdate,用于更新到新的kv数据
    - 全部文件都写入完成后，等待主循环处理完最后一个SysKvOpeBatchUpdate，（即已经全部更新到新的索引，
    - 此时删除所有旧文件（已经没有索引了），并通知compact结束
    - 主循环清除标志位
  - get从hash获取记录在文件的位置，然后把数据解析出来

- 循环读取用户输入，通过sender发给kv主循环

#### 注意点

##### 目标写入文件id：tarfid

- 需要保证每次变更要写入到meta中，这样下次启动可以恢复
- 压缩文件取名需要排除目标文件id以及旧文件（我这里采取的是取最大值

##### compact模式下set，del

- 此时的数据要记录下来，不然可能会再后续SysKvOpeBatchUpdate更新来自压缩的索引时，覆盖掉当前数据，
- 后续SysKvOpeBatchUpdate，跳过被记录下的k值数据

#### 实现记录

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