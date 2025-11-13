use std::f32::consts::PI;
use std::fs::File;
use std::io::Write;

const WIDTH: usize = 800;
const HEIGHT: usize = 800;

// Color struct
#[derive(Clone, Copy, Debug)]
struct Color {
    r: u8,
    g: u8,
    b: u8,
}

impl Color {
    fn new(r: u8, g: u8, b: u8) -> Self {
        Color { r, g, b }
    }

    fn from_float(r: f32, g: f32, b: f32) -> Self {
        Color {
            r: (r.clamp(0.0, 1.0) * 255.0) as u8,
            g: (g.clamp(0.0, 1.0) * 255.0) as u8,
            b: (b.clamp(0.0, 1.0) * 255.0) as u8,
        }
    }

    fn mix(&self, other: &Color, t: f32) -> Color {
        let t = t.clamp(0.0, 1.0);
        Color::new(
            ((self.r as f32) * (1.0 - t) + (other.r as f32) * t) as u8,
            ((self.g as f32) * (1.0 - t) + (other.g as f32) * t) as u8,
            ((self.b as f32) * (1.0 - t) + (other.b as f32) * t) as u8,
        )
    }
}

// 3D Vector
#[derive(Clone, Copy, Debug)]
struct Vec3 {
    x: f32,
    y: f32,
    z: f32,
}

impl Vec3 {
    fn new(x: f32, y: f32, z: f32) -> Self {
        Vec3 { x, y, z }
    }

    fn dot(&self, other: &Vec3) -> f32 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    fn cross(&self, other: &Vec3) -> Vec3 {
        Vec3::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }

    fn length(&self) -> f32 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }

    fn normalize(&self) -> Vec3 {
        let len = self.length();
        if len > 0.0 {
            Vec3::new(self.x / len, self.y / len, self.z / len)
        } else {
            Vec3::new(0.0, 0.0, 0.0)
        }
    }

    fn add(&self, other: &Vec3) -> Vec3 {
        Vec3::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }

    fn sub(&self, other: &Vec3) -> Vec3 {
        Vec3::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }

    fn mul(&self, scalar: f32) -> Vec3 {
        Vec3::new(self.x * scalar, self.y * scalar, self.z * scalar)
    }

    fn rotate_y(&self, angle: f32) -> Vec3 {
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        Vec3::new(
            self.x * cos_a + self.z * sin_a,
            self.y,
            -self.x * sin_a + self.z * cos_a,
        )
    }
}

// Fragment struct
struct Fragment {
    position: Vec3,
    normal: Vec3,
    intensity: f32,
    time: f32,
}

// Noise functions
fn noise_3d(p: &Vec3) -> f32 {
    let x = p.x.sin() * 43758.5453;
    let y = p.y.sin() * 22578.1459;
    let z = p.z.sin() * 19134.3872;
    (x + y + z).fract()
}

fn fbm(p: &Vec3, octaves: i32) -> f32 {
    let mut value = 0.0;
    let mut amplitude = 0.5;
    let mut frequency = 1.0;
    let mut max_value = 0.0;

    for _ in 0..octaves {
        let sample_point = Vec3::new(
            p.x * frequency,
            p.y * frequency,
            p.z * frequency,
        );
        value += noise_3d(&sample_point) * amplitude;
        max_value += amplitude;
        amplitude *= 0.5;
        frequency *= 2.0;
    }

    value / max_value
}

fn turbulence(p: &Vec3, octaves: i32) -> f32 {
    let mut value = 0.0;
    let mut amplitude = 1.0;
    let mut frequency = 1.0;

    for _ in 0..octaves {
        let sample_point = Vec3::new(
            p.x * frequency,
            p.y * frequency,
            p.z * frequency,
        );
        value += (noise_3d(&sample_point) * 2.0 - 1.0).abs() * amplitude;
        amplitude *= 0.5;
        frequency *= 2.0;
    }

    value
}

