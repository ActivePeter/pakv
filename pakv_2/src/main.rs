mod pakv;
mod cmd;
mod input;
mod file;
mod test;


fn main() {
    let ope_sender=pakv::start_kernel();
    //阻塞等待用户输入
    input::input_wait(ope_sender);
}