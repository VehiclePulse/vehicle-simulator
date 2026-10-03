mod model;
mod simulator;
mod service_manager;

use std::{
    thread,
    time::{Duration, Instant},
};

use chrono::Utc;
use rand::RngExt;
use uuid::Uuid;

use crate::model::{
    alarm::AlarmGenerator,
    event::Event,
    telemetry::{TelemetryState, TirePressure},
    variable,
};

use crate::simulator::{
    coordindate,
    fuel,
    speed,
    temperature,
};

use crate::service_manager::service::ServiceManager;

fn main() {
    /*
     * ------------------------------------------------------------
     * Simulator configuration
     * ------------------------------------------------------------
     */

    let tick_rate = Duration::from_secs(1);

    /*
     * ------------------------------------------------------------
     * Initial vehicle telemetry
     * ------------------------------------------------------------
     */

    let mut telemetry_state = TelemetryState {
        vehicle_id: Uuid::new_v4(),

        latitude: 45.1234,
        longitude: 74.1263,

        speed: 0.0,
        heading: 45.6,

        fuel_level: 30.01,

        engine_temperature: 45.6,

        battery_level: 75.5,

        tire_pressure: TirePressure {
            front_left: 42.6,
            front_right: 58.9,
            rear_left: 456.8,
            rear_right: 47.2,
        },

        last_updated_at: Utc::now(),
    };

    /*
     * ------------------------------------------------------------
     * Random number generator
     * ------------------------------------------------------------
     */

    let mut rng = rand::rng();

    /*
     * ------------------------------------------------------------
     * Time tracking
     * ------------------------------------------------------------
     */

    let mut last_tick = Instant::now();

    let startup_duration =
        rng.random_range(10..=30) as f64;

    let mut vehicle_elapsed = 0.0;

    /*
     * ------------------------------------------------------------
     * Managers
     * ------------------------------------------------------------
     */

    let mut alarm_generator = AlarmGenerator::new();

    let mut service_manager = ServiceManager::new();

    println!(
        "Vehicle starting | startup duration: {:.0}s",
        startup_duration
    );

    println!("--------------------------------------------------");

    /*
     * ============================================================
     * MAIN SIMULATION LOOP
     * ============================================================
     */

    loop {
        /*
         * --------------------------------------------------------
         * 1. Calculate elapsed time.
         *
         * We use actual elapsed time instead of assuming that
         * every iteration takes exactly one second.
         * --------------------------------------------------------
         */

        let elapsed_seconds =
            last_tick.elapsed().as_secs_f64();

        last_tick = Instant::now();

        vehicle_elapsed += elapsed_seconds;

        /*
         * --------------------------------------------------------
         * 2. Vehicle movement / servicing
         *
         * If a service is active:
         *
         *   speed > 0
         *       -> gradually stop vehicle
         *
         *   speed == 0
         *       -> perform servicing
         *
         * Otherwise:
         *
         *   normal vehicle driving
         * --------------------------------------------------------
         */

        if service_manager.has_active_service() {

            /*
             * ----------------------------------------------------
             * Service has been triggered.
             *
             * Vehicle must first come to a gradual halt.
             * ----------------------------------------------------
             */

            if telemetry_state.speed > 0.0 {

                let stopped = speed::update_stopping_speed(
                    &mut telemetry_state,
                    elapsed_seconds,
                );

                if stopped {
                    println!(
                        "Vehicle stopped. Starting service."
                    );
                }
            }

            /*
             * ----------------------------------------------------
             * Vehicle has stopped.
             *
             * ServiceManager performs the servicing.
             * ----------------------------------------------------
             */

            else {
                service_manager.update(
                    &mut telemetry_state,
                    elapsed_seconds,
                );
            }
        }

        /*
         * --------------------------------------------------------
         * No active service.
         *
         * Perform normal vehicle movement.
         * --------------------------------------------------------
         */

        else {

            /*
             * Startup phase.
             */

            if vehicle_elapsed < startup_duration {

                speed::update_starting_speed(
                    &mut telemetry_state,
                    &mut rng,
                );
            }

            /*
             * Normal driving phase.
             */

            else {

                speed::update_speed(
                    &mut telemetry_state,
                    &mut rng,
                    elapsed_seconds,
                );
            }
        }

        /*
         * --------------------------------------------------------
         * 3. Update coordinates.
         *
         * Coordinates use the CURRENT speed.
         *
         * Therefore, while the vehicle is gradually stopping,
         * it continues to move until speed reaches zero.
         * --------------------------------------------------------
         */

        coordindate::update_coordinates(
            telemetry_state.speed,
            elapsed_seconds,
            telemetry_state.heading,
            &mut telemetry_state.latitude,
            &mut telemetry_state.longitude,
        );

        /*
         * --------------------------------------------------------
         * 4. Update fuel.
         * --------------------------------------------------------
         */

        fuel::update_fuel(
            &mut telemetry_state.fuel_level,
            telemetry_state.speed,
            elapsed_seconds,
        );

        /*
         * --------------------------------------------------------
         * 5. Update engine temperature.
         * --------------------------------------------------------
         */

        temperature::update_engine_temperature(
            &mut telemetry_state.engine_temperature,
            telemetry_state.speed,
            elapsed_seconds,
        );

        /*
         * --------------------------------------------------------
         * 6. Update telemetry timestamp.
         * --------------------------------------------------------
         */

        telemetry_state.last_updated_at = Utc::now();

        /*
         * --------------------------------------------------------
         * 7. Generate alarms.
         *
         * AlarmGenerator compares the current telemetry against
         * the previous telemetry.
         * --------------------------------------------------------
         */

        let alarms = alarm_generator.generate(
            elapsed_seconds,
            &telemetry_state,
        );

        /*
         * --------------------------------------------------------
         * 8. Print VARIABLE.
         * --------------------------------------------------------
         */

        println!("Fuel level: {:?}", telemetry_state.fuel_level);
        println!(
            "variable: {:#?}",
            variable::Variable::from(&telemetry_state)
        );

        /*
         * --------------------------------------------------------
         * 9. Print ALARMS.
         * --------------------------------------------------------
         */

        println!(
            "alarm: {:#?}",
            alarms
        );
        

        /*
         * --------------------------------------------------------
         * 10. Convert Warning/Critical alarms to EVENTS.
         *
         * Event::from_alarm() determines whether the alarm has
         * a corresponding servicing action.
         *
         * Example:
         *
         * FuelLevel
         *      -> Refuel
         *
         * EngineTemperature
         *      -> Radiate
         *
         * BatteryLevel
         *      -> Recharge
         *
         * TirePressure
         *      -> NormalizePressure
         * --------------------------------------------------------
         */

        for alarm in &alarms {

            let Some(event) = Event::from_alarm(alarm) else {
                continue;
            };

            /*
             * Print the event.
             */

            println!(
                "event: {:#?}",
                event
            );

            /*
             * Give the alarm to ServiceManager.
             *
             * ServiceManager generates a random target value and
             * adds the service to pending_services.
             */

            service_manager.handle_alarm(
                alarm,
                &mut rng,
            );
        }

        /*
         * --------------------------------------------------------
         * 11. Check pending service targets.
         *
         * Only one service can become active at a time.
         *
         * Example:
         *
         * pending_services:
         *
         *   FuelLevel       target = 15.0
         *   BatteryLevel    target = 20.0
         *   TirePressure    target = 21.0
         *
         * When one target is reached, that service is moved to
         * active_service.
         * --------------------------------------------------------
         */

        if !service_manager.has_active_service() {

            service_manager.check_targets(
                &telemetry_state,
            );
        }

        /*
         * --------------------------------------------------------
         * 12. Print service state.
         * --------------------------------------------------------
         */

        println!(
            "service active: {}",
            service_manager.has_active_service()
        );

        println!(
            "servicing: {}",
            service_manager.is_servicing()
        );

        println!("--------------------------------------------------");

        /*
         * --------------------------------------------------------
         * 13. Wait for the next simulator tick.
         * --------------------------------------------------------
         */

        thread::sleep(tick_rate);
    }
}
