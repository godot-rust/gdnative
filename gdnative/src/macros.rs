#![macro_use]

macro_rules! impl_basic_trait {
    (
        Drop for $Type:ident as $GdType:ident : $gd_method:ident
    ) => {
        impl Drop for $Type {
            fn drop(&mut self) {
                unsafe {
                    (get_api().$gd_method)(&mut self.0)
                }
            }
        }
    };

    (
        Clone for $Type:ident as $GdType:ident : $gd_method:ident
    ) => {
        impl Clone for $Type {
            fn clone(&self) -> Self {
               unsafe {
                    let mut result = $Type::default();
                    (get_api().$gd_method)(&mut result.0, &self.0);
                    result
                }
            }
        }
    };

    (
        Default for $Type:ident as $GdType:ident : $gd_method:ident
    ) => {
        impl Default for $Type {
            fn default() -> Self {
                $Type(sys::$GdType::default())
            }
        }
    };

    (
        PartialEq for $Type:ident as $GdType:ident : $gd_method:ident
    ) => {
        impl PartialEq for $Type {
            fn eq(&self, other: &Self) -> bool {
                unsafe {
                    (get_api().$gd_method)(&self.0, &other.0)
                }
            }
        }
    };

    (
        Eq for $Type:ident as $GdType:ident : $gd_method:ident
    ) => {
        impl PartialEq for $Type {
            fn eq(&self, other: &Self) -> bool {
                unsafe {
                    (get_api().$gd_method)(&self.0, &other.0)
                }
            }
        }
        impl Eq for $Type {}
    };
}

macro_rules! impl_basic_traits {
    (
        for $Type:ident as $GdType:ident {
            $( $Trait:ident => $gd_method:ident; )*
        }
    ) => (
        $(
            impl_basic_trait!(
                $Trait for $Type as $GdType : $gd_method
            );
        )*
    )
}
