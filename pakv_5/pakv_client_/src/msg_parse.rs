
//处理tcp收发粘包半包
// pub mod msg_pack_make {
    use byteorder::{BigEndian, ByteOrder};
    // use tokio::macros::support::Future;

    const MSG_PACK_HEAD_SIZE: u8 = 4;

    //描述数据包头
    #[derive(Default, Debug)]
    struct MsgPackHead {
        // pack_id: u8,
        pack_len: u32,
    }

    //进行数据的解包
    #[derive(Default, Debug)]
    pub struct MsgParser {
        pack_head: MsgPackHead,
        //描述包头
        head_buff: [u8; 4],
        head_rec_cnt: u8,
        //描述包体
        body_buff: Vec<u8>,
        body_rec_cnt: u32,

        handled_offset:usize
    }

    impl MsgParser {
        pub fn create() -> Self {
            return Self::default();
        }
        pub fn before_handle(&mut self){
            self.handled_offset=0;
        }
        pub async fn handle_a_buffset
        (&mut self, buffset: &[u8], _byte_cnt: usize)->Option<&[u8]>
            // where F: FnMut(&[u8])
        {
            // let mut handled_offset = 0;

            while self.handled_offset < _byte_cnt {
                let byte_cnt_left = _byte_cnt - self.handled_offset;
                //头本次还是未收全
                if self.head_rec_cnt < MSG_PACK_HEAD_SIZE {
                    if byte_cnt_left + (self.head_rec_cnt as usize) < MSG_PACK_HEAD_SIZE as usize {
                        for i in 0..byte_cnt_left {
                            self.head_buff[(self.head_rec_cnt as usize) + i]
                                = buffset[self.handled_offset + i];
                        }
                        self.head_rec_cnt += byte_cnt_left as u8;
                    }//头本次收全
                    else {
                        let cpylen = MSG_PACK_HEAD_SIZE - self.head_rec_cnt;
                        for i in 0..cpylen {
                            self.head_buff[(self.head_rec_cnt + i) as usize] =
                                buffset[self.handled_offset + i as usize];
                        }
                        self.handled_offset += (MSG_PACK_HEAD_SIZE - self.head_rec_cnt) as usize;
                        self.calc_pack_head();
                        self.head_rec_cnt = MSG_PACK_HEAD_SIZE;
                        //扩大缓冲区
                        if self.pack_head.pack_len > self.body_buff.len() as u32 {
                            self.body_buff.resize(self.pack_head.pack_len as usize, 0);
                        }
                        // continue;
                    }
                }

                // 1.剩余数据小于需要读的字节数量(不够
                if byte_cnt_left <
                    (self.pack_head.pack_len - self.body_rec_cnt) as usize {
                    self.write_data_2_body(&buffset[self.handled_offset..], byte_cnt_left);
                    return None;
                } else {
                    //完成读包
                    let len = self.pack_head.pack_len - self.body_rec_cnt;
                    self.write_data_2_body(&buffset[self.handled_offset..], len as usize);
                    self.handled_offset += len as usize;

                    self.reset();
                    return Some(&self.body_buff.as_slice()[..self.pack_head.pack_len as usize]);
                    //对包进行解析
                    // onepack_cb(self.body_buff.as_slice()[..self.pack_head.pack_len as usize]);
                    // let a = bytes_to_pack(self.pack_head.pack_id as i32,
                    //                       &self.body_buff.as_slice()[..self.pack_head.pack_len as usize]);
                    // match a {
                    //     None => {
                    //         println!("no msg type matched");
                    //     }
                    //     Some(msg_enum) => {
                    //         receive_handler.handle_one_msg(msg_enum).await;
                    //         // callback(receive_handler, msg_enum).await;
                    //     }
                    // }
                }
            }
            None
        }
        fn reset(&mut self) {
            self.head_rec_cnt = 0;
            self.body_rec_cnt = 0;
        }
        fn write_data_2_body(&mut self, buffset: &[u8], byte_cnt_left: usize) {
            for i in 0..byte_cnt_left {
                self.body_buff[self.body_rec_cnt as usize + i] =
                    buffset[i];
            }
        }
        fn calc_pack_head(&mut self) {
            // self.pack_head.pack_id = self.head_buff[0];
            self.pack_head.pack_len = BigEndian::read_u32(&self.head_buff);
        }
    }
// }
