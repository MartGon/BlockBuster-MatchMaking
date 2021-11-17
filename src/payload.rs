
use serde::{Deserialize, Serialize};

pub mod request
{
    use super::*;

    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct Login
    {
        pub username : String,
    }
}

pub mod response
{
    use serde::{Deserialize, Serialize};
    
    #[derive(Debug, Deserialize, Serialize, Clone)]
    pub struct Login
    {
        pub username : String,
    }
}