# pakv
Start from talentplan rust practice course, continue developing for better performance. [talent-plan/lesson-plan.md ](https://github.com/pingcap/talent-plan/blob/master/courses/rust/docs/lesson-plan.md)

A simple KV DB implemented in rust. The main project is in folder pakv_kernel.

## features

- bitcast mode. (index by mem, store by log) 

  - Advantages: Because using log for each time store, it is fast.
  - Disadvantages: Indexes in memory, so there's a limit of capacity.

- sync log append, no risk of losing data.

- async, user-first compress

  - In bitcast mode, sync compact is impractical (compressing time might be too long), so I used a thread to compact

  - The user has higher priority, compressing thread will check user reach when some data are written, if user is reaching, compressing thread will stop compact.

  - Compress thread checks in a low frequency.

  - Compress threshold is dynamic 

    For example: first time compress threshold is 10, after compressing we get 5, the next compress threshold should be 5*N

    Here we set N=2;


## working on

## todo
- bufferpool to speed up get.
- muti store engine（lsm）

## tests

- continuous set (single thread)

  ![image-20221113130934862](README.assets\image-20221113130934862.png)

- 

[Practical Networked Applications in Rust](https://github.com/pingcap/talent-plan/blob/master/courses/rust/README.md)

[lesson-plan](https://github.com/pingcap/talent-plan/blob/master/courses/rust/docs/lesson-plan.md)

[pakv Project 1: The Rust toolbox](./pakv_1/readme.md)

[pakv Project 2: Log-structured file I/O](./pakv_2/readme.md)

[pakv Project 3: Synchronous client-server networking](./pakv_3/readme.md)

[pakv Project 4: Concurrency and parallelism](./pakv_4/readme.md)

[pakv Project 5: Asynchrony](./pakv_5/readme.md)

