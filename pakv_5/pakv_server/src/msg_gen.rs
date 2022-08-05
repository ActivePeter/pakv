use byteorder::{BigEndian, ByteOrder};

pub fn append_headlen(mut vec:Vec<u8>)->Vec<u8>{
    vec.resize(vec.len() + 4, 0);
    for i in vec.len()-1..4{
        vec[i]=vec[i-4];
    }
    BigEndian::write_u32(&mut vec[0..4]);

    vec
}

pub fn genmsg_delrpl(succ:bool) ->Vec<u8>{
    let vec;
    if succ {
        vec= format!("s:del success").into_bytes()
    }else{
        vec =format!("f:k not found").into_bytes()
    }

    append_headlen(vec)
}
pub fn genmsg_setrpl(succ:bool)->Vec<u8>{
    let vec;
    if succ {
        vec= format!("s:set success").into_bytes()
    }else{
        vec =format!("f:set failed").into_bytes()
    }

    append_headlen(vec)
}
pub fn genmsg_getrpl(res:Option<String>)->Vec<u8>{
    let vec;
    if let Some(V)=res {
        vec= format!("s:get value: {}",v).into_bytes()
    }else{
        vec =format!("f:k not found").into_bytes()
    }

    append_headlen(vec)
}
