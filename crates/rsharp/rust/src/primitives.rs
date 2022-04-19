// TODO: It doesn't make sense to refer to tp_client types in here, these should
// just be regular primitives. This stuff is only here as a stopgap.

#[macro_export]
macro_rules! primitives {
    (; types, $macro_name:ident, $($x:tt)+) => {
        $macro_name!(
            $($x)+,
            u8,
            u16,
            u32,
            u64,
            i8,
            i16,
            i32,
            i64,
            bool,
            f32,
            f64,
            // String,
            ObjectHandle,
            ContractDataHandle,
        );
    };
    (; types, $macro_name:ident) => {
        $macro_name!(
            u8,
            u16,
            u32,
            u64,
            i8,
            i16,
            i32,
            i64,
            bool,
            f32,
            f64,
            // String,
            ObjectHandle,
            ContractDataHandle,
        );
    };
}