use std::path::Path;
use std::sync::Arc;

use mc_math::{
    BlockPos, ImprovedNoise, LegacyRandomSource, NormalNoise, PerlinNoise, RandomSource,
    SectionPos, SimpleBitStorage, SimplexNoise, VecRngTracer, XoroshiroRandomSource, atan2,
    ceil_f64, ceillog2, clamped_lerp_f64, cos, floor_f64, get_seed, lerp_f64, lfloor, log2,
    mix_stafford13, murmur_hash3_mixer, seed_from_hash_of, seed_slime_chunk, sin,
    smallest_encompassing_power_of_two, smoothstep, smoothstep_derivative, upgrade_seed_to_128bit,
};
use serde_json::Value;

fn load_vector(name: &str) -> Option<Value> {
    let p = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../reference/vectors")
        .join(name);
    if !p.exists() {
        eprintln!("[skip] golden vector {:?} not present", p);
        return None;
    }
    Some(serde_json::from_str(&std::fs::read_to_string(&p).unwrap()).unwrap())
}

fn parse_f32_hex(s: &str) -> u32 {
    u32::from_str_radix(s, 16).unwrap()
}

fn parse_f64_hex(s: &str) -> u64 {
    u64::from_str_radix(s, 16).unwrap()
}

fn check_stream(rng: &mut dyn RandomSource, data: &Value, label: &str) {
    for (i, v) in data["next_int"].as_array().unwrap().iter().enumerate() {
        assert_eq!(rng.next_int(), v.as_i64().unwrap() as i32, "{label} next_int[{i}]");
    }
    for (i, pair) in data["next_int_bounded"].as_array().unwrap().iter().enumerate() {
        let arr = pair.as_array().unwrap();
        let bound = arr[0].as_i64().unwrap() as i32;
        let expected = arr[1].as_i64().unwrap() as i32;
        assert_eq!(rng.next_int_bound(bound), expected, "{label} next_int_bound[{i}]({bound})");
    }
    for (i, v) in data["next_long"].as_array().unwrap().iter().enumerate() {
        let expected: i64 = v.as_str().unwrap().parse().unwrap();
        assert_eq!(rng.next_long(), expected, "{label} next_long[{i}]");
    }
    for (i, v) in data["next_boolean"].as_array().unwrap().iter().enumerate() {
        assert_eq!(rng.next_boolean(), v.as_bool().unwrap(), "{label} next_boolean[{i}]");
    }
    for (i, v) in data["next_float_bits"].as_array().unwrap().iter().enumerate() {
        let expected = parse_f32_hex(v.as_str().unwrap());
        assert_eq!(rng.next_float().to_bits(), expected, "{label} next_float[{i}]");
    }
    for (i, v) in data["next_double_bits"].as_array().unwrap().iter().enumerate() {
        let expected = parse_f64_hex(v.as_str().unwrap());
        assert_eq!(rng.next_double().to_bits(), expected, "{label} next_double[{i}]");
    }
    for (i, v) in data["next_gaussian_bits"].as_array().unwrap().iter().enumerate() {
        let expected = parse_f64_hex(v.as_str().unwrap());
        assert_eq!(rng.next_gaussian().to_bits(), expected, "{label} next_gaussian[{i}]");
    }

    let mut forked = rng.fork();
    for (i, v) in data["fork_longs"].as_array().unwrap().iter().enumerate() {
        let expected: i64 = v.as_str().unwrap().parse().unwrap();
        assert_eq!(forked.next_long(), expected, "{label} fork_longs[{i}]");
    }

    let pos = rng.fork_positional();
    for p in data["pos_at"].as_array().unwrap() {
        let x = p["x"].as_i64().unwrap() as i32;
        let y = p["y"].as_i64().unwrap() as i32;
        let z = p["z"].as_i64().unwrap() as i32;
        let mut pr = pos.at(x, y, z);
        assert_eq!(
            pr.next_long(),
            p["next_long"].as_str().unwrap().parse::<i64>().unwrap(),
            "{label} pos_at({x},{y},{z}) next_long"
        );
        assert_eq!(
            pr.next_int(),
            p["next_int"].as_i64().unwrap() as i32,
            "{label} pos_at({x},{y},{z}) next_int"
        );
    }
    for h in data["pos_hash"].as_array().unwrap() {
        let name = h["name"].as_str().unwrap();
        let mut hr = pos.from_hash_of(name);
        assert_eq!(
            hr.next_long(),
            h["next_long"].as_str().unwrap().parse::<i64>().unwrap(),
            "{label} pos_hash({name:?}) next_long"
        );
        assert_eq!(
            hr.next_int(),
            h["next_int"].as_i64().unwrap() as i32,
            "{label} pos_hash({name:?}) next_int"
        );
    }
}

