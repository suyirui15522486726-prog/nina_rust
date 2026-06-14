// 本文件作用：启动 Rust CLI，并把顶层错误转换为终端退出码。

// 函数作用：启动当前命令入口。
fn main() {
    if let Err(error) = nina_rust::cli::run() {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}
