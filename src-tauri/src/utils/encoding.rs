use encoding_rs::GB18030;

pub fn gb18030_to_utf8(bytes: &[u8]) -> String {
    let (cow, _, _) = GB18030.decode(bytes);
    cow.into_owned()
}