#[test]
fn test_random_golden_vectors() {
    let Some(v) = load_vector("random.json") else { return };

    for row in v["stafford13"].as_array().unwrap() {
        let inp: i64 = row["in"].as_str().unwrap().parse().unwrap();
        let out: i64 = row["out"].as_str().unwrap().parse().unwrap();
        assert_eq!(mix_stafford13(inp), out);
    }
    for row in v["upgrade_128"].as_array().unwrap() {
        let inp: i64 = row["in"].as_str().unwrap().parse().unwrap();
        let lo: i64 = row["lo"].as_str().unwrap().parse().unwrap();
        let hi: i64 = row["hi"].as_str().unwrap().parse().unwrap();
        let got = upgrade_seed_to_128bit(inp);
        assert_eq!((got.seed_lo, got.seed_hi), (lo, hi));
    }
    for row in v["seed_from_hash"].as_array().unwrap() {
        let s = row["str"].as_str().unwrap();
        let lo: i64 = row["lo"].as_str().unwrap().parse().unwrap();
        let hi: i64 = row["hi"].as_str().unwrap().parse().unwrap();
        let got = seed_from_hash_of(s);
        assert_eq!((got.seed_lo, got.seed_hi), (lo, hi), "seed_from_hash({s:?})");
    }

    for entry in v["legacy"].as_array().unwrap() {
        let seed: i64 = entry["seed"].as_str().unwrap().parse().unwrap();
        let mut rng = LegacyRandomSource::new(seed);
        check_stream(&mut rng, &entry["data"], &format!("legacy({seed})"));
    }
    for entry in v["xoroshiro"].as_array().unwrap() {
        let seed: i64 = entry["seed"].as_str().unwrap().parse().unwrap();
        let mut rng = XoroshiroRandomSource::new(seed);
        check_stream(&mut rng, &entry["data"], &format!("xoroshiro({seed})"));
    }
    {
        let zp = &v["xoroshiro_zero_pair"];
        let mut rng = XoroshiroRandomSource::from_pair(0, 0);
        check_stream(&mut rng, &zp["data"], "xoroshiro(0,0)");
    }
    for row in v["slime_chunk"].as_array().unwrap() {
        let cx = row["cx"].as_i64().unwrap() as i32;
        let cz = row["cz"].as_i64().unwrap() as i32;
        let ws: i64 = row["world_seed"].as_str().unwrap().parse().unwrap();
        let salt: i64 = row["salt"].as_str().unwrap().parse().unwrap();
        let mut sr = seed_slime_chunk(cx, cz, ws, salt);
        assert_eq!(sr.next_int_bound(10), row["first_int_10"].as_i64().unwrap() as i32);
    }

    // Also test RNG tracer hook (Method 2)
    let tracer = Arc::new(VecRngTracer::new());
    let mut traced = LegacyRandomSource::with_tracer(42, 7, tracer.clone());
    let _ = traced.next_int();
    let _ = traced.next_long();
    let evs = tracer.snapshot();
    assert_eq!(evs.len(), 4); // setSeed + next(32) + 2*next(32)
    assert_eq!(evs[0].method, "setSeed");
    assert_eq!(evs[1].stream_id, 7);
}

