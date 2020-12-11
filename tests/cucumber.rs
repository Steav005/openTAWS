use std::convert::Infallible;
use std::str::FromStr;

use async_trait::async_trait;
use rand::random;

use uom::si::{f64::*, length::foot, ratio::ratio, time::second, velocity::foot_per_minute};

use cucumber::{t, Steps};
use opentaws::prelude::*;

struct ScenarioContext {}

#[derive(Debug)]
pub struct MyWorld {
    taws: TAWS,
    template_frame: AircraftState,
    props: ScenarioProperties,
}

#[derive(Clone, Debug, Default)]
struct ScenarioProperties {
    height_min: Option<Length>,
    height_max: Option<Length>,
    height_inside: Option<bool>,
    rate_of_descent_min: Option<Velocity>,
}
impl std::panic::UnwindSafe for MyWorld {}

#[async_trait(?Send)]
impl cucumber::World for MyWorld {
    type Error = Infallible;

    async fn new() -> Result<Self, Infallible> {
        Ok(Self {
            taws: TAWS::new(Default::default()),
            template_frame: Default::default(),
            props: Default::default(),
        })
    }
}

pub fn steps() -> Steps<crate::MyWorld> {
    let mut builder: Steps<crate::MyWorld> = Steps::new();

    builder
        .given("the plane is flying", |world, _step| world)
        .given_regex("^(.+) is armed$", |world, mut matches, _step| {
            matches[1].retain(|c| !c.is_whitespace());
            let alert_system = matches[1].parse().unwrap();
            //if matches[2].starts_with("not") {
            //    assert!(!world.taws.function_is_armed(&functionality));
            //} else {
            assert!(world.taws.is_armed(alert_system));
            //}
            world
        })
        .given_regex(
            "^(.+) is (.*)inhibited$",
            |mut world, mut matches, _step| {
                matches[1].retain(|c| !c.is_whitespace());
                let alert_system = matches[1].parse().unwrap();
                if matches[2].starts_with("not") {
                    world.taws.uninhibit(alert_system);
                } else {
                    world.taws.inhibit(alert_system);
                }
                world
            },
        )
        .given_regex(
            r"^steep approach is (.*)selected$",
            |mut world, matches, _step| {
                if matches[1].starts_with("not") {
                    world.template_frame.steep_approach = false;
                } else {
                    world.template_frame.steep_approach = true;
                }
                world
            },
        )
        .then_regex(r"^(.+) shall be armed$", |world, mut matches, _step| {
            matches[1].retain(|c| !c.is_whitespace());
            let alert_system = matches[1].parse().unwrap();
            assert!(world.taws.is_armed(alert_system));
            world
        })
        .when_regex(
            r"^the rate of descent is at least (\d+) feet per minute$",
            |mut world, matches, _step| {
                world.props.rate_of_descent_min = Some(Velocity::new::<foot_per_minute>(
                    matches[1].parse().unwrap(),
                ));
                world
            },
        )
        .when_regex(
            r"^the height above terrain is (.*)between (\d+) and (\d+) feet$",
            |mut world, matches, _step| {
                world.props.height_inside = Some(!matches[1].starts_with("not"));
                world.props.height_min = Some(Length::new::<foot>(matches[2].parse().unwrap()));
                world.props.height_max = Some(Length::new::<foot>(matches[3].parse().unwrap()));
                world
            },
        )
        .then_regex(
            "^a Mode 1 (.*) alert is not emitted at all$",
            |mut world, matches, _step| {
                let alert: AlertLevel = matches[1].parse().unwrap();

                let mut frame = world.template_frame.clone();

                let min = world
                    .props
                    .height_min
                    .unwrap_or(Length::new::<foot>(random()));
                let max = world
                    .props
                    .height_max
                    .unwrap_or(Length::new::<foot>(random()));
                let inside = world.props.height_inside.unwrap_or(random());
                if inside {
                    frame.altitude_ground = min;
                    assert_eq!(world.taws.process(&frame).alerts_count(alert), 0);

                    frame.altitude_ground = max;
                    assert_eq!(world.taws.process(&frame).alerts_count(alert), 0);

                    frame.altitude_ground = (max + min) / Ratio::new::<ratio>(2.0);
                    assert_eq!(world.taws.process(&frame).alerts_count(alert), 0);
                } else {
                    frame.altitude_ground = min - Length::new::<foot>(1.0);
                    assert_eq!(world.taws.process(&frame).alerts_count(alert), 0);

                    frame.altitude_ground = max + Length::new::<foot>(1.0);
                    assert_eq!(world.taws.process(&frame).alerts_count(alert), 0);
                }

                assert_eq!(
                    world.taws.process(&frame).mode_alert_level(Alert::Mode1),
                    None
                );

                /*
                use quickcheck::QuickCheck;
                let mut qc = QuickCheck::new();

                fn tests(mut world: MyWorld )->bool {
                let alert_state = world.taws.push(&world.template_frame);

                alert_state.alerts.is_empty() &&
                alert_state.nuisance_alerts.is_empty()
                };

                qc.quickcheck(tests as fn(_)->_);


                let new_frame = world.template_frame.clone();

                let alert_state = world.taws.push(&new_frame);

                assert!(alert_state.alerts.is_empty());
                assert!(alert_state.nuisance_alerts.is_empty());
                */
                world
            },
        )
        .then_regex(
            r"^a Mode 1 (.*) alert is emitted within (\d+) seconds$",
            |mut world, matches, _step| {
                let alert: AlertLevel = matches[1].parse().unwrap();
                let _max_latency = Time::new::<second>(matches[2].parse().unwrap());

                let mut frame = world.template_frame.clone();
                frame.timestamp += Time::new::<second>(0.1);
                let min = world.props.height_min.unwrap();
                let max = world.props.height_max.unwrap();
                let inside = world.props.height_inside.unwrap();
                if inside {
                    frame.altitude_ground = min;
                    assert!(world.taws.process(&frame).alerts_count(alert) > 0);

                    frame.altitude_ground = max;
                    assert!(world.taws.process(&frame).alerts_count(alert) > 0);

                    frame.altitude_ground = (max + min) / Ratio::new::<ratio>(2.0);
                    assert!(world.taws.process(&frame).alerts_count(alert) > 0);
                } else {
                    frame.altitude_ground = min - Length::new::<foot>(1.0);
                    assert!(world.taws.process(&frame).alerts_count(alert) > 0);

                    frame.altitude_ground = max + Length::new::<foot>(1.0);
                    assert!(world.taws.process(&frame).alerts_count(alert) > 0);
                }
                world
            },
        );
    //.given_regex(
    //    r"the rate of rage is at least (.+) feet per minute",
    //    |world, matches, _step| {
    //        panic!("{}", matches[1]);
    //        world
    //    },
    //);

    builder
}

fn main() {
    let runner = cucumber::Cucumber::<MyWorld>::new()
        .features(&["features"])
        .steps(steps());

    futures::executor::block_on(runner.run());
}