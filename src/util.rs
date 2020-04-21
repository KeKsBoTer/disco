

pub fn partial_cartesian<T: Clone>(a: Vec<Vec<T>>, b: Vec<T>) -> Vec<Vec<T>> {
    a.into_iter()
        .flat_map(|xs| {
            b.iter()
                .cloned()
                .map(|y| {
                    let mut vec = xs.clone();
                    vec.push(y);
                    vec
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

/// Computes the Cartesian product of lists[0] * lists[1] * ... * lists[n].
///
/// # Example
///
/// ```
/// let lists: &[&[_]] = &[&[1, 2], &[4, 5], &[6, 7]];
/// let product = cartesian_product(lists);
/// assert_eq!(product, vec![vec![1, 4, 6],
///                          vec![1, 4, 7],
///                          vec![1, 5, 6],
///                          vec![1, 5, 7],
///                          vec![2, 4, 6],
///                          vec![2, 4, 7],
///                          vec![2, 5, 6],
///                          vec![2, 5, 7]]);
/// ```
pub fn cartesian_product<T: Clone>(lists: Vec<Vec<T>>) -> Vec<Vec<T>> {
    match lists.split_first() {
        Some((first, rest)) => {
            let init: Vec<Vec<T>> = first.iter().cloned().map(|n| vec![n]).collect();

            rest.iter()
                .cloned()
                .fold(init, |vec, list| partial_cartesian(vec, list))
        }
        None => vec![],
    }
}