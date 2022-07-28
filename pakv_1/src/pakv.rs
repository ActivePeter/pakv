use std::collections::HashMap;

struct KVStore{
    map:HashMap<String,String>
}
impl KVStore{
    pub fn create() -> KVStore {
        return KVStore{
            map:HashMap::new()
        }
    }
    pub fn set(&mut self,k:String,v:String){
        self.map.entry(k).and_modify(|mut v1|{
            *v1=v.clone();
        }).or_insert(v);
    }
    pub fn get(&mut self, k:String) -> Option<&String> {
        return self.map.get(&k);
    }
    pub fn del(&mut self, k:String) -> Option<String> {
        self.map.remove(&k)
    }
}


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