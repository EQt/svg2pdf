macro_rules! ddbg {
    ($val:expr $(,)?) => {
        match $val {
            tmp => {
                eprintln!("{}:{}: {} = {:?}",
                          file!(), line!(), stringify!($val), &tmp);
                tmp
            }
        }
    };
    ($($val:expr),+ $(,)?) => {
        ($(ddbg!($val)),+,)
    };
}
