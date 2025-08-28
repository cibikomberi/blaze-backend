// use lazy_static::lazy_static;
//
// lazy_static! {
//     static ref POOL = {
//         let database_url = env::var("REDIS_URL").expect("DATABASE_URL must be set");
//         static ref pool = redis::Client::open("redis://127.0.0.1/").unwrap();
//     }
// }