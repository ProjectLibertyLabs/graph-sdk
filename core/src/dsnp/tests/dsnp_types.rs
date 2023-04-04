use crate::dsnp::dsnp_types::DsnpPrid;

#[test]
#[should_panic]
fn prid_creation_with_less_than_8_byte_values_should_fail() {
	DsnpPrid::new(&[1, 2, 3, 4, 5, 6, 7]);
}

#[test]
#[should_panic]
fn prid_creation_with_more_than_8_byte_values_should_fail() {
	DsnpPrid::new(&[1, 2, 3, 4, 5, 6, 7, 8, 9]);
}
