use crate::server_app::{PaKVServerApp, Requester};
use crate::net::msg2app_sender::NetMsg2App;
use crate::pakv::{KernelWorker2Main, KernelToAppMsg, PaKvOpeResult};

impl PaKVServerApp {
    pub async fn consume_kernel(&mut self, msg: KernelWorker2Main) {
        let res = self.kernel.consume_selfmsg(msg).await;
        match res{
            None => {}//其他事务
            Some(res) => {
                let requster = self.consume_requester_of_opeid(res.opeid);
                match requster {
                    Requester::NetClient(cid) => {
                        let c_ = self.clients_cid2sender.get(&cid);
                        if let Some(c) = c_ {
                            match res.res {
                                PaKvOpeResult::SetResult { .. } => {
                                    c.set_rpl(true).await;
                                }
                                PaKvOpeResult::DelResult { succ } => {
                                    c.del_rpl(succ).await;
                                }
                                PaKvOpeResult::GetResult { v } => {
                                    c.get_rpl(v).await;
                                }
                            }
                        }
                    }
                    Requester::ChannelSendBack(send) => {
                        send.send(res.res).unwrap();
                    }
                }
            }
        }
    }
    pub async fn consume_net(&mut self, msg: NetMsg2App) {
        match msg {
            NetMsg2App::ClientIn { cid, s2csender } => {
                println!("client in");
                self.client_in(cid, s2csender);
            }
            NetMsg2App::ClientOut { cid } => {
                println!("client out");
                self.client_out(cid);
            }

            ///先进行文件io，让内核的其他task进行文件写入，然后再让主循环进行内存修改
            NetMsg2App::Set { cid, k, v } => {
                //此处有两种考虑，
                // 一种是await调用结果，这样后面的消息就得在这个之后处理
                // 另一种考虑到io操作有时间，也可以先调用，但是不等待结果，等有了执行结果再返回给客户端
                //  要使用这个模式需要用select处理两个通道，
                //   一个是当前接收来自客户端的消息，
                //   另一个是接收来自内核的消息

                //采取第二种方法需要标记需要异步处理的任务，
                // 等处理结束后，返回结果携带这个标记，这样就可以在上下文中找到对应的客户端id，然后,将消息返回给客户端
                let opeid = self.kernel.set(k, v).await;
                //记录任务号
                self.kernelopeid_2_requester.insert(opeid, Requester::NetClient(cid));
            }
            NetMsg2App::Get { cid, k } => {
                let opeid = self.kernel.get(k).await;
                if let Some(id) = opeid {
                    self.kernelopeid_2_requester.insert(id, Requester::NetClient(cid));
                } else {
                    //hash没有，直接返回
                    self.clients_cid2sender.get(&cid).unwrap()
                        .get_rpl(None).await;
                }
            }
            NetMsg2App::Del { cid, k } => {
                let opeid = self.kernel.del(k).await;
                self.kernelopeid_2_requester.insert(opeid, Requester::NetClient(cid));
                // self.kernelopeid_2_cid.insert(opeid,cid);
            }
            NetMsg2App::DelWithResultSender {
                sender, k
            } => {
                let opeid = self.kernel.del(k).await;
                self.kernelopeid_2_requester.insert(
                    opeid,
                    Requester::ChannelSendBack(sender));
            }
            NetMsg2App::GetWithResultSender {
                sender, k
            } => {
                let opeid = self.kernel.get(k).await;
                if let Some(opeid)=opeid{
                    self.kernelopeid_2_requester.insert(
                        opeid,
                        Requester::ChannelSendBack(sender));
                }else{
                    sender.send(PaKvOpeResult::GetResult { v: None }).unwrap();
                }
            }
            NetMsg2App::SetWithResultSender {
                sender, k, v
            } => {
                let opeid = self.kernel.set(k,v).await;
                self.kernelopeid_2_requester.insert(
                    opeid,Requester::ChannelSendBack(sender));
            }
        }
    }
}