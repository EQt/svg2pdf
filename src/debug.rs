macro_rules! ddbg {
    ($val:expr $(,)?) => {
        if false {
            match $val {
                tmp => {
                    eprintln!("{}:{}: {} = {:?}",
                              file!(), line!(), stringify!($val), &tmp);
                    tmp
                }
            }
        } else {
            $val
        }
    };
    ($($val:expr),+ $(,)?) => {
        ($(ddbg!($val)),+,)
    };
}