// Shader 1: Sun
fn sun_shader(fragment: &Fragment) -> Color {
    let radial = (fragment.position.x.powi(2) + fragment.position.y.powi(2) + fragment.position.z.powi(2)).sqrt();
    let radial_normalized = (radial * 2.0).clamp(0.0, 1.0);
    
    let core_color = Color::from_float(1.0, 1.0, 0.9);
    let surface_color = Color::from_float(1.0, 0.6, 0.1);
    let edge_color = Color::from_float(1.0, 0.2, 0.0);
    
    let base_color = if radial_normalized < 0.5 {
        core_color.mix(&surface_color, radial_normalized * 2.0)
    } else {
        surface_color.mix(&edge_color, (radial_normalized - 0.5) * 2.0)
    };

    let turb_pos = Vec3::new(
        fragment.position.x * 3.0,
        fragment.position.y * 3.0 + fragment.time * 0.5,
        fragment.position.z * 3.0,
    );
    let plasma = turbulence(&turb_pos, 4);
    
    let flare_pos = Vec3::new(
        fragment.position.x * 8.0 + fragment.time * 0.8,
        fragment.position.y * 8.0,
        fragment.position.z * 8.0,
    );
    let flares = noise_3d(&flare_pos).powf(3.0);
    
    let edge_intensity = 1.0 - fragment.normal.dot(&Vec3::new(0.0, 0.0, 1.0)).abs();
    let corona = edge_intensity.powf(3.0);
    
    let brightness = fragment.intensity * (0.6 + plasma * 0.3 + flares * 0.5 + corona * 0.8);
    
    Color::from_float(
        base_color.r as f32 / 255.0 * brightness * (1.0 + corona * 0.5),
        base_color.g as f32 / 255.0 * brightness * (1.0 + flares * 0.3),
        base_color.b as f32 / 255.0 * brightness * (1.0 + plasma * 0.2),
    )
}

// Shader 2: Rocky Planet
fn rocky_planet_shader(fragment: &Fragment) -> Color {
    let ocean_deep = Color::from_float(0.0, 0.1, 0.3);
    let ocean_shallow = Color::from_float(0.0, 0.3, 0.6);
    
    let continent_pos = Vec3::new(
        fragment.position.x * 2.0,
        fragment.position.y * 2.0,
        fragment.position.z * 2.0,
    );
    let continent_noise = fbm(&continent_pos, 5);
    let is_land = continent_noise > 0.48;
    
    let terrain_pos = Vec3::new(
        fragment.position.x * 10.0,
        fragment.position.y * 10.0,
        fragment.position.z * 10.0,
    );
    let terrain = fbm(&terrain_pos, 4);
    
    let beach = Color::from_float(0.85, 0.8, 0.6);
    let lowland = Color::from_float(0.2, 0.5, 0.1);
    let highland = Color::from_float(0.4, 0.3, 0.2);
    let mountain = Color::from_float(0.6, 0.6, 0.6);
    
    let land_color = if terrain < 0.3 {
        beach.mix(&lowland, terrain * 3.3)
    } else if terrain < 0.6 {
        lowland.mix(&highland, (terrain - 0.3) * 3.3)
    } else {
        highland.mix(&mountain, (terrain - 0.6) * 2.5)
    };
    
    let cloud_pos = Vec3::new(
        fragment.position.x * 5.0 + fragment.time * 0.1,
        fragment.position.y * 5.0,
        fragment.position.z * 5.0,
    );
    let clouds = fbm(&cloud_pos, 3);
    let has_cloud = clouds > 0.6;
    let cloud_density = ((clouds - 0.6) * 2.5).clamp(0.0, 1.0);
    
    let mut final_color = if is_land {
        land_color
    } else {
        let depth = (continent_noise - 0.3) / 0.18;
        ocean_deep.mix(&ocean_shallow, depth.clamp(0.0, 1.0))
    };
    
    if has_cloud {
        let cloud_color = Color::from_float(0.95, 0.95, 1.0);
        final_color = final_color.mix(&cloud_color, cloud_density * 0.7);
    }
    
    let lit = fragment.intensity * (0.4 + 0.6 * fragment.intensity);
    
    Color::from_float(
        final_color.r as f32 / 255.0 * lit,
        final_color.g as f32 / 255.0 * lit,
        final_color.b as f32 / 255.0 * lit,
    )
}