#[test]
fn test_mth_golden_vectors() {
    let Some(v) = load_vector("mth.json") else { return };

    for row in v["trig"].as_array().unwrap() {
        let x = f64::from_bits(parse_f64_hex(row["in_bits"].as_str().unwrap()));
        assert_eq!(
            sin(x).to_bits(),
            parse_f32_hex(row["sin_bits"].as_str().unwrap()),
            "sin({x})"
        );
        assert_eq!(
            cos(x).to_bits(),
            parse_f32_hex(row["cos_bits"].as_str().unwrap()),
            "cos({x})"
        );
    }
    for row in v["get_seed"].as_array().unwrap() {
        let x = row["x"].as_i64().unwrap() as i32;
        let y = row["y"].as_i64().unwrap() as i32;
        let z = row["z"].as_i64().unwrap() as i32;
        let expected: i64 = row["seed"].as_str().unwrap().parse().unwrap();
        assert_eq!(get_seed(x, y, z), expected);
    }
    for row in v["murmur3"].as_array().unwrap() {
        let inp = row["in"].as_i64().unwrap() as i32;
        let out = row["out"].as_i64().unwrap() as i32;
        assert_eq!(murmur_hash3_mixer(inp), out);
    }
    for row in v["pow2"].as_array().unwrap() {
        let inp = row["in"].as_i64().unwrap() as i32;
        assert_eq!(
            smallest_encompassing_power_of_two(inp),
            row["smallest_pow2"].as_i64().unwrap() as i32
        );
        if inp >= 1 {
            assert_eq!(ceillog2(inp), row["ceillog2"].as_i64().unwrap() as i32);
            assert_eq!(log2(inp), row["log2"].as_i64().unwrap() as i32);
        }
    }
    for row in v["atan2"].as_array().unwrap() {
        let y = f64::from_bits(parse_f64_hex(row["y_bits"].as_str().unwrap()));
        let x = f64::from_bits(parse_f64_hex(row["x_bits"].as_str().unwrap()));
        let expected = parse_f64_hex(row["out_bits"].as_str().unwrap());
        let got = atan2(y, x).to_bits();
        if f64::from_bits(expected).is_nan() {
            assert!(f64::from_bits(got).is_nan());
        } else {
            assert_eq!(got, expected, "atan2({y}, {x})");
        }
    }
    for row in v["interp"].as_array().unwrap() {
        let t = f64::from_bits(parse_f64_hex(row["t_bits"].as_str().unwrap()));
        assert_eq!(smoothstep(t).to_bits(), parse_f64_hex(row["smoothstep_bits"].as_str().unwrap()));
        assert_eq!(smoothstep_derivative(t).to_bits(), parse_f64_hex(row["smoothstep_deriv_bits"].as_str().unwrap()));
        assert_eq!(lerp_f64(t, -12.34, 56.78).to_bits(), parse_f64_hex(row["lerp_bits"].as_str().unwrap()));
        assert_eq!(clamped_lerp_f64(t, -12.34, 56.78).to_bits(), parse_f64_hex(row["clamped_lerp_bits"].as_str().unwrap()));
        assert_eq!(floor_f64(t), row["floor"].as_i64().unwrap() as i32);
        assert_eq!(lfloor(t), row["lfloor"].as_str().unwrap().parse::<i64>().unwrap());
        assert_eq!(ceil_f64(t), row["ceil"].as_i64().unwrap() as i32);
    }
}

#[test]
fn test_noise_golden_vectors() {
    let Some(v) = load_vector("noise.json") else { return };

    let imp_v = &v["improved"];
    let mut r1 = XoroshiroRandomSource::new(42);
    let imp = ImprovedNoise::new(&mut r1);
    assert_eq!(imp.xo.to_bits(), parse_f64_hex(imp_v["xo_bits"].as_str().unwrap()));
    assert_eq!(imp.yo.to_bits(), parse_f64_hex(imp_v["yo_bits"].as_str().unwrap()));
    assert_eq!(imp.zo.to_bits(), parse_f64_hex(imp_v["zo_bits"].as_str().unwrap()));
    for s in imp_v["samples"].as_array().unwrap() {
        let x = f64::from_bits(parse_f64_hex(s["x_bits"].as_str().unwrap()));
        let y = f64::from_bits(parse_f64_hex(s["y_bits"].as_str().unwrap()));
        let z = f64::from_bits(parse_f64_hex(s["z_bits"].as_str().unwrap()));
        let ys = f64::from_bits(parse_f64_hex(s["y_scale_bits"].as_str().unwrap()));
        let ym = f64::from_bits(parse_f64_hex(s["y_max_bits"].as_str().unwrap()));
        assert_eq!(
            imp.noise_with_y_scale(x, y, z, ys, ym).to_bits(),
            parse_f64_hex(s["val_bits"].as_str().unwrap())
        );
        let mut d = [1.0, -2.0, 3.0];
        let dv = imp.noise_with_derivative(x, y, z, &mut d);
        assert_eq!(dv.to_bits(), parse_f64_hex(s["deriv_val_bits"].as_str().unwrap()));
        assert_eq!(d[0].to_bits(), parse_f64_hex(s["d0_bits"].as_str().unwrap()));
        assert_eq!(d[1].to_bits(), parse_f64_hex(s["d1_bits"].as_str().unwrap()));
        assert_eq!(d[2].to_bits(), parse_f64_hex(s["d2_bits"].as_str().unwrap()));
    }

    let mut r2 = XoroshiroRandomSource::new(12345);
    let perlin = PerlinNoise::create(&mut r2, -7, vec![1.0, 1.0, 2.0, 0.0, 0.5]);
    for s in v["perlin"].as_array().unwrap() {
        let x = f64::from_bits(parse_f64_hex(s["x_bits"].as_str().unwrap()));
        let y = f64::from_bits(parse_f64_hex(s["y_bits"].as_str().unwrap()));
        let z = f64::from_bits(parse_f64_hex(s["z_bits"].as_str().unwrap()));
        assert_eq!(
            perlin.get_value(x, y, z).to_bits(),
            parse_f64_hex(s["val_bits"].as_str().unwrap())
        );
    }

    let mut r3 = XoroshiroRandomSource::new(999);
    let normal = NormalNoise::create(&mut r3, -5, vec![1.0, 0.0, 1.0, 2.0]);
    for s in v["normal"].as_array().unwrap() {
        let x = f64::from_bits(parse_f64_hex(s["x_bits"].as_str().unwrap()));
        let y = f64::from_bits(parse_f64_hex(s["y_bits"].as_str().unwrap()));
        let z = f64::from_bits(parse_f64_hex(s["z_bits"].as_str().unwrap()));
        assert_eq!(
            normal.get_value(x, y, z).to_bits(),
            parse_f64_hex(s["val_bits"].as_str().unwrap())
        );
    }

    let mut r4 = LegacyRandomSource::new(777);
    let simplex = SimplexNoise::new(&mut r4);
    for s in v["simplex"].as_array().unwrap() {
        let x = f64::from_bits(parse_f64_hex(s["x_bits"].as_str().unwrap()));
        let y = f64::from_bits(parse_f64_hex(s["y_bits"].as_str().unwrap()));
        let z = f64::from_bits(parse_f64_hex(s["z_bits"].as_str().unwrap()));
        assert_eq!(
            simplex.get_value_2d(x, y).to_bits(),
            parse_f64_hex(s["val2d_bits"].as_str().unwrap())
        );
        assert_eq!(
            simplex.get_value_3d(x, y, z).to_bits(),
            parse_f64_hex(s["val3d_bits"].as_str().unwrap())
        );
    }
}

