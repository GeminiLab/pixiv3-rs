#![allow(unused_imports)]

#[cfg(feature = "log")]
pub use log::{debug, error, info, trace, warn};

#[cfg(not(feature = "log"))]
mod no_op {
    pub use pixiv3_rs_proc::no_op_macro as trace;
    pub use pixiv3_rs_proc::no_op_macro as debug;
    pub use pixiv3_rs_proc::no_op_macro as info;
    pub use pixiv3_rs_proc::no_op_macro as warn;
    pub use pixiv3_rs_proc::no_op_macro as error;
}

#[cfg(not(feature = "log"))]
pub use no_op::*;
