use std::collections::HashMap;

use crate::val::{Function, Val};

#[macro_export]
macro_rules! global_maker {
    ($f:ident => {$($g:tt)*}) => {
        use std::collections::HashMap;
        use $crate::val::Val;
        fn $f() -> HashMap<String, Val> {
            let mut globals = HashMap::with_capacity(global_maker!(@count $($g)*));

            $(global_maker!(@def $g));*

        }
    };

    (@def $g:ident: $t:ty = $val:expr) => {
        let $g = $crate::val::Val::$t($val)
    };
    (@def $g:item) => { $g; };

    (@count ) => {0};
    (@count $odd:item $($a:item $b:item)*) => { 1 | (global_maker!(@count $($a)*) << 1) };
    (@count $($a:item $b:item)*) => { (global_maker!(@count $($a)*) << 1) };
}

// global_maker! {globals => {
//     fn clock() -> Val {
//         use std::time::{SystemTime, UNIX_EPOCH};
//         let now = SystemTime::now();
//         let since_epoch = now.duration_since(UNIX_EPOCH).expect("time went backwards");
//         Val::Number(since_epoch.as_secs_f64())
//     }
//     zero: Number = 0.0;
// }}

pub fn globals() -> HashMap<String, Val> {
    let mut g = HashMap::new();
    fn clock(_: Vec<Val>) -> Val {
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now();
        let since_epoch = now.duration_since(UNIX_EPOCH).expect("time went backwards");
        Val::Number(since_epoch.as_secs_f64())
    }
    g.insert("clock".to_string(), Val::Func(Function::Native(0, clock)));
    g
}
