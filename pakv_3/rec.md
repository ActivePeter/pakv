1. 1.单线程短链接tcp

   [Building a Single-Threaded Web Server - The Rust Programming Language (rust-lang.org)](https://doc.rust-lang.org/book/ch20-01-single-threaded.html)

   协议就先简单点好了，直接吧命令发过去，返回也直接返回消息，开头用f: s:标注是否成功

   基本很快就换完了

2. logging

3. 对比sled

4. ~~可插拔（可替换引擎）~~

   这里考虑到后面要引入async，但trait不支持零开销async，所以暂时不考虑用trait

5. 

