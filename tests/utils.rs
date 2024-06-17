extern crate aeron_client;
use aeron_client::utils::str_to_c;

#[test]

fn test_util() {
    let str = "test";

    str_to_c(str);
}