#[test]
fn test_packing_golden_vectors() {
    let Some(v) = load_vector("packing.json") else { return };

    for row in v["block_pos"].as_array().unwrap() {
        let x = row["x"].as_i64().unwrap() as i32;
        let y = row["y"].as_i64().unwrap() as i32;
        let z = row["z"].as_i64().unwrap() as i32;
        let packed: i64 = row["packed"].as_str().unwrap().parse().unwrap();
        assert_eq!(BlockPos::as_long_coords(x, y, z), packed);
        let back = BlockPos::of(packed);
        assert_eq!(back.x, row["back_x"].as_i64().unwrap() as i32);
        assert_eq!(back.y, row["back_y"].as_i64().unwrap() as i32);
        assert_eq!(back.z, row["back_z"].as_i64().unwrap() as i32);
    }

    for row in v["section_pos"].as_array().unwrap() {
        let x = row["x"].as_i64().unwrap() as i32;
        let y = row["y"].as_i64().unwrap() as i32;
        let z = row["z"].as_i64().unwrap() as i32;
        let packed: i64 = row["packed"].as_str().unwrap().parse().unwrap();
        assert_eq!(SectionPos::as_long_coords(x, y, z), packed);
        let back = SectionPos::of(packed);
        assert_eq!(back.x, row["back_x"].as_i64().unwrap() as i32);
        assert_eq!(back.y, row["back_y"].as_i64().unwrap() as i32);
        assert_eq!(back.z, row["back_z"].as_i64().unwrap() as i32);
    }

    for row in v["bit_storage"].as_array().unwrap() {
        let bits = row["bits"].as_u64().unwrap() as usize;
        let size = row["size"].as_u64().unwrap() as usize;
        let vals: Vec<i32> = row["values"]
            .as_array()
            .unwrap()
            .iter()
            .map(|x| x.as_i64().unwrap() as i32)
            .collect();
        let expected_raw: Vec<i64> = row["raw_longs"]
            .as_array()
            .unwrap()
            .iter()
            .map(|x| x.as_str().unwrap().parse().unwrap())
            .collect();
        let st = SimpleBitStorage::from_ints(bits, size, &vals).unwrap();
        assert_eq!(st.raw(), expected_raw.as_slice(), "bits={bits}");
        for (i, &expected) in vals.iter().enumerate() {
            assert_eq!(st.get(i), expected);
        }
        let mut unpacked = vec![0_i32; size];
        st.unpack(&mut unpacked);
        assert_eq!(unpacked, vals);
    }
}