// Shader 3: Gas Giant
fn gas_giant_shader(fragment: &Fragment) -> Color {
    let band_frequency = 8.0;
    let band = (fragment.position.y * band_frequency).sin() * 0.5 + 0.5;
    
    let color1 = Color::from_float(0.8, 0.6, 0.4);
    let color2 = Color::from_float(0.5, 0.3, 0.2);
    let color3 = Color::from_float(0.9, 0.7, 0.5);
    
    let base_band = if band < 0.33 {
        color1.mix(&color2, band * 3.0)
    } else if band < 0.66 {
        color2.mix(&color3, (band - 0.33) * 3.0)
    } else {
        color3.mix(&color1, (band - 0.66) * 3.0)
    };
    
    let flow_pos = Vec3::new(
        fragment.position.x * 6.0 + fragment.time * 0.2,
        fragment.position.y * 12.0,
        fragment.position.z * 6.0,
    );
    let flow = turbulence(&flow_pos, 4);
    
    let spot_center = Vec3::new(0.3, -0.2, 0.8);
    let dist_to_spot = fragment.position.sub(&spot_center).length();
    let spot_size = 0.25;
    let spot_intensity = if dist_to_spot < spot_size {
        ((1.0 - dist_to_spot / spot_size) * PI / 2.0).cos().powf(2.0)
    } else {
        0.0
    };
    let spot_color = Color::from_float(0.7, 0.2, 0.1);
    
    let detail_pos = Vec3::new(
        fragment.position.x * 20.0,
        fragment.position.y * 20.0,
        fragment.position.z * 20.0,
    );
    let detail = noise_3d(&detail_pos) * 0.3;
    
    let mut final_color = base_band;
    
    let flow_influence = flow * 0.2 - 0.1;
    final_color = Color::from_float(
        (final_color.r as f32 / 255.0 + flow_influence).clamp(0.0, 1.0),
        (final_color.g as f32 / 255.0 + flow_influence).clamp(0.0, 1.0),
        (final_color.b as f32 / 255.0 + flow_influence).clamp(0.0, 1.0),
    );
    
    final_color = final_color.mix(&spot_color, spot_intensity * 0.8);
    
    let brightness = fragment.intensity * (0.7 + detail);
    
    Color::from_float(
        final_color.r as f32 / 255.0 * brightness,
        final_color.g as f32 / 255.0 * brightness,
        final_color.b as f32 / 255.0 * brightness,
    )
}

// Shader 4: Ice Giant (Neptune-like)
fn ice_giant_shader(fragment: &Fragment) -> Color {
    // Layer 1: Base blue-cyan gradient
    let base_color1 = Color::from_float(0.2, 0.4, 0.8);
    let base_color2 = Color::from_float(0.1, 0.6, 0.9);
    let base_color3 = Color::from_float(0.3, 0.7, 1.0);
    
    // Layer 2: Atmospheric bands (less pronounced than Jupiter)
    let band_frequency = 12.0;
    let band = (fragment.position.y * band_frequency + fragment.time * 0.3).sin() * 0.5 + 0.5;
    
    let base_color = if band < 0.33 {
        base_color1.mix(&base_color2, band * 3.0)
    } else if band < 0.66 {
        base_color2.mix(&base_color3, (band - 0.33) * 3.0)
    } else {
        base_color3.mix(&base_color1, (band - 0.66) * 3.0)
    };
    
    // Layer 3: Wispy clouds and storms
    let cloud_pos = Vec3::new(
        fragment.position.x * 4.0 + fragment.time * 0.15,
        fragment.position.y * 8.0,
        fragment.position.z * 4.0,
    );
    let clouds = fbm(&cloud_pos, 4);
    
    // Layer 4: Dark spot (like Neptune's Great Dark Spot)
    let spot_center = Vec3::new(-0.4, 0.3, 0.7);
    let dist_to_spot = fragment.position.sub(&spot_center).length();
    let spot_size = 0.2;
    let spot_intensity = if dist_to_spot < spot_size {
        ((1.0 - dist_to_spot / spot_size) * PI / 2.0).cos().powf(2.0)
    } else {
        0.0
    };
    let spot_color = Color::from_float(0.1, 0.2, 0.4);
    
    let mut final_color = base_color;
    
    // Apply clouds
    let cloud_influence = clouds * 0.15;
    final_color = Color::from_float(
        (final_color.r as f32 / 255.0 + cloud_influence).clamp(0.0, 1.0),
        (final_color.g as f32 / 255.0 + cloud_influence).clamp(0.0, 1.0),
        (final_color.b as f32 / 255.0 + cloud_influence * 0.8).clamp(0.0, 1.0),
    );
    
    // Apply dark spot
    final_color = final_color.mix(&spot_color, spot_intensity * 0.6);
    
    let brightness = fragment.intensity * (0.6 + clouds * 0.2);
    
    Color::from_float(
        final_color.r as f32 / 255.0 * brightness,
        final_color.g as f32 / 255.0 * brightness,
        final_color.b as f32 / 255.0 * brightness,
    )
}

