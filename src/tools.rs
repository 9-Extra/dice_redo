pub fn vec2str<T>(vec: &Vec<T>) -> String
where
    T: ToString,
{
    debug_assert!(!vec.is_empty());
    let mut str = String::new();
    let mut iter = vec.iter();
    str += &iter.next().unwrap().to_string();
    iter.for_each(|n| {
        str += &" ";
        str += &n.to_string()
    });
    str
}
