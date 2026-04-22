use actix_web::Scope;

pub mod user;

pub trait ScopedHandler {
    fn get_service() -> Scope;
}