// Shader 5: Desert Planet (Mars-like)
fn desert_planet_shader(fragment: &Fragment) -> Color {
    // Layer 1: Base rust colors
    let rust_light = Color::from_float(0.8, 0.4, 0.2);
    let rust_dark = Color::from_float(0.5, 0.2, 0.1);
    let rust_sand = Color::from_float(0.9, 0.6, 0.3);
    
    // Layer 2: Terrain variation
    let terrain_pos = Vec3::new(
        fragment.position.x * 3.0,
        fragment.position.y * 3.0,
        fragment.position.z * 3.0,
    );
    let terrain = fbm(&terrain_pos, 5);
    
    let base_color = if terrain < 0.3 {
        rust_dark.mix(&rust_light, terrain * 3.3)
    } else if terrain < 0.7 {
        rust_light.mix(&rust_sand, (terrain - 0.3) * 2.5)
    } else {
        rust_sand.mix(&rust_dark, (terrain - 0.7) * 3.3)
    };
    
    // Layer 3: Canyon and crater systems
    let crater_pos = Vec3::new(
        fragment.position.x * 8.0,
        fragment.position.y * 8.0,
        fragment.position.z * 8.0,
    );
    let craters = turbulence(&crater_pos, 3);
    let crater_effect = (craters - 0.7).max(0.0) * 3.0;
    
    // Layer 4: Polar ice caps
    let polar = fragment.position.y.abs();
    let ice_threshold = 0.7;
    let ice_color = Color::from_float(0.95, 0.95, 1.0);
    let has_ice = polar > ice_threshold;
    let ice_amount = if has_ice {
        ((polar - ice_threshold) / (1.0 - ice_threshold)).clamp(0.0, 1.0)
    } else {
        0.0
    };
    
    let mut final_color = base_color;
    
    // Apply crater darkening
    final_color = Color::from_float(
        (final_color.r as f32 / 255.0 * (1.0 - crater_effect * 0.3)).clamp(0.0, 1.0),
        (final_color.g as f32 / 255.0 * (1.0 - crater_effect * 0.3)).clamp(0.0, 1.0),
        (final_color.b as f32 / 255.0 * (1.0 - crater_effect * 0.3)).clamp(0.0, 1.0),
    );
    
    // Apply ice caps
    final_color = final_color.mix(&ice_color, ice_amount * 0.8);
    
    let brightness = fragment.intensity * (0.5 + terrain * 0.3);
    
    Color::from_float(
        final_color.r as f32 / 255.0 * brightness,
        final_color.g as f32 / 255.0 * brightness,
        final_color.b as f32 / 255.0 * brightness,
    )
}

// Shader 6: Volcanic Planet (Io-like)
fn volcanic_planet_shader(fragment: &Fragment) -> Color {
    // Layer 1: Base yellow-orange sulfur surface
    let sulfur_yellow = Color::from_float(0.9, 0.8, 0.2);
    let sulfur_orange = Color::from_float(0.8, 0.5, 0.1);
    let sulfur_white = Color::from_float(0.95, 0.9, 0.7);
    
    let surface_pos = Vec3::new(
        fragment.position.x * 2.5,
        fragment.position.y * 2.5,
        fragment.position.z * 2.5,
    );
    let surface_variation = fbm(&surface_pos, 4);
    
    let base_color = if surface_variation < 0.4 {
        sulfur_yellow.mix(&sulfur_orange, surface_variation * 2.5)
    } else {
        sulfur_orange.mix(&sulfur_white, (surface_variation - 0.4) * 1.67)
    };
    
    // Layer 2: Volcanic hotspots (glowing lava)
    let volcano_pos = Vec3::new(
        fragment.position.x * 6.0,
        fragment.position.y * 6.0,
        fragment.position.z * 6.0 + fragment.time * 0.5,
    );
    let volcano_noise = turbulence(&volcano_pos, 4);
    let is_hotspot = volcano_noise > 0.75;
    let hotspot_intensity = if is_hotspot {
        ((volcano_noise - 0.75) * 4.0).clamp(0.0, 1.0)
    } else {
        0.0
    };
    
    // Layer 3: Lava flows
    let lava_pos = Vec3::new(
        fragment.position.x * 10.0,
        fragment.position.y * 10.0 + fragment.time * 0.3,
        fragment.position.z * 10.0,
    );
    let lava_flow = fbm(&lava_pos, 3);
    let is_lava = lava_flow > 0.65;
    let lava_amount = if is_lava {
        ((lava_flow - 0.65) * 2.86).clamp(0.0, 1.0)
    } else {
        0.0
    };
    
    // Layer 4: Atmospheric glow from volcanic gases
    let edge_intensity = 1.0 - fragment.normal.dot(&Vec3::new(0.0, 0.0, 1.0)).abs();
    let atmosphere_glow = edge_intensity.powf(2.0) * 0.3;
    
    let mut final_color = base_color;
    
    // Apply lava flows (red-orange)
    let lava_color = Color::from_float(1.0, 0.3, 0.0);
    final_color = final_color.mix(&lava_color, lava_amount * 0.7);
    
    // Apply volcanic hotspots (bright orange-red)
    let hotspot_color = Color::from_float(1.0, 0.5, 0.0);
    final_color = final_color.mix(&hotspot_color, hotspot_intensity * 0.9);
    
    let brightness = fragment.intensity * (0.7 + hotspot_intensity * 0.8 + atmosphere_glow);
    
    Color::from_float(
        final_color.r as f32 / 255.0 * brightness * (1.0 + hotspot_intensity * 0.5),
        final_color.g as f32 / 255.0 * brightness * (1.0 + hotspot_intensity * 0.3),
        final_color.b as f32 / 255.0 * brightness,
    )
}

