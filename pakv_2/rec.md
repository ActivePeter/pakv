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

**定时：**

对log记录进行压缩，只保留，最新的操作，压缩中将数据写入到新的文件，

