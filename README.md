# Stammer

> ni zai suo sen me a?

一个 Naive 的输入法实现。人工智能导论 作业1。报告内容在 report.md 中。

## 编译

使用 Rust，依赖 Nightly features:

```bash
cargo +nightly build --release
```

之后在 `target/release` 下得到两个可执行文件 `trainer` 和 `main`，即训练器和输入法。

## 训练

训练需要：
- 字表，包含所有认为合法的字
- 训练数据，格式可以是:
  - 纯文本文件
  - 提供数据中给出的新浪新闻的格式，即 LF 分割的 JSON Object，其中包含一个 html 属性，里面是训练数据

假设按照默认选项：字表位于 `./data/chars.txt`，所有训练数据位于目录 `./provided/data` 下

```bash
./target/release/trainer
# Or use cargo:
# cargo +nightly --release --bin trainer

# Other options: trainer --help
```

之后会生成 `./data/engine.json`，是训练得到的状态文件。

## 执行
直接从终端输入输出：

```bash
./target/release/main
# Or: cargo run --release
```

包含输入输出文件：
```
./target/release/main -o ./data/output.txt ./data/input.txt
# Or: cargo run --release -- -o ./data/output.txt ./data/input.txt

# Other options: trainer --help
```