fn generate_sphere(radius: f32, segments: usize) -> Vec<Vec3> {
    let mut vertices = Vec::new();

    for lat in 0..=segments {
        let theta = PI * lat as f32 / segments as f32;
        let sin_theta = theta.sin();
        let cos_theta = theta.cos();

        for lon in 0..=segments {
            let phi = 2.0 * PI * lon as f32 / segments as f32;
            let sin_phi = phi.sin();
            let cos_phi = phi.cos();

            let x = radius * sin_theta * cos_phi;
            let y = radius * cos_theta;
            let z = radius * sin_theta * sin_phi;

            vertices.push(Vec3::new(x, y, z));
        }
    }

    vertices
}

fn render_triangle<F>(
    buffer: &mut Vec<Color>,
    z_buffer: &mut Vec<f32>,
    v1: Vec3,
    v2: Vec3,
    v3: Vec3,
    light_dir: &Vec3,
    shader: &F,
    time: f32,
) where
    F: Fn(&Fragment) -> Color,
{
    let scale = 200.0;
    let center_x = WIDTH as f32 / 2.0;
    let center_y = HEIGHT as f32 / 2.0;

    let p1 = (center_x + v1.x * scale, center_y - v1.y * scale);
    let p2 = (center_x + v2.x * scale, center_y - v2.y * scale);
    let p3 = (center_x + v3.x * scale, center_y - v3.y * scale);

    let min_x = p1.0.min(p2.0).min(p3.0).max(0.0) as usize;
    let max_x = p1.0.max(p2.0).max(p3.0).min(WIDTH as f32 - 1.0) as usize;
    let min_y = p1.1.min(p2.1).min(p3.1).max(0.0) as usize;
    let max_y = p1.1.max(p2.1).max(p3.1).min(HEIGHT as f32 - 1.0) as usize;

    let edge1 = v2.sub(&v1);
    let edge2 = v3.sub(&v1);
    let normal = edge1.cross(&edge2).normalize();

    for y in min_y..=max_y {
        for x in min_x..=max_x {
            let px = x as f32;
            let py = y as f32;

            let v0 = (p2.0 - p1.0, p2.1 - p1.1);
            let v1_local = (p3.0 - p1.0, p3.1 - p1.1);
            let v2_local = (px - p1.0, py - p1.1);

            let dot00 = v0.0 * v0.0 + v0.1 * v0.1;
            let dot01 = v0.0 * v1_local.0 + v0.1 * v1_local.1;
            let dot02 = v0.0 * v2_local.0 + v0.1 * v2_local.1;
            let dot11 = v1_local.0 * v1_local.0 + v1_local.1 * v1_local.1;
            let dot12 = v1_local.0 * v2_local.0 + v1_local.1 * v2_local.1;

            let inv_denom = 1.0 / (dot00 * dot11 - dot01 * dot01);
            let u = (dot11 * dot02 - dot01 * dot12) * inv_denom;
            let v = (dot00 * dot12 - dot01 * dot02) * inv_denom;

            if u >= 0.0 && v >= 0.0 && u + v <= 1.0 {
                let position = v1.add(&edge1.mul(u)).add(&edge2.mul(v));
                let z = position.z;

                let idx = y * WIDTH + x;
                if z > z_buffer[idx] {
                    z_buffer[idx] = z;

                    let intensity = normal.dot(light_dir).max(0.0) * 0.8 + 0.2;

                    let fragment = Fragment {
                        position,
                        normal,
                        intensity,
                        time,
                    };

                    buffer[idx] = shader(&fragment);
                }
            }
        }
    }
}

