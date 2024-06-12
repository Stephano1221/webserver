/// Returns the start index of the first occurance of the `delimiter` in `bytes`.
/// 
/// If the delimiter is found, [`Option<usize>`] will be returned, otherwise [`Option<None>`].
pub fn find(bytes: &[u8], delimiter: &[u8]) -> Option<usize> {
    let bytes_length = bytes.len();
    let delimiter_length = delimiter.len();

    if bytes_length < delimiter_length {
        return None
    }
    
    let mut index = 0;
    loop {
        if index > bytes_length - delimiter_length {
            break None
        }
        if &bytes[index..(index + delimiter_length)] == delimiter {
            break Some(index)
        }
        index += 1;
    }
}