#[allow(dead_code)]
pub fn print_type_of<T>(_: &T) {
    println!("{}", std::any::type_name::<T>());
}


// async fn route_handler(token: Option<TokenServer>) -> HttpRepsonse {
//     if token.is_none() {
//         HttpResponse::Found().header("Location", "/login").finish()
//     } else {
//         HttpResponse::Ok().finish()
//     }
// }