// TODO: license and attribution
#[macro_export]
macro_rules! define_guid {
    ($name:ident = $d1:expr, $d2:expr, $d3:expr, $($d4:expr),*) => (
        #[allow(non_upper_case_globals)]
        const $name: $crate::types::GUID = $crate::types::GUID {
            data1: $d1,
            data2: $d2,
            data3: $d3,
            data4: [$($d4),*],
        };
    );

    (pub $name:ident = $d1:expr, $d2:expr, $d3:expr, $($d4:expr),*) => (
        #[allow(non_upper_case_globals)]
        pub const $name: $crate::types::GUID = $crate::types::GUID {
            data1: $d1,
            data2: $d2,
            data3: $d3,
            data4: [$($d4),*],
        };
    );
}

#[macro_export]
macro_rules! com_interface {
    (
        $(#[$iface_attr:meta])*
        interface $iface:ident: $base_iface:ty $(,$extra_iface:ty)* {
            iid: $iid:ident,
            vtable: $vtable:ident,
            $(
                $(#[$fn_attr:meta])*
                fn $func:ident($($i:ident: $t:ty),*) -> $rt:ty;
            )*
        }
    ) => (
        #[allow(missing_debug_implementations)]
        #[doc(hidden)]
        #[repr(C)]
        pub struct $vtable {
            pub base: <$base_iface as $crate::iunknown::ComInterface>::Vtable,
            $(pub $func: extern "stdcall" fn(*const $iface, $($t),*) -> $rt),*
        }

        $(#[$iface_attr])*
        #[derive(Debug)]
        #[repr(C)]
        pub struct $iface {
            vtable: *const $vtable
        }

        impl $iface {
            $($(#[$fn_attr])*
            pub unsafe fn $func(&self, $($i: $t),*) -> $rt {
                ((*self.vtable).$func)(self $(,$i)*)
            })*
        }

        impl ::std::ops::Deref for $iface {
            type Target = $base_iface;
            fn deref(&self) -> &$base_iface {
                unsafe {
                    &*(self as *const $iface as *const $base_iface)
                }
            }
        }

        impl ::std::convert::AsRef<$iface> for $iface {
            fn as_ref(&self) -> &$iface {
                self
            }
        }

        impl ::std::convert::AsRef<$base_iface> for $iface {
            fn as_ref(&self) -> &$base_iface {
                unsafe {
                    &*(self as *const $iface as *const $base_iface)
                }
            }
        }

        $(
        impl ::std::convert::AsRef<$extra_iface> for $iface {
            fn as_ref(&self) -> &$extra_iface {
                unsafe {
                    &*(self as *const $iface as *const $extra_iface)
                }
            }
        }
        )*

        unsafe impl $crate::iunknown::ComInterface for $iface {
            #[doc(hidden)]
            type Vtable = $vtable;

            fn iid() -> $crate::types::IID { $iid }
        }
    )
}

macro_rules! offset_of {
    ($t:ty, $f:ident) => {
        unsafe { &(*(0 as *const $t)).$f as *const _ as usize }
    }
}

macro_rules! com_stubs {
    (
        coclass $cls:ty {
            $(
                mod $prefix:ident in $field:ident {
                    $(
                        interface $iface:ty {
                            $(
                                fn $func:ident($($i:ident: $t:ty),*) -> $rt:ty;
                            )*
                        }
                    )*
                }
            )*
        }
    ) => {
        $(
            mod $prefix {
                use super::*;

                $(
                $(
                    pub extern "stdcall" fn $func(this: *const $iface $(, $i: $t)*) -> $rt {
                        let this = (this as usize - offset_of!($cls, $field)) as *const $cls;
                        let this = unsafe { &*this };

                        unsafe { this.$func($($i),*) }
                    }
                )*
                )*
            }
        )*
    }
}
