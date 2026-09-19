use ts_rs::TS;

use crate::{
    openapi::Operation,
    operation::{add_parameters, parameters_from_schema, ParamLocation},
    OperationInput,
};

#[cfg(feature = "axum")]
impl<T> OperationInput for serde_qs::axum::QsQuery<T>
where
    T: TS + 'static,
{
    fn operation_input(ctx: &mut crate::generate::GenContext, operation: &mut Operation) {
        let params = parameters_from_schema::<T>(ctx, ParamLocation::Query);
        add_parameters(ctx, operation, params);
    }
}
