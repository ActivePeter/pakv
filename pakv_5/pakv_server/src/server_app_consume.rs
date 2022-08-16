use crate::server_app::PaKVServerApp;
use pakv_server_lib::pakv::{KernelToAppMsg, KernelWorker2Main};
use crate::net::msg2app_sender::NetMsg2App;

impl PaKVServerApp{
    pub async fn consume_kernel(&mut self,msg:KernelWorker2Main){
        println!("kernel work done with result");
        let res=self.kernel.consume_selfmsg(msg);
        match res{
            KernelToAppMsg::SetResult { opeid} => {
                self.get_server2clientsender_of_opeid(opeid)
                    .unwrap().set_rpl(true).await;
            }
            KernelToAppMsg::DelResult { opeid,succ} => {
                self.get_server2clientsender_of_opeid(opeid)
                    .unwrap().del_rpl(succ).await;
            }
            KernelToAppMsg::GetResult { opeid, v} => {
                self.get_server2clientsender_of_opeid(opeid)
                    .unwrap().get_rpl(v).await;
            }
        }
    }
    pub async fn  consume_net(&mut self,msg:NetMsg2App){
        match msg {
            NetMsg2App::ClientIn { cid,s2csender } => {
                println!("client in");
                self.client_in(cid,s2csender);
            }
            NetMsg2App::ClientOut { cid} => {
                println!("client out");
                self.client_out(cid);
            }

            ///先进行文件io，让内核的其他task进行文件写入，然后再让主循环进行内存修改
            NetMsg2App::Set { cid,k,v} => {
                //此处有两种考虑，
                // 一种是await调用结果，这样后面的消息就得在这个之后处理
                // 另一种考虑到io操作有时间，也可以先调用，但是不等待结果，等有了执行结果再返回给客户端
                //  要使用这个模式需要用select处理两个通道，
                //   一个是当前接收来自客户端的消息，
                //   另一个是接收来自内核的消息

                //采取第二种方法需要标记需要异步处理的任务，
                // 等处理结束后，返回结果携带这个标记，这样就可以在上下文中找到对应的客户端id，然后,将消息返回给客户端
                let opeid= self.kernel.set(k,v).await;
                //记录任务号
                self.kernelopeid_2_cid.insert(opeid,cid);
            }
            NetMsg2App::Get { cid,k } => {
                let opeid= self.kernel.get(k).await;
                if let Some(id)=opeid{
                    self.kernelopeid_2_cid.insert(id,cid);
                }else{
                    //hash没有，直接返回
                    self.clients_cid2sender.get(&cid).unwrap()
                        .get_rpl(None);
                }
            }
            NetMsg2App::Del { cid,k } => {
                let opeid= self.kernel.del(k).await;
                self.kernelopeid_2_cid.insert(opeid,cid);
            }
        }
    }
}