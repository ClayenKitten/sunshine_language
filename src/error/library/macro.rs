//! Declarative macroses used to generate error library.

macro_rules! define_error {
    ($(
        $(#[doc = $doc:expr])*
        $severity:ident $name:ident
        $({$($field:ident: $type:ty),*})?
        = $message:expr
        $(=> $into:ty = $into_by:expr)*
        ;
    )*) => ($(
        $(#[doc = $doc])*
        #[derive(Debug)]
        pub struct $name {
            span: crate::error::ErrorSpan,
            $($(
                $field: $type,
            )*)?
        }

        impl $name {
            pub fn report(
                provider: &impl crate::error::ReportProvider,
                start: crate::input_stream::Location,
                $($($field: $type,)*)?
            ) {
                let error = Self {
                    span: crate::error::ErrorSpan {
                        source: provider.source(),
                        start,
                        end: provider.location(),
                    },
                    $($($field,)*)?
                };
                provider.error_reporter().report(error);
            }
        }

        impl crate::error::ReportableError for $name {
            fn severity(&self) -> crate::error::Severity {
                severity!($severity)
            }

            fn span(&self) -> crate::error::ErrorSpan {
                self.span
            }
        }

        impl std::error::Error for $name { }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                $($(let $field = &self.$field;)*)?
                message!(f $message)
            }
        }

        $(
            impl std::convert::Into<$into> for $name {
                fn into(self) -> $into {
                    $into_by
                }
            }
        )*
    )*)
}

macro_rules! severity {
    (deny) => {
        crate::error::Severity::Deny
    };
    (warn) => {
        crate::error::Severity::Warn
    };
}

macro_rules! message {
    ($fmt:ident $message:literal) => {
        write!($fmt, $message)
    };
    ($fmt:ident $message:expr) => {
        write!($fmt, "{}", { $message })
    };
}
