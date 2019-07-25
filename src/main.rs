use ggez::{self, conf, event, graphics, Context, ContextBuilder, GameResult};
use std::{env, path};
use specs::prelude::*;

mod components;
mod systems;
mod neighborhood;
mod globals;
use components::*;
use globals::*;


struct Game {
    world: World,
    updater: Dispatcher<'static, 'static>,
}

pub type DeltaTime = (f32);

impl Game {
    fn new(_ctx: &mut Context) -> GameResult<Self> {
        use rand::prelude::*;
        use neighborhood::{Neighborhood, get_area};
        
        let mut world = World::new();
        world.insert(Neighborhood::new(
            (SCREEN_W / AREA_SIZE).ceil() as i32 ,
            (SCREEN_H / AREA_SIZE).ceil() as i32
            ));
        world.insert::<DeltaTime>(0.0);
        let mut updater = DispatcherBuilder::new()
            .with(
                systems::VelocitySystem,
                "VelocitySystem",
                &[]
            )
            .with(
                systems::BoidSystem,
                "BoidSystem",
                &["VelocitySystem"]
            )
            .with(
                systems::AccelSystem,
                "AccelSystem",
                &["VelocitySystem"]
            )
            .build();
        updater.setup(&mut world);

        let mut rng = thread_rng();
        for _ in 0..BOID_N {
            world.create_entity()
                .with(
                    Pos(Point2::new(rng.gen_range(0.0, SCREEN_W), rng.gen_range(0.0, SCREEN_H)))
                )
                .with( {
                    let angle = rng.gen::<f32>() * TAU;
                    let mag = rng.gen_range(-AREA_SIZE, AREA_SIZE);
                    Vel(Vector2::new(angle.cos() * mag, angle.sin() * mag))
                } )
                .with(
                    Acc(Vector2::new(0.0, 0.0))
                )
                .build();
        }

        world.exec(|(ent, pos, mut nh): (Entities, ReadStorage<Pos>, WriteExpect<Neighborhood>)| {
            for (ent, pos) in (&ent, &pos).join() {
                let (x,y) = get_area(pos.0, AREA_SIZE, AREA_SIZE);
                nh.insert(x,y, ent.id());
            }
        });

        Ok(
            Self {world, updater}
        )
    }
}

impl event::EventHandler for Game {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        let dt = ggez::timer::delta(ctx);
        let dt = ggez::timer::duration_to_f64(dt) as f32;
        self.world.insert::<DeltaTime>(dt);
        self.updater.dispatch(&self.world);
        Ok(())
    }

    
    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        graphics::clear(ctx, graphics::BLACK);
        {
            systems::DrawSystem::new(ctx)
                .run_now(&self.world);
        }
        graphics::present(ctx)?;
        Ok(())
    }
}

fn main() -> GameResult {
    let resource_dir = if let Ok(manifest_dir) = env::var("CARGO_MANIFEST_DIR") {
        let mut path = path::PathBuf::from(manifest_dir);
        path.push("resources");
        path
    } else {
        path::PathBuf::from("./resources")
    };

    let cb = ContextBuilder::new("boids", "ggez")
        .window_setup(
            conf::WindowSetup::default()
                .title("ggez + specs")
        )
        .window_mode(
            conf::WindowMode::default()
                .dimensions(SCREEN_W, SCREEN_H)
        )
        .add_resource_path(resource_dir);
    let (ctx, event_loop) = &mut cb.build()?;

    let state = &mut Game::new(ctx)?;
    event::run(ctx, event_loop, state)
}
