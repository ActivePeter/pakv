## pakv Project 1: The Rust toolbox

**Task**: Create an in-memory key/value store that passes simple tests and responds
to command-line arguments.

**Goals**:

- Install the Rust compiler and tools
- Learn the project structure used throughout this course
- Use `cargo init` / `run` / `test` / `clippy` / `fmt`
- Learn how to find and import crates from crates.io
- Define an appropriate data type for a key-value store

**Topics**: testing, the `clap` crate, `CARGO_VERSION` etc., the `clippy` and
  `rustfmt` tools.

**Extensions**: the `structopt` crate.

## Introduction

In this project you will create a simple in-memory key/value store that maps
strings to strings, and that passes some tests and responds to command line
arguments. The focus of this project is on the tooling and setup that goes into
a typical Rust project.

If this sounds basic to you, please do the project anyway as it discusses some
general patterns that will be used throughout the course.



## record

#### 1.引入clap

toml ：clap = "3.2.15"

添加derive feature ：clap = { version="3.2.15", features = ["derive"]  }

使用clap的derive实现get set del命令

```rust
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 获取键值
    Get(Get),
    /// 设置键值
    Set(Set),
    /// 移除键值
    Del(Del),
}

#[derive(Args)]
struct Get {
    #[clap(value_parser)]
    key: String,
}
#[derive(Args)]
struct Set {
    #[clap(value_parser)]
    key: String,
    #[clap(value_parser)]
    value: String,
}
#[derive(Args)]
struct Del {
    #[clap(value_parser)]
    key: String,
}
```

#### 2.编写in memory 数据结构，使用map

#### 3.编写test

```rust
#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_get_none() {
        let mut kvs=KVStore::create();
        // This assert would fire and test will fail.
        // Please note, that private functions can be tested too!
        assert_eq!(kvs.get(("1").to_owned()), None);
        assert_eq!(kvs.get("2".to_owned()), None);
    }

    #[test]
    fn test_add_get() {
        let mut kvs=KVStore::create();
        kvs.set("1".to_owned(),"111".to_owned());
        kvs.set("2".to_owned(),"222".to_owned());
        // This assert would fire and test will fail.
        // Please note, that private functions can be tested too!
        assert_eq!(kvs.get("1".to_owned()).unwrap(), &"111".to_owned());
        assert_eq!(kvs.get("2".to_owned()).unwrap(), &"222".to_owned());
    }

    #[test]
    fn test_del() {
        let mut kvs=KVStore::create();
        kvs.set("1".to_owned(),"111".to_owned());
        kvs.set("2".to_owned(),"222".to_owned());
        kvs.del("1".to_owned());
        kvs.del("2".to_owned());
        // This assert would fire and test will fail.
        // Please note, that private functions can be tested too!
        assert_eq!(kvs.get("1".to_owned()), None);
        assert_eq!(kvs.get("2".to_owned()), None);
    }
}
```

![image-20220728161659620](https://hanbaoaaa.xyz/tuchuang/images/2022/07/28/image-20220728161659620.png)

#### Q:clion derive生成的数据结构貌似代码提示反应不过来

![image-20220728162228370](https://hanbaoaaa.xyz/tuchuang/images/2022/07/28/image-20220728162228370.png)