fn render_sphere<F>(
    vertices: &[Vec3],
    segments: usize,
    shader: F,
    time: f32,
    rotation: f32,
) -> Vec<Color>
where
    F: Fn(&Fragment) -> Color,
{
    let mut buffer = vec![Color::new(0, 0, 0); WIDTH * HEIGHT];
    let mut z_buffer = vec![f32::NEG_INFINITY; WIDTH * HEIGHT];
    
    let light_dir = Vec3::new(0.5, 0.5, 1.0).normalize();

    for lat in 0..segments {
        for lon in 0..segments {
            let idx = lat * (segments + 1) + lon;
            let v1 = vertices[idx].rotate_y(rotation);
            let v2 = vertices[idx + 1].rotate_y(rotation);
            let v3 = vertices[idx + segments + 1].rotate_y(rotation);
            let v4 = vertices[idx + segments + 2].rotate_y(rotation);

            render_triangle(&mut buffer, &mut z_buffer, v1, v2, v3, &light_dir, &shader, time);
            render_triangle(&mut buffer, &mut z_buffer, v2, v4, v3, &light_dir, &shader, time);
        }
    }

    buffer
}

fn save_ppm(filename: &str, buffer: &[Color]) -> std::io::Result<()> {
    let mut file = File::create(filename)?;
    writeln!(file, "P3")?;
    writeln!(file, "{} {}", WIDTH, HEIGHT)?;
    writeln!(file, "255")?;
    
    for color in buffer {
        writeln!(file, "{} {} {}", color.r, color.g, color.b)?;
    }
    
    Ok(())
}

fn main() {
    println!("Generating Solar System renders...");
    
    let sphere_vertices = generate_sphere(1.0, 50);
    
    // Render Sun
    println!("Rendering Sun...");
    let sun_buffer = render_sphere(&sphere_vertices, 50, sun_shader, 2.5, 0.8);
    save_ppm("screenshots/sun.ppm", &sun_buffer).unwrap();
    println!("✓ Sun saved");
    
    // Render Rocky Planet
    println!("Rendering Rocky Planet...");
    let rocky_buffer = render_sphere(&sphere_vertices, 50, rocky_planet_shader, 5.0, 1.2);
    save_ppm("screenshots/rocky_planet.ppm", &rocky_buffer).unwrap();
    println!("✓ Rocky Planet saved");
    
    // Render Gas Giant
    println!("Rendering Gas Giant...");
    let gas_buffer = render_sphere(&sphere_vertices, 50, gas_giant_shader, 3.5, 0.5);
    save_ppm("screenshots/gas_giant.ppm", &gas_buffer).unwrap();
    println!("✓ Gas Giant saved");
    
    // Render Ice Giant
    println!("Rendering Ice Giant...");
    let ice_buffer = render_sphere(&sphere_vertices, 50, ice_giant_shader, 4.0, 0.3);
    save_ppm("screenshots/ice_giant.ppm", &ice_buffer).unwrap();
    println!("✓ Ice Giant saved");
    
    // Render Desert Planet
    println!("Rendering Desert Planet...");
    let desert_buffer = render_sphere(&sphere_vertices, 50, desert_planet_shader, 1.5, 1.8);
    save_ppm("screenshots/desert_planet.ppm", &desert_buffer).unwrap();
    println!("✓ Desert Planet saved");
    
    // Render Volcanic Planet
    println!("Rendering Volcanic Planet...");
    let volcanic_buffer = render_sphere(&sphere_vertices, 50, volcanic_planet_shader, 3.0, 0.7);
    save_ppm("screenshots/volcanic_planet.ppm", &volcanic_buffer).unwrap();
    println!("✓ Volcanic Planet saved");
    
    println!("\nAll renders complete! 6 planets total.");
    println!("Check the screenshots/ directory.");
}
