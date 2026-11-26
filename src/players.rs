use crate::Outbox;

struct Player {
    outbox: Outbox,
    room: String,
    hp: i32,
    atk: i32,
    group: Option<String>,
}



impl Player {
    fn new(outbox: Outbox) -> Self {
        Self { outbox, room: "first".into(), hp: 100}
    }
    fn send(&self, msg: &str) {
        let _ = self.outbox.send(msg.to_string());
    }
    fn take_damage(&mut self, dmg: i32) {
        self.hp = (self.hp - dmg).max(0);
    }
}



