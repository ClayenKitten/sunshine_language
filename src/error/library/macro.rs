//! Declarative macroses used to generate error library.

macro_rules! define_error {
    ($(
        $(#[doc = $doc:expr])*
        $severity:ident $name:ident
        $({$($field:ident: $type:ty),*})?
        = $message:literal
        $(=> $into:ty = $into_by:expr)*
        ;
    )*) => ($(
        $(#[doc = $doc])*
        #[derive(Debug)]
        pub struct $name {
            span: crate::error::types::ErrorSpan,
            $($(
                $field: $type,
            )*)?
        }

        impl $name {
            pub fn report(
                provider: &impl crate::error::types::ReportProvider,
                start: crate::input_stream::Location,
                $($($field: $type,)*)?
            ) {
                let error = Self {
                    span: crate::error::types::ErrorSpan {
                        source: provider.source(),
                        start,
                        end: provider.location(),
                    },
                    $($($field,)*)?
                };
                provider.error_reporter().report(error);
            }
        }

        impl crate::error::types::ReportableError for $name {
            fn severity(&self) -> crate::error::types::Severity {
                severity!($severity)
            }

            fn span(&self) -> crate::error::types::ErrorSpan {
                self.span
            }
        }

        impl std::error::Error for $name { }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                $($(let $field = &self.$field;)*)?
                write!(f, $message)
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
        crate::error::types::Severity::Deny
    };
    (warn) => {
        crate::error::types::Severity::Warn
    };
}
