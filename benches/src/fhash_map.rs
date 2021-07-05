pub fn insert5(){
    for _ in 0..10 {
        let mut map:FxHashMap<i32,i32>=FxHashMap::with_capacity_and_hasher(1000_0000, Default::default());
        let t1=common::current_milliseconds();
        for key in 0..1000_0000 {
            map.insert(key, key);
        }
        let t2=common::current_milliseconds();
        println!("{}", t2 - t1);
    }
}