mod to_comma_string;
pub use to_comma_string::*;

pub fn to_pretty_byte_size<T>(n: T) -> String
where
    T: num_traits::PrimInt + num_traits::ToPrimitive,
{
    let mut size = n.to_f64().unwrap();
    let mut unit = "B";
    if size >= 1024.0 {
        size /= 1024.0;
        unit = "KB";
    }
    if size >= 1024.0 {
        size /= 1024.0;
        unit = "MB";
    }
    if size >= 1024.0 {
        size /= 1024.0;
        unit = "GB";
    }
    format!("{:.2}{}", size, unit)
}
