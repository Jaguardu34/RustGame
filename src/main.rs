use bevy::prelude::*;


#[derive(Component)]
struct Entity{
    id: String,
}

#[derive(Component)]
struct Monster {
    monster_type: MonsterType,
}

#[derive(Resource)]
struct GreetTimer(Timer);


pub struct HelloPlugin;

impl Plugin for HelloPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GreetTimer(Timer::from_seconds(2.0, TimerMode::Repeating)));
        app.add_systems(Startup, (startup_message, add_monsters));
        app.add_systems(Update, list_monster);
    }
}



fn add_monsters(mut commands: Commands) {
    commands.spawn((Entity{id: "John".to_string()}, Monster{monster_type: MonsterType::Zombie}));
    commands.spawn((Entity{id: "Ellen".to_string()}, Monster{monster_type: MonsterType::Enderman}));
    commands.spawn((Entity{id: "Arachnida".to_string()}, Monster{monster_type: MonsterType::Spider}));
    commands.spawn(Entity{id: "Steve".to_string()});
    
    
}

fn list_monster(
    time : Res<Time>,
    mut timer : ResMut<GreetTimer>,
    entity_query : Query<(&Entity, &Monster)>
) {
    if timer.0.tick(time.delta()).just_finished() {
        for entity in entity_query.iter() {
            let monster_name = match entity.1.monster_type {
                MonsterType::Enderman => "Enderman",
                MonsterType::Zombie => "Zombie",
                MonsterType::Spider => "Spider",

            };
            println!("{} is a {}", entity.0.id, monster_name);
            
        }
    }
}

fn startup_message() {
    println!("Hello world this is RustCraft speaking !");
}


enum MonsterType {
    Zombie,
    Spider,
    Enderman
    
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(HelloPlugin)
        .run();
}



