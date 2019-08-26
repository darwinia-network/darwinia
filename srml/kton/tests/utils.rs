extern crate evo_kton as kton;
extern crate rand;

#[macro_use]
mod support;

#[test]
#[ignore]
fn accounts_macro() {
    accounts![ALL; Alice(644), Bob(755), Dave(777)];

    let cmp = |a, b| {
        assert_eq!(a, b);

        a
    };

    for (from_macro, expected) in ALL.iter().zip(
        [
            (cmp(Alice, 644), "Alice"),
            (cmp(Bob, 755), "Bob"),
            (cmp(Dave, 777), "Dave"),
        ]
        .iter(),
    ) {
        assert_eq!(from_macro, expected);
    }
}
