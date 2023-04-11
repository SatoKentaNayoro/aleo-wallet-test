use snarkvm_console_account::{PrivateKey, ViewKey};
use snarkvm_console_program::Network;
use std::str::FromStr;

// pub fn set_panic_hook() {
//     // When the `console_error_panic_hook` feature is enabled, we can call the
//     // `set_panic_hook` function at least once during initialization, and then
//     // we will get better error messages if our code ever panics.
//     //
//     // For more details see
//     // https://github.com/rustwasm/console_error_panic_hook#readme
//     #[cfg(feature = "console_error_panic_hook")]
//     console_error_panic_hook::set_once();
// }

pub(crate) fn parse_account<N: Network>(
    private_key: Option<String>,
    view_key: String,
) -> anyhow::Result<(Option<PrivateKey<N>>, ViewKey<N>)> {
    let mut pk = None;
    if let Some(..) = private_key {
        let result = PrivateKey::<N>::from_str(&private_key.unwrap());
        if let Ok(..) = result {
            pk = Some(result.unwrap())
        }
    }

    let view_key = ViewKey::from_str(&view_key)?;
    Ok((pk, view_key))
}
