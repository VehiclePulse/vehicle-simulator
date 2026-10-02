use std::f64::consts::PI;

pub fn update_coordinates(
    speed_kmph: f64,
    elapsed_seconds: f64,
    heading_degrees: f64,
    latitude: &mut f64,
    longitude: &mut f64,
) {
    const EARTH_RADIUS_KM: f64 = 6371.0;

    // Distance traveled during this tick.
    let distance_km = speed_kmph * elapsed_seconds / 3600.0;

    // Convert heading from degrees to radians.
    let heading_radians = heading_degrees * PI / 180.0;

    let north_km = distance_km * heading_radians.cos();
    let east_km = distance_km * heading_radians.sin();

    let latitude_radians = *latitude * PI / 180.0;

    let delta_latitude =
        north_km / EARTH_RADIUS_KM * 180.0 / PI;

    let delta_longitude =
        east_km / (EARTH_RADIUS_KM * latitude_radians.cos())
            * 180.0
            / PI;

    *latitude += delta_latitude;
    *longitude += delta_longitude;
}