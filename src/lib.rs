pub mod handlers {
    pub mod users {
        pub mod updating {
            pub mod password {
                pub mod change;
            }
            pub mod username {
                pub mod change;
            }
        }
        pub mod authenticate;
        pub mod login;
        pub mod new;
        pub mod refresh;
    }
}

pub mod models {
    pub mod appstate;
    pub mod user;
}

pub mod util{
    pub mod hash;
    pub mod validation;
    pub mod jwt {
        pub mod claims;
    }
}



