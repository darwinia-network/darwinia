// --- external ---
use rand::{
    distributions::{uniform::SampleUniform, Uniform},
    Rng,
};

/// This macro does not guarantee the id is unique!
///
/// # Example:
///
/// ```
/// accounts![ALL; Alice(644), Bob(755), Dave(777)];
///
/// // equals to
///
/// #[allow(non_snake_case)]
/// let ALL = {
///     let mut all = vec![];
///
///     all.push((644, "Alice"));
///     all.push((755, "Bob"));
///     all.push((777, "Dave"));
///
///     all
/// };
///
/// #[allow(non_snake_case)]
/// let Alice = 644;
/// #[allow(non_snake_case)]
/// let Bob = 755;
/// #[allow(non_snake_case)]
/// let Dave = 777;
///
/// // ---
///
/// accounts![_; Alice(644), Bob(755), Dave(777)];
///
/// // equals to
///
/// #[allow(non_snake_case)]
/// let Alice = 644;
/// #[allow(non_snake_case)]
/// let Bob = 755;
/// #[allow(non_snake_case)]
/// let Dave = 777;
/// ```
#[macro_export]
macro_rules! accounts {
    (
        _;
        $(
            $account:ident($id:expr)
        ),+
    ) => {
        $(
            #[allow(non_snake_case)]
            let $account = $id;
        )+
    };
    (
        $all:ident;
        $(
            $account:ident($id:expr)
        ),+
    ) => {
        #[allow(non_snake_case)]
        let $all = {
            let mut all = vec![];

            $(
                all.push(($id, stringify!($account)));
            )+

            all
        };

        $(
            #[allow(non_snake_case)]
            let $account = $id;
        )+
    };
}

pub fn uniform_range<T>(low: T, high: T, len: usize) -> Vec<T>
where
    T: SampleUniform,
{
    let range = Uniform::from(low..high);

    rand::thread_rng().sample_iter(&range).take(len).collect()
}
