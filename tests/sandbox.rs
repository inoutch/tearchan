struct A {}

struct B<'a> {
    a: &'a A,
}

struct C<'a> {
    a: &'a A,
}

struct D<'a> {
    a: Option<A>,
    b: B<'a>,
    c: C<'a>,
}

#[test]
fn sandbox() {

}
