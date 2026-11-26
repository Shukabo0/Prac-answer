use players::Player;

type Map = HashMap<String, Player>;

Map.insert(name.clone(), Player::new(out_tx.clone()));

if let Some(p) = map.get_mut(&name) {
    p.room = "tavern".into();
}

let room = map[&name].room.clone();
for p in map.values().filter(|p| p.room == room) {
    p.send("EVT ...");
}

[#derive(Debug, Clone)]
struct item { id: String, name: String };

enum NpcKind { Dialogue, QuestGiver, Enemy: { Damage: i32 } }
struct Npc { name: String, hp: i32, kind: NpcKind }