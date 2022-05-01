pub mod simulator;

#[macro_export]
macro_rules! assert_vec_eq {
    ($v1:expr, $v2:expr) => {
        assert!($v1.iter().all(|m| $v2.contains(m)));
        assert!($v2.iter().all(|m| $v1.contains(m)));
        assert_eq!($v1.len(), $v2.len());
    };
}

#[derive(Debug)]
pub struct Message {
    pub player_id: usize,
    pub messages: Vec<String>,
}

pub trait Game {
    fn new() -> Self;

    fn turn(&self) -> Option<Message>;

    fn play(&mut self, msg: String);

    fn winners(&self) -> Option<Vec<bool>>;
}
